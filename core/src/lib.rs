use std::collections::HashMap;
use std::sync::Arc;
use slog::{debug, error, Logger, trace};
use tokio::sync::RwLock;
use crate::analysis::{ppg, ecg};
use crate::ble::ExternalBleEvent;

uniffi::include_scaffolding!("vvcore");

pub mod ble;
pub mod storage;
mod analysis;
mod log;

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

// TODO: This is a bit of a hack, but it works for now
// We should probably have a separate module etc. for the analysis "subpackage"
pub type ECGAnalysisParameters = ecg::Parameters;
pub type ECGAnalysisResults = ecg::Results;
pub type ECGAnalysis = ecg::Analysis;

pub type PPGAnalysisParameters = ppg::Parameters;

pub type PPGAnalysisResults = ppg::Results;

pub type PPGAnalysis = ppg::Analysis;


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
    logger: Logger,
}

#[derive(Debug, PartialEq, Clone)]
enum VVCoreInternalEvent {
    SyncTime,
    Pause,
    Resume,
}

impl VVCore {
    pub fn new(config: VVCoreConfig, delegate: Arc<dyn VVCoreDelegate>) -> Self {
        let device_storage = storage::DeviceStorage::new();
        let arc_device_storage = Arc::new(RwLock::new(device_storage));

        let data_storage = storage::DataStorage::new(config.hist_size_api as usize, config.hist_size_analytics as usize);
        let arc_data_storage = Arc::new(RwLock::new(data_storage));

        let (event_broadcast, _) = tokio::sync::broadcast::channel(1000);

        let rt = tokio::runtime::Runtime::new().unwrap();

        let logger = log::create_logger("VVCore".to_string());
        
        error!(logger, "Starting VVCore"; "config" => format!("{:?}", config));

        Self {
            config,
            delegate,
            device_storage: arc_device_storage,
            data_storage: arc_data_storage,
            event_broadcast,
            rt,
            logger,
        }
    }
    
    pub fn add_logger(&mut self, logger: Logger) {
        self.logger = logger;
    }

