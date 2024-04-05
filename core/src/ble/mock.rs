use crate::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use std::f32::consts::PI;

pub async fn mock_loop(delegate: Arc<dyn VVCoreDelegate>) {
    let mut phase = 0.0;
    let mut devices = mock_devices();
    loop {
        delegate.devices_changed(devices.clone());

        for device in devices.iter() {
            for channel in device.channels.iter() {
                let data: Vec<u16> = (0..100).map(|i| {
                    let time = phase + i as f32 * 0.01;
                    generate_waveform(time, &channel.channel_type)
                }).collect();
                delegate.new_data(channel.id.clone(), data);
            }
        }

        phase += 0.025;
        if phase > 2.0 * PI {
            phase -= 2.0 * PI;
            devices = mock_devices();
        }

        sleep(Duration::from_millis(100)).await;
    }
}

fn generate_waveform(time: f32, channel_type: &ChannelType) -> u16 {
    let base_wave = match channel_type {
        ChannelType::PPG => {
            // Generate a smoother, slower waveform for PPG
            20480.0 + (10240.0 * (2.0 * PI * time * 1.0).sin())
        },
        ChannelType::ECG => {
            // Generate a faster, more complex waveform for ECG
            20480.0 + (10240.0 * (2.0 * PI * time * 4.0).sin() + 5120.0 * (2.0 * PI * time * 20.0).sin())
        },
        _ => 0.0,
    };

    // Add random noise and ensure the waveform stays within the u16 range
    let noise: f32 = rand::random::<f32>() * 200.0 - 100.0; // Random noise between -100 and +100
    let waveform_value = base_wave + noise;
    waveform_value.clamp(0.0, u16::MAX as f32) as u16
}

fn mock_devices() -> Vec<Device> {
    let mut rng = rand::thread_rng();
    vec![
        Device {
            mac: "00:11:22:33:00:01".to_string(),
            serial: 1,
            name: "Device 1".to_string(),
            battery: rng.gen_range(0..100),
            drift_us: 30,
            connected: true,
            status: DeviceStatus::Ok,
            channels: vec![
                Channel {
                    id: "00:11:22:33:00:01-1".to_string(),
                    name: "PPG".to_string(),
                    channel_type: ChannelType::PPG,
                    signal_min: 0,
                    signal_max: u16::MAX,
                    status: ChannelStatus::Ok,
                },
                Channel {
                    id: "00:11:22:33:00:01-2".to_string(),
                    name: "PPG".to_string(),
                    channel_type: ChannelType::PPG,
                    signal_min: 0,
                    signal_max: u16::MAX,
                    status: ChannelStatus::Ok,
                },
                Channel {
                    id: "00:11:22:33:00:01-3".to_string(),
                    name: "PPG".to_string(),
                    channel_type: ChannelType::PPG,
                    signal_min: 0,
                    signal_max: u16::MAX,
                    status: ChannelStatus::Ok,
                },
                Channel {
                    id: "00:11:22:33:00:01-4".to_string(),
                    name: "ECG".to_string(),
                    channel_type: ChannelType::ECG,
                    signal_min: 0,
                    signal_max: u16::MAX,
                    status: ChannelStatus::Ok,
                },
            ],
        },
        Device {
            mac: "00:11:22:33:00:02".to_string(),
            serial: 2,
            name: "Device 2".to_string(),
            battery: rng.gen_range(0..100),
            drift_us: 30,
            connected: true,
            status: DeviceStatus::Ok,
            channels: vec![
                Channel {
                    id: "00:11:22:33:00:02-1".to_string(),
                    name: "PPG".to_string(),
                    channel_type: ChannelType::PPG,
                    signal_min: 0,
                    signal_max: u16::MAX,
                    status: ChannelStatus::Ok,
                },
                Channel {
                    id: "00:11:22:33:00:02-2".to_string(),
                    name: "PPG".to_string(),
                    channel_type: ChannelType::PPG,
                    signal_min: 0,
                    signal_max: u16::MAX,
                    status: ChannelStatus::Ok,
                },
                Channel {
                    id: "00:11:22:33:00:02-3".to_string(),
                    name: "PPG".to_string(),
                    channel_type: ChannelType::PPG,
                    signal_min: 0,
                    signal_max: u16::MAX,
                    status: ChannelStatus::Ok,
                },
                Channel {
                    id: "00:11:22:33:00:02-4".to_string(),
                    name: "ECG".to_string(),
                    channel_type: ChannelType::ECG,
                    signal_min: 0,
                    signal_max: u16::MAX,
                    status: ChannelStatus::Ok,
                },
            ],
        },
    ]
}
