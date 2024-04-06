use crate::*;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use std::f32::consts::PI;

pub async fn mock_loop(delegate: Arc<dyn VVCoreDelegate>, hist_size: u32) {
    let mut phase = 0.0;
    let mut devices = mock_devices();
    loop {
        delegate.devices_changed(devices.clone());

        for device in devices.iter() {
            for channel in device.channels.iter() {
                let data: Vec<Option<u16>> = (0..hist_size).map(|i| {
                    let time = phase + i as f32 * 0.01;
                    generate_waveform(time, &channel.channel_type)
                }).map(|x| Some(x))
                    .collect();
                delegate.new_data(channel.id.clone(), data);
            }
        }

        phase += 0.05;
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
            //20480.0 + (10240.0 * (2.0 * PI * time * 4.0).sin() + 5120.0 * (2.0 * PI * time * 20.0).sin())
            generate_ecg_waveform(time)
        },
        _ => 0.0,
    };

    // Add random noise and ensure the waveform stays within the u16 range
    let noise: f32 = rand::random::<f32>() * 200.0 - 100.0; // Random noise between -100 and +100
    let waveform_value = base_wave + noise;
    waveform_value.clamp(0.0, u16::MAX as f32) as u16
}

fn generate_ecg_waveform(time: f32) -> f32 {
    // Constants for waveform characteristics
    let p_wave_height = 0.1;
    let qrs_complex_height = 0.5;
    let t_wave_height = 0.3;

    let cycle_time = time % 1.0; // Simulate a repeating cycle every 1 second

    let ecg_value = if cycle_time < 0.1 {
        // P wave
        (cycle_time / 0.1) * p_wave_height
    } else if cycle_time < 0.2 {
        // Downturn to Q
        ((0.15 - cycle_time) / 0.05) * p_wave_height
    } else if cycle_time < 0.25 {
        0.0
    } else if cycle_time < 0.35 {
        // QRS Complex
        ((cycle_time - 0.25) / 0.05) * qrs_complex_height
    } else if cycle_time < 0.40 {
        // S wave
        ((0.35 - cycle_time) / 0.05) * qrs_complex_height
    } else if cycle_time < 0.55 {
        0.0
    } else if cycle_time < 0.70 {
        // T wave
        ((cycle_time - 0.45) / 0.25) * t_wave_height - ((cycle_time - 0.45) / 0.25) * t_wave_height * (cycle_time - 0.45) / 0.25
    } else {
        // Baseline
        0.0
    };

    // Normalize to fit into the u16 range, assuming ecg_value varies from -1 to 1
    let normalized_value = ((ecg_value + 1.0) / 2.0) * (u16::MAX as f32);
    normalized_value.clamp(0.0, u16::MAX as f32)
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
