use async_std::sync::{Arc, RwLock};
use rand::prelude::*;
use tokio::sync::watch::{self, Receiver, Sender};

uniffi::include_scaffolding!("vvcore");

pub mod ble;
mod mock;
pub mod storage;

#[derive(Debug, PartialEq, Clone)]
pub struct Device {
    uuid: String,
    name: String,
    battery: u8,
    connected: bool,
    status: DeviceStatus,
    channels: Vec<Channel>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Channel {
    uuid: String,
    name: String,
    channel_type: ChannelType,
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
    ECG,
    PPG,
}

pub struct VVCoreConfig {
    pub hist_size: u32,
    pub ble_service_filter: String,
    pub mock_data: bool,
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
        let storage = storage::Storage::new(config.hist_size.try_into().unwrap(), delegate.clone());
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
        if self.config.mock_data {
            let delegate = self.delegate.clone();
            self.rt.spawn(async move {
                mock::mock_loop(delegate).await;
            });
            return;
        }

        let ble = self.ble.clone();
        let filter = self.config.ble_service_filter.clone();
        self.rt.spawn(async move {
            ble.run_loop(filter).await.unwrap();
        });
    }
}