    pub fn start_ble_loop(&self) {
        if self.config.enable_mock_devices {
            debug!(self.logger, "Starting mock BLE loop");
            let delegate = self.delegate.clone();
            let hist_size = self.config.hist_size_api;
            self.rt.spawn(async move {
                ble::mock::mock_loop(delegate, hist_size).await;
            });
            return;
        }

        let rt = &self.rt;

        let (ble_tx, mut ble_rx) = tokio::sync::mpsc::channel(1000);
        let max_initial_rtt_ms = self.config.max_initial_rtt_ms;
        let logger = self.logger.clone();
        let ble = Arc::new(ble::Ble::new(ble_tx, max_initial_rtt_ms, logger));

        let logger = self.logger.clone();
        let ble_clone = ble.clone();
        let ble_loop = rt.spawn(async move {
            debug!(logger, "Starting BLE task");
            ble_clone.run_loop().await.unwrap();
        });
        
        let ble_clone = ble.clone();
        let sync_interval = self.config.sync_interval_sec;
        let analysis_interval = self.config.analysis_interval_points;
        let logger = self.logger.clone();
        let periodic_time_sync = rt.spawn(async move {
            debug!(logger, "Starting periodic time sync task");
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(sync_interval)).await;
                ble_clone.forward_event(VVCoreInternalEvent::SyncTime).await;
            }
        });

        let ble_clone = ble.clone();
        let mut rx = self.event_broadcast.subscribe();
        let logger = self.logger.clone();
        let global_events = rt.spawn(async move {
            debug!(logger, "Starting global event handler task");
            loop {
                // forward SyncTime, Pause, Resume events to BLE
                let event = rx.recv().await;
                ble_clone.forward_event(event.unwrap()).await;
            }
        });

        let device_storage = self.device_storage.clone();
        let data_storage = self.data_storage.clone();
        let delegate = self.delegate.clone();

        let ecg_analysis = ecg::Analysis::new_with_logs(
            self.config.ecg_analysis_params.clone(),
            self.logger.clone(),
        );

        let ppg_analysis = ppg::Analysis::new_with_logs(
            self.config.ppg_analysis_params.clone(),
            self.logger.clone(),
        );

        let logger = self.logger.clone();

        let ble_event_handler = self.rt.spawn(async move {
            debug!(logger, "Starting BLE event handler task");
            loop {
                let event = ble_rx.recv().await;
                if event.is_none() {
                    break;
                }

                match event.unwrap() {
                    ExternalBleEvent::DeviceConnected(device) => {
                        trace!(logger, "Device connected: {:?}", device);
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
                        trace!(logger, "Device disconnected: {:?}", uuid);
                        let mut device_storage = device_storage.write().await;
                        if let Some(device) = device_storage.get_mut(&uuid) {
                            device.connected = false;
                            device.drift_us = 0;
                            for channel in device.channels.iter_mut() {
                                channel.signal_quality = None;
                            }
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
                        trace!(logger, "Battery level changed: {:?} {:?}", uuid, battery);
                        let mut device_storage = device_storage.write().await;
                        device_storage.get_mut(&uuid).map(|x| x.battery = battery);
                        delegate.devices_changed(device_storage.values().cloned().collect());
                    }
                    ExternalBleEvent::DriftChanged(uuid, drift) => {
                        trace!(logger, "Drift changed: {:?} {:?}", uuid, drift);
                        let mut device_storage = device_storage.write().await;
                        device_storage.get_mut(&uuid).map(|x| x.drift_us = drift);
                        delegate.devices_changed(device_storage.values().cloned().collect());
                    }
                    ExternalBleEvent::DataReceived(data) => {
                        trace!(logger, "Data received: {:?}", data);
                        let mut data_storage = data_storage.write().await;
                        let mut analysis_results = HashMap::new();

                        for (uuid, data) in data.iter() {
                            let ret = data_storage.add_datapoint(uuid.clone(), data.clone());
                            if let Some((window_api, window_analysis, channel_type, datapoint_counter)) = ret {
                                delegate.new_data(uuid.clone(), window_api.to_vec());

                                if datapoint_counter > analysis_interval {
                                    trace!(logger, "Analyzing data for {}", uuid);

                                    // TODO: Clean this up, analysis should output a single result
                                    let mut quality: Option<f32> = match channel_type {
                                        ChannelType::ECG => {
                                            let as_f64 = window_analysis.iter().filter_map(|x| x.map(|x| x as f64)).collect::<Vec<f64>>();
                                            let results = ecg_analysis.analyze(as_f64);
                                            Some(results.signal_quality as f32)
                                        }
                                        ChannelType::PPG => {
                                            let as_f64 = window_analysis.iter().filter_map(|x| x.map(|x| x as f64)).collect::<Vec<f64>>();
                                            let results = ppg_analysis.analyze(as_f64);
                                            if let Some(results) = results {
                                                Some(results.signal_quality as f32)
                                            } else {
                                                None
                                            }
                                        }
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
                            debug!(logger, "Analysis results: {:?}", analysis_results);

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

        let _handles = vec![ble_loop, periodic_time_sync, global_events, ble_event_handler];
        // maybe await on handles?
    }

    pub fn sync_time(&self) {
        if self.config.enable_mock_devices {
            return;
        }

        let _ = self.event_broadcast.send(VVCoreInternalEvent::SyncTime);
    }

    pub fn pause(&self) {
        if self.config.enable_mock_devices {
            return;
        }

        let _ = self.event_broadcast.send(VVCoreInternalEvent::Pause);
    }

    pub fn resume(&self) {
        if self.config.enable_mock_devices {
            return;
        }

        let _ = self.event_broadcast.send(VVCoreInternalEvent::Resume);
    }
}
