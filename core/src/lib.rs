use std::sync::Arc;
use ndarray::Array1;
use tokio::sync::RwLock;
use crate::analysis::ppg;

uniffi::include_scaffolding!("vvcore");

pub mod ble;
pub mod storage;
mod analysis;

#[derive(Debug, PartialEq, Clone)]
pub struct Device {
    mac: String,
    serial: u16,
    name: String,
    battery: u8,
    drift_us: i64,
    connected: bool,
    status: DeviceStatus,
    channels: Vec<Channel>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Channel {
    id: String,
    name: String,
    channel_type: ChannelType,
    signal_max: u16,
    signal_min: u16,
    status: ChannelStatus,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DeviceStatus {
    Ok,
    SignalIssue,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ChannelStatus {
    Ok,
    SignalIssue,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ChannelType {
    CNT,
    ECG,
    PPG,
}

pub struct VVCoreConfig {
    pub hist_size_api: u32,
    pub hist_size_analytics: u32,
    pub max_initial_rtt_ms: u32,
    pub sync_interval_min: u32,
    pub ble_mac_prefix: String,
    pub max_signal_resolution_bit: u8,
    pub max_signal_sampling_rate_hz: u8,
    pub enable_mock_devices: bool,
}

pub trait VVCoreDelegate: Send + Sync {
    fn devices_changed(&self, devices: Vec<Device>);
    fn new_data(&self, uuid: String, data: Vec<Option<u16>>);
}

pub struct VVCore {
    config: VVCoreConfig,
    delegate: Arc<dyn VVCoreDelegate>,
    storage: Arc<RwLock<storage::Storage>>,
    ble: Arc<ble::Ble>,
    rt: tokio::runtime::Runtime,
}

impl VVCore {
    pub fn new(config: VVCoreConfig, delegate: Arc<dyn VVCoreDelegate>) -> Self {
        let hist_size = config.hist_size_api.max(config.hist_size_analytics);

        let storage = storage::Storage::new(hist_size.try_into().unwrap(), config.hist_size_api.try_into().unwrap(), delegate.clone());
        let arc_storage = Arc::new(RwLock::new(storage));

        let ble = ble::Ble::new(arc_storage.clone());
        let arc_ble = Arc::new(ble);

        let rt = tokio::runtime::Runtime::new().unwrap();

        Self {
            config,
            delegate,
            storage: arc_storage,
            ble: arc_ble,
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

        let ble = self.ble.clone();
        let mac_prefix = self.config.ble_mac_prefix.clone();
        let interval = self.config.sync_interval_min as u64 * 60;

        self.rt.spawn(async move {
            let event_sender = ble.start_loop(mac_prefix).await.unwrap();

            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval));
            loop {
                interval.tick().await;
                event_sender.send(ble::BleEvent::SyncTime).await.unwrap();
            }
        });
    }

    pub fn start_analytics_loop(&self) {
        let storage = self.storage.clone();
        let delegate = self.delegate.clone();

        let analysis = ppg::Analysis {
            params: ppg::Parameters {
                sampling_frequency: 30.0,
                filter_cutoff_low: 1.0,
                filter_cutoff_high: 10.0,
                filter_order: 4,
                envelope_range: 23, // 0.5 seconds
                amplitude_min: 10,
                amplitude_max: 2000,
            },
            plotter: None
        };
        self.rt.spawn(async move {
            loop {
                let uuid_to_data = storage.read().await.get_data_for_all_channels();
                let ok = uuid_to_data.iter().filter_map(|(uuid, data)| {
                    let data = data.iter().filter_map(|x| *x).collect::<Array1<u16>>();

                    if uuid != "00:00:00:00:00:00-0" && uuid != "00:00:00:00:00:00-1"{
                        let data = data.map(|&x| -(x as f64));
                        let result = analysis.analyze(data);

                        println!("{}: {:?}", uuid, result.signal_quality);

                        let ok = result.signal_quality.iter().map(|x| if *x == 1 { 1 } else { 0 }).sum::<u16>() as f64 / result.signal_quality.len() as f64;
                        
                        return Some((uuid, ok));
                    }
                    
                    None
                }).collect::<Vec<_>>();
                
                storage.write().await.modify_devices(|devices| {
                    for device in devices.iter_mut() {
                        for channel in device.channels.iter_mut() {
                            let ok = ok.iter().find(|(x, _)| x == &&channel.id).map(|(_, x)| *x).unwrap_or(0.0);
                            channel.status = if ok > 0.75 { ChannelStatus::Ok } else { ChannelStatus::SignalIssue };
                        }
                    }
                });

                println!("Sending devices");
                
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
    }

    pub fn sync_time(&self) {
        if self.config.enable_mock_devices {
            return;
        }

        // TODO: Implement
    }
}
