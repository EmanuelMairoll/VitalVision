use async_std::sync::RwLock;
use btleplug::api::{
    Central, CentralEvent, Manager as _, Peripheral, PeripheralProperties, ScanFilter,
    ValueNotification,
};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;
use std::io::Cursor;
use std::sync::Arc;
use uuid::Uuid;

use super::*;
use crate::storage::Storage;

pub struct Ble {
    storage: Arc<RwLock<Storage>>,
}

impl Ble {
    pub fn new(storage: Arc<RwLock<Storage>>) -> Self {
        Self { storage }
    }

    pub async fn run_loop(&self, service_filter_uuid: String) -> Result<(), Box<dyn Error>> {
        let manager = Manager::new().await?;
        let central = self.get_central(&manager).await?;
        self.scan_and_handle_devices(&central, &service_filter_uuid)
            .await?;
        Ok(())
    }

    async fn get_central(&self, manager: &Manager) -> Result<Adapter, Box<dyn Error>> {
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().next().ok_or("No adapter found")?;
        Ok(central)
    }

    async fn scan_and_handle_devices(
        &self,
        central: &Adapter,
        service_filter_uuid: &str,
    ) -> Result<(), Box<dyn Error>> {
        let filter_uuid = Uuid::parse_str(service_filter_uuid)?;
        central.start_scan(ScanFilter { services: vec![] }).await?;
        println!("Scanning for devices...");
        let mut events = central.events().await?;

        while let Some(event) = events.next().await {
            if let CentralEvent::DeviceDiscovered(id) = event {
                let device = central.peripheral(&id).await?;
                self.handle_device(&device, &filter_uuid).await?;
            }
        }
        Ok(())
    }

    async fn handle_device(
        &self,
        device: &impl Peripheral,
        filter_uuid: &Uuid,
    ) -> Result<(), Box<dyn Error>> {
        let properties = match device.properties().await? {
            Some(properties) => properties,
            None => return Err("Device properties not found".into()),
        };
        println!("Found device: {:?}", properties.address);

        if properties.local_name == Some("Dialog Peripheral".to_string())
            && !device.is_connected().await?
        {
            device.connect().await?;
            println!("Connected to device: {:?}", properties.address);
            self.discover_services_and_subscribe(device, filter_uuid)
                .await?;
            let (battery, channel_mapping) = self.get_battery_and_channel_mapping(device).await?;
            let channels = self
                .create_storage_entry(properties.clone(), battery, channel_mapping.clone())
                .await?;
            self.handle_notifications(device, properties, channels)
                .await?;
        }
        Ok(())
    }

    async fn discover_services_and_subscribe(
        &self,
        device: &impl Peripheral,
        filter_uuid: &Uuid,
    ) -> Result<(), Box<dyn Error>> {
        device.discover_services().await?;
        for service in device.services() {
            if service.uuid == *filter_uuid {
                for characteristic in &service.characteristics {
                    device.subscribe(characteristic).await?;
                    println!("Subscribed to characteristic: {:?}", characteristic.uuid);
                }
            }
        }
        Ok(())
    }

    async fn get_battery_and_channel_mapping(
        &self,
        device: &impl Peripheral,
    ) -> Result<(u8, String), Box<dyn Error>> {
        let characteristic_uuid_battery = Uuid::parse_str("00002a19-0000-1000-8000-00805f9b34fb")?;
        let characteristic_uuid_channel_mapping =
            Uuid::parse_str("0000180a-0000-1000-8000-00805f9b34fb")?;
        let mut battery = 0;
        let mut channel_mapping = "CNT,PPG,PPG,ECG".to_string(); // TODO: Replace with actual channel mapping

        for service in device.services() {
            for characteristic in &service.characteristics {
                if characteristic.uuid == characteristic_uuid_battery {
                    let value = device.read(&characteristic).await?;
                    if let Some(&battery_level) = value.first() {
                        battery = battery_level;
                    }
                } else if characteristic.uuid == characteristic_uuid_channel_mapping {
                    let value = device.read(&characteristic).await?;
                    channel_mapping = String::from_utf8(value)?;
                }
            }
        }
        Ok((battery, channel_mapping))
    }

