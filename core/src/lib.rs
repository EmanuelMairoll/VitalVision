use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use ndarray::ArrayView1;
use plotters::prelude::*;

use tokio::sync::RwLock;
use crate::analysis::{ppg, ecg};
use crate::ble::ExternalBleEvent;

uniffi::include_scaffolding!("vvcore");

pub mod ble;
pub mod storage;
mod analysis;

#[derive(Debug, PartialEq, Clone)]
pub struct Device {
    id: String,
    serial: u16,
    name: String,
    battery: u8,
    drift_us: i64,
    connected: bool,
    channels: Vec<Channel>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Channel {
    id: String,
    name: String,
    channel_type: ChannelType,
    signal_quality: Option<f32>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ChannelType {
    CNT,
    ECG,
    PPG,
}

pub type ECGAnalysisParameters = ecg::Parameters;
pub type PPGAnalysisParameters = ppg::Parameters;

#[derive(Debug, PartialEq, Clone)]
pub struct VVCoreConfig {
    pub hist_size_api: u32,
    pub hist_size_analytics: u32,
    pub max_initial_rtt_ms: u32,
    pub sync_interval_sec: u64,
    pub enable_mock_devices: bool,
    pub analysis_interval_points: u32,
    pub ecg_analysis_params: ECGAnalysisParameters,
    pub ppg_analysis_params: PPGAnalysisParameters,
}

pub trait VVCoreDelegate: Send + Sync {
    fn devices_changed(&self, devices: Vec<Device>);
    fn new_data(&self, uuid: String, data: Vec<Option<i32>>);
}

pub struct VVCore {
    config: VVCoreConfig,
    delegate: Arc<dyn VVCoreDelegate>,
    device_storage: Arc<RwLock<storage::DeviceStorage>>,
    data_storage: Arc<RwLock<storage::DataStorage>>,
    event_broadcast: tokio::sync::broadcast::Sender<VVCoreInternalEvent>,
    rt: tokio::runtime::Runtime,
}

#[derive(Debug, PartialEq, Clone)]
enum VVCoreInternalEvent {
    SyncTime,
}

impl VVCore {
    pub fn new(config: VVCoreConfig, delegate: Arc<dyn VVCoreDelegate>) -> Self {
        let device_storage = storage::DeviceStorage::new();
        let arc_device_storage = Arc::new(RwLock::new(device_storage));

        let data_storage = storage::DataStorage::new(config.hist_size_api as usize, config.hist_size_analytics as usize);
        let arc_data_storage = Arc::new(RwLock::new(data_storage));

        let (event_broadcast, _) = tokio::sync::broadcast::channel(1000);

        let rt = tokio::runtime::Runtime::new().unwrap();
        Self {
            config,
            delegate,
            device_storage: arc_device_storage,
            data_storage: arc_data_storage,
            event_broadcast,
            rt,
        }
    }

    pub fn start_ble_loop(&self) {
        if self.config.enable_mock_devices {
            let delegate = self.delegate.clone();
            let hist_size = self.config.hist_size_api;
            self.rt.spawn(async move {
                ble::mock::mock_loop(delegate, hist_size).await;
            });
            return;
        }

        let rt = &self.rt;

        let (ble_tx, mut ble_rx) = tokio::sync::mpsc::channel(1000);
        let ble = Arc::new(ble::Ble::new(ble_tx));

        let ble_clone = ble.clone();
        let ble_loop = rt.spawn(async move {
            ble_clone.run_loop().await.unwrap();
        });

        let ble_clone = ble.clone();
        let sync_interval = self.config.sync_interval_sec;
        let analysis_interval = self.config.analysis_interval_points;
        let sync_time_interval = rt.spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(sync_interval)).await;
                ble_clone.sync_time().await;
            }
        });

        let ble_clone = ble.clone();
        let mut rx = self.event_broadcast.subscribe();
        let sync_time_event = rt.spawn(async move {
            loop {
                let event = rx.recv().await.unwrap();
                match event {
                    VVCoreInternalEvent::SyncTime => {
                        ble_clone.sync_time().await;
                    }
                }
            }
        });

        let device_storage = self.device_storage.clone();
        let data_storage = self.data_storage.clone();
        let delegate = self.delegate.clone();
        
        let ecg_analysis = ecg::Analysis {
            params: self.config.ecg_analysis_params.clone(),
            plotter: None // Some(Box::new(VVCore::plot_signal)) 
        };

        let ppg_analysis = ppg::Analysis {
            params: self.config.ppg_analysis_params.clone(),
            plotter: None
        };

        let storage_handler = self.rt.spawn(async move {
            println!("Starting storage handler");
            loop {
                let event = ble_rx.recv().await;
                if event.is_none() {
                    break;
                }
                
                match event.unwrap() {
                    ExternalBleEvent::DeviceConnected(device) => {
                        let mut device_storage = device_storage.write().await;
                        device_storage.insert(device.id.clone(), device.clone());
                        delegate.devices_changed(device_storage.values().cloned().collect());
                        drop(device_storage);

                        let mut data_storage = data_storage.write().await;
                        for channel in device.channels.iter() {
                            data_storage.add_channel(channel.id.clone(), channel.channel_type.clone());
                        }
                        drop(data_storage);
                    }
                    ExternalBleEvent::DeviceDisconnected(uuid) => {
                        let mut device_storage = device_storage.write().await;
                        if let Some(device) = device_storage.get_mut(&uuid) {
                            device.connected = false;
                            let channels = device.channels.clone();
                            delegate.devices_changed(device_storage.values().cloned().collect());
                            drop(device_storage);

                            let mut data_storage = data_storage.write().await;
                            for channel in channels.iter() {
                                data_storage.remove_channel(channel.id.clone());
                            }
                            drop(data_storage);
                        }
                    }
                    ExternalBleEvent::BatteryLevelChanged(uuid, battery) => {
                        let mut device_storage = device_storage.write().await;
                        device_storage.get_mut(&uuid).map(|x| x.battery = battery);
                        delegate.devices_changed(device_storage.values().cloned().collect());
                    }
                    ExternalBleEvent::DriftChanged(uuid, drift) => {
                        let mut device_storage = device_storage.write().await;
                        device_storage.get_mut(&uuid).map(|x| x.drift_us = drift);
                        delegate.devices_changed(device_storage.values().cloned().collect());
                    }
                    ExternalBleEvent::DataReceived(data) => {
                        let mut data_storage = data_storage.write().await;
                        let mut analysis_results = HashMap::new();

                        for (uuid, data) in data.iter() {
                            let ret = data_storage.add_datapoint(uuid.clone(), data.clone());
                            if let Some((window_api, window_analysis, channel_type, datapoint_counter)) = ret {
                                delegate.new_data(uuid.clone(), window_api.to_vec());

                                if datapoint_counter > analysis_interval {
                                    println!("Analyzing data for {}", uuid);
                                    
                                    // TODO: Clean this up, analysis should output a single result
                                    let mut quality: Option<f32> = match channel_type {
                                        ChannelType::ECG => {
                                            // filter map
                                            let as_f64 = window_analysis.iter().filter_map(|x| x.map(|x| x as f64)).collect::<Vec<f64>>();
                                            let array = ndarray::Array1::from(as_f64);
                                            let results = ecg_analysis.analyze(array.view());
                                            Some(results.signal_quality.iter().map(|x| if *x == 1.0 { 1 } else { 0 }).sum::<u16>() as f32 / results.signal_quality.len() as f32)
                                        },
                                        ChannelType::PPG => {
                                            // filter map
                                            let as_f64 = window_analysis.iter().filter_map(|x| x.map(|x| x as f64)).collect::<Vec<f64>>();
                                            let array = ndarray::Array1::from(as_f64);
                                            let results = ppg_analysis.analyze(array);
                                            Some(results.signal_quality.iter().map(|x| if *x == 1 { 1 } else { 0 }).sum::<u16>() as f32 / results.signal_quality.len() as f32)
                                        },
                                        _ => None,
                                    };
                                    
                                    // treat nan as 0.0
                                    if let Some(quality_nan) = quality {
                                        if quality_nan.is_nan() {
                                            quality = Some(0.0);
                                        }
                                    }
                                    
                                    analysis_results.insert(uuid, quality);
                                    data_storage.reset_counter(uuid.clone());
                                }
                            }
                        }
                        drop(data_storage);

                        if !analysis_results.is_empty() {
                            println!("Analysis results: {:?}", analysis_results);

                            let mut device_storage = device_storage.write().await;
                            for device in device_storage.values_mut() { 
                                for channel in &mut device.channels { // TODO: This is a bit inefficient
                                    if let Some(Some(result)) = analysis_results.get(&channel.id) {
                                        channel.signal_quality = Some(*result);
                                    }
                                }
                            }
                            delegate.devices_changed(device_storage.values().cloned().collect());
                        }
                    }
                }
            }
        });

        let _handles = vec![ble_loop, sync_time_interval, sync_time_event, storage_handler];
        // maybe await on handles?
    }

    pub fn sync_time(&self) {
        if self.config.enable_mock_devices {
            return;
        }

        let _ = self.event_broadcast.send(VVCoreInternalEvent::SyncTime);
    }


    pub fn plot_signal(
        data: ArrayView1<f64>,
        title: &str,
        file_path: &str,
        points: Option<Vec<usize>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = BitMapBackend::new(file_path, (640, 480)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_value = *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap_or(&0f64);
        let min_value = *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap_or(&0f64);

        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 40).into_font())
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0..data.len() as i32, min_value..max_value)?;

        chart.configure_mesh().draw()?;

        chart.draw_series(LineSeries::new(
            data.iter().enumerate().map(|(x, y)| (x as i32, *y)),
            &RED,
        ))?;

        if let Some(indexes) = points {
            chart.draw_series(indexes.into_iter().filter_map(|index| {
                data.get(index).map(|&value| Circle::new((index as i32, value), 5, &BLUE))
            }))?;
        }

        root.present()?;
        Ok(())
    }

    
}
