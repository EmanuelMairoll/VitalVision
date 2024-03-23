use crate::*;

pub async fn mock_loop(delegate: Arc<dyn VVCoreDelegate>) {
    loop {
        let devices = mock_devices();
        delegate.devices_changed(devices.clone());

        for device in devices.iter() {
            for channel in device.channels.iter() {
                let data = (0..100).map(|_| rand::random::<u16>()).collect();
                delegate.new_data(channel.uuid.clone(), data);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

fn mock_devices() -> Vec<Device> {
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