    async fn create_storage_entry(
        &self,
        properties: PeripheralProperties,
        battery: u8,
        channel_mapping: String,
    ) -> Result<Vec<Channel>, Box<dyn Error>> {
        // channel mapping is a comma separated string of service names
        // e.g. "CNT,PPG,PPG,ECG"

        let uuid = properties.address.to_string();

        let channels: Vec<Channel> = channel_mapping
            .split(',')
            .filter_map(|service_name| {
                // create a new channel for each
                let cuuid = Uuid::new_v4().to_string();

                let channel = match service_name {
                    "CNT" => Channel {
                        uuid: cuuid,
                        name: "CNT".to_string(),
                        channel_type: ChannelType::CNT,
                        status: ChannelStatus::Ok,
                    },
                    "PPG" => Channel {
                        uuid: cuuid,
                        name: "PPG".to_string(),
                        channel_type: ChannelType::PPG,
                        status: ChannelStatus::Ok,
                    },
                    "ECG" => Channel {
                        uuid: cuuid,
                        name: "ECG".to_string(),
                        channel_type: ChannelType::ECG,
                        status: ChannelStatus::Ok,
                    },
                    _ => return None,
                };

                Some(channel)
            })
            .collect();

        let storage_device = Device {
            uuid,
            name: properties.local_name.unwrap_or("Unknown".to_string()),
            battery,
            connected: true,
            status: DeviceStatus::Ok,
            channels: channels.clone(),
        };

        let mut storage = self.storage.write().await;
        storage.modify_devices(|devices| devices.push(storage_device));
        drop(storage);
        Ok(channels)
    }

    async fn handle_notifications(
        &self,
        device: &impl Peripheral,
        properties: PeripheralProperties,
        channels: Vec<Channel>,
    ) -> Result<(), Box<dyn Error>> {
        let device_uuid = properties.address.to_string();

        let characteristic_uuid_battery = Uuid::parse_str("00002a19-0000-1000-8000-00805f9b34fb")?; // Example battery level UUID
        let characteristic_uuid_data = Uuid::parse_str("dcf31a27-a904-f4a3-a24e-5ae42f8617b6")?; // Custom data characteristic 1

        let mut notification_stream = device.notifications().await?;
        println!("Waiting for notifications...");

        while let Some(notification) = notification_stream.next().await {
            let ValueNotification { uuid, value, .. } = notification;
            match uuid {
                uuid if uuid == characteristic_uuid_battery => {
                    // Assuming the battery level is a single byte indicating the percentage
                    if let Some(&battery_level) = value.first() {
                        println!("Battery level update: {}%", battery_level);
                        self.update_battery_status(device_uuid.clone(), battery_level)
                            .await?;
                    }
                }
                uuid if uuid == characteristic_uuid_data => {
                    let data_points: Vec<(String, u16)> =
                        self.parse_data_points(&value, channels.clone());
                    for data in data_points {
                        self.store_data_point(data.0.to_string(), data.1).await;
                    }
                }
                _ => println!(
                    "Received notification from an unknown characteristic: {:?}",
                    uuid
                ),
            }
        }
        Ok(())
    }

    async fn update_battery_status(
        &self,
        device_uuid: String,
        battery_level: u8,
    ) -> Result<(), Box<dyn Error>> {
        let mut storage = self.storage.write().await;
        storage.modify_devices(|devices| {
            for device in devices.iter_mut() {
                if device.uuid == device_uuid {
                    device.battery = battery_level;
                }
            }
        });
        drop(storage);
        Ok(())
    }

    fn parse_data_points(
        &self,
        value: &[u8], // Count is u8, PPG is u16, ECG is u16
        channels: Vec<Channel>,
    ) -> Vec<(String, u16)> {
        let mut data_points = Vec::new();
        let mut value_index = 0;

        for channel in channels {
            let data_point = match channel.channel_type {
                ChannelType::CNT => {
                    let data = value[value_index] as u16;
                    value_index += 1;
                    (channel.uuid, data)
                }
                ChannelType::PPG => {
                    let data = u16::from_be_bytes([value[value_index], value[value_index + 1]]);
                    value_index += 2;
                    (channel.uuid, data)
                }
                ChannelType::ECG => {
                    let data = u16::from_be_bytes([value[value_index], value[value_index + 1]]);
                    value_index += 2;
                    (channel.uuid, data)
                }
            };
            data_points.push(data_point);
        }

        data_points
    }

    async fn store_data_point(&self, channel_uuid: String, data_point: u16) {
        let mut storage = self.storage.write().await;
        storage.add_datapoint(channel_uuid, data_point);
        drop(storage);
    }
}
