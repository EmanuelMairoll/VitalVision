use async_std::sync::{Arc, RwLock};
use rand::prelude::*;

uniffi::include_scaffolding!("vvcore");

pub mod ble;
pub mod storage;

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
    fn new_data(&self, uuid: String, data: Vec<u16>);
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
        let storage = storage::Storage::new(hist_size.try_into().unwrap(), delegate.clone());
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
            self.rt.spawn(async move {
                ble::mock::mock_loop(delegate).await;
            });
            return;
        }

        let ble = self.ble.clone();
        let mac_prefix = self.config.ble_mac_prefix.clone();
        self.rt.spawn(async move {
            ble.start_loop(mac_prefix).await.unwrap();
        });
    }

    pub fn sync_time(&self) {
        let ble = self.ble.clone();
        self.rt.spawn(async move {
            ble.sync_time().await.unwrap();
        });
    }
}
