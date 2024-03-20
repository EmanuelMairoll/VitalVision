use async_std::future::{pending, timeout};
use rand::prelude::*;
use std::time::Duration;

uniffi::include_scaffolding!("vvcore");

pub struct Device {
    uuid: String,
    name: String,
    battery: u8,
    connected: bool,
    status: DeviceStatus,
    channels: Vec<Channel>,
}

pub struct Channel {
    uuid: String,
    name: String,
    channel_type: ChannelType,
    status: ChannelStatus,
}

pub enum DeviceStatus {
    Ok,
    SignalIssue,
}

pub enum ChannelStatus {
    Ok,
    SignalIssue,
}

pub enum ChannelType {
    ECG,
    PPG,
}

pub struct VVCore {}

impl VVCore {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run_ble_loop(&self) {
        println!("BLE loop started");
        loop {
            async_std::task::sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn devices(&self, wait_for_change: bool) -> Vec<Device> {
        if wait_for_change {
            let never = pending::<()>();
            timeout(Duration::from_secs(10), never).await.unwrap_err();
        }

        let mut rng = rand::thread_rng();

        vec![
            Device {
                uuid: "123".to_string(),
                name: "Device 1".to_string(),
                battery: rng.gen_range(0..100),
                connected: true,
                status: DeviceStatus::Ok,
                channels: vec![
                    Channel {
                        uuid: "124".to_string(),
                        name: "ECG".to_string(),
                        channel_type: ChannelType::ECG,
                        status: ChannelStatus::Ok,
                    },
                    Channel {
                        uuid: "125".to_string(),
                        name: "PPG".to_string(),
                        channel_type: ChannelType::PPG,
                        status: ChannelStatus::Ok,
                    },
                ],
            },
            Device {
                uuid: "126".to_string(),
                name: "Device 2".to_string(),
                battery: rng.gen_range(0..100),
                connected: true,
                status: DeviceStatus::Ok,
                channels: vec![
                    Channel {
                        uuid: "127".to_string(),
                        name: "ECG".to_string(),
                        channel_type: ChannelType::ECG,
                        status: ChannelStatus::Ok,
                    },
                    Channel {
                        uuid: "128".to_string(),
                        name: "PPG".to_string(),
                        channel_type: ChannelType::PPG,
                        status: ChannelStatus::Ok,
                    },
                ],
            },
        ]
    }

    pub async fn device(&self, uuid: String, wait_for_change: bool) -> Device {
        if wait_for_change {
            let never = pending::<()>();
            timeout(Duration::from_secs(1), never).await.unwrap_err();
        }

        let mut rng = rand::thread_rng();

        Device {
            uuid: uuid.clone(),
            name: "Device 1".to_string(),
            battery: rng.gen_range(0..100),
            connected: true,
            status: DeviceStatus::Ok,
            channels: vec![
                Channel {
                    uuid: "123".to_string(),
                    name: "ECG".to_string(),
                    channel_type: ChannelType::ECG,
                    status: ChannelStatus::Ok,
                },
                Channel {
                    uuid: "124".to_string(),
                    name: "PPG".to_string(),
                    channel_type: ChannelType::PPG,
                    status: ChannelStatus::Ok,
                },
            ],
        }
    }

    pub async fn channel_data(&self, channel_uuid: String, wait_for_change: bool) -> Vec<u16> {
        if wait_for_change {
            let never = pending::<()>();
            timeout(Duration::from_secs(1), never).await.unwrap_err();
        }

        let mut rng = rand::thread_rng();

        (0..100).map(|_| rng.gen_range(0..100)).collect()
    }
}
