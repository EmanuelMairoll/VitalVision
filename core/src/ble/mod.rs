use super::*;
use crate::storage::Storage;
use async_std::sync::RwLock;
use ble_date_converter::*;
use btleplug::api::{
    Central, CentralEvent, Manager as _, Peripheral, PeripheralProperties,
    ScanFilter, ValueNotification, WriteType,
};
use btleplug::platform::{Adapter, Manager};
use chrono::Utc;
use futures::stream::StreamExt;
use std::error::Error;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::broadcast::Receiver;
use uuid::Uuid;

mod ble_date_converter;
pub mod mock;

pub struct Ble {
    storage: Arc<RwLock<Storage>>,
    notification_center: tokio::sync::broadcast::Sender<BleNotification>,
}

#[derive(Clone, Debug)]
enum BleNotification {
    SyncTime,
}

impl Ble {
    pub fn new(storage: Arc<RwLock<Storage>>) -> Self {
        let (notification_center, _) = tokio::sync::broadcast::channel(10);
        Self {
            storage,
            notification_center,
        }
    }

    pub async fn start_loop(&self, mac_prefix: String) -> Result<(), Box<dyn Error>> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().next().ok_or("No adapter found")?;
        let arc_central = Arc::new(central);

        arc_central
            .start_scan(ScanFilter { services: vec![] })
            .await?;
        println!("Scanning for devices...");


        let central_event_handle = tokio::spawn ({
            let central = arc_central.clone();
            let storage = self.storage.clone();
            async move {
                Ble::run_central_event_loop(central, mac_prefix, storage).await.unwrap();
            }
        });

        let notification_handle = tokio::spawn({
            let central = arc_central.clone();
            let notification_center = self.notification_center.subscribe();
            let storage = self.storage.clone();
            async move {
                Ble::run_notification_loop(central, notification_center, storage).await.unwrap();
            }
        });

        let handles = vec![central_event_handle, notification_handle];
        futures::future::join_all(handles).await;

        Ok(())
    }

    pub async fn sync_time(&self) -> Result<(), Box<dyn Error>> {
        self.notification_center.send(BleNotification::SyncTime)?;
        Ok(())
    }

    async fn run_central_event_loop(central: Arc<impl Central>, mac_prefix: String, storage: Arc<RwLock<Storage>>) -> Result<(), Box<dyn Error>> {
        let mut events = central.events().await?;
        while let Some(event) = events.next().await {
            match event {
                CentralEvent::DeviceDiscovered(id) => {
                    println!("Device discovered: {:?}", id);
                    let device = central.clone().peripheral(&id).await?;
                    Ble::handle_discovered_device(&device, &mac_prefix, storage.clone()).await?;
                }
                CentralEvent::DeviceDisconnected(id) => {
                    println!("Device disconnected: {:?}", id);
                    let device = central.clone().peripheral(&id).await?;
                    if let Ok(props) = device.properties().await {
                        if let Some(properties) = props {
                            Ble::mark_device_as_disconnected(&properties.address.to_string(), storage.clone())
                                .await?;
                        }
                    }
                }
                _ => {
                    println!("Unhandled event: {:?}", event);
                }
            }
        }
        Ok(())
    }

    #[allow(irrefutable_let_patterns)]
    async fn run_notification_loop(central: Arc<impl Central>, mut rx: Receiver<BleNotification>,storage: Arc<RwLock<Storage>>) -> Result<(), Box<dyn Error>> {
        while let Ok(notification) = rx.recv().await {
            if let BleNotification::SyncTime = notification {
                for p in central.peripherals().await.unwrap() {
                    println!("Checking device: {:?}", p.properties().await.unwrap().unwrap().address);
                    if !p.is_connected().await.unwrap() {
                        continue;
                    }
                    println!("Device is connected: {:?}", p.properties().await.unwrap().unwrap().address);

                    if let Ok(props) = p.properties().await {
                        if let Some(properties) = props {
                            let mac = Ble::get_mac_from_properties(&properties);
                            let rtt = Ble::sync_time_for_device(&p).await.unwrap();

                            storage.write().await.modify_devices(|devices| {
                                devices
                                    .iter_mut()
                                    .find(|device| device.mac == mac)
                                    .map(|device| device.drift_us = rtt);
                            });
                        }
                    }
                }

            }
        }
        Ok(())

        /*
tokio::spawn(async move {
    while let Ok(notification) = rx.recv().await {
        if let BleNotification::SyncTime = notification {
            let mac = Ble::get_mac_from_properties(&properties_clone);
            let drift = 0; // Ble::sync_time_for_device(&arc_device_clone).await.unwrap();

            arc_device_clone.services();

            let mut storage = storage.write().await;
            storage.modify_devices(|devices| {
                devices
                    .iter_mut()
                    .find(|device| device.mac == mac)
                    .map(|device| device.drift_us = drift);
            });
        }
    }
});*/

    }

    async fn handle_discovered_device(
        device: &impl Peripheral,
        mac_prefix: &str,
        storage: Arc<RwLock<Storage>>,
    ) -> Result<(), Box<dyn Error>> {
        let properties = match device.properties().await? {
            Some(properties) => properties,
            None => return Err("Device properties not found".into()),
        };
        println!(
            "Found device: {:?}, {:?}",
            properties.local_name, properties.address
        );

        let matches = properties.address.to_string().starts_with(mac_prefix);
        println!("Matches prefix: {}", matches);

        if properties.local_name == Some("Dialog Peripheral".to_string())
            && !device.is_connected().await? || properties.local_name == Some("SIP Vitaltracker".to_string()) && !device.is_connected().await?
        {
            device.connect().await?;
            println!("Connected to device: {:?}", properties.address);
            device.discover_services().await?;

            Ble::subscribe_to_data(device).await?;
            let drift = Ble::sync_time_for_device(device).await?;
            let (battery, channel_mapping) =
                Ble::get_battery_and_channel_mapping(device).await?;
            let channels = Ble::create_storage_entry(properties.clone(), battery, drift, channel_mapping.clone(), storage.clone())
                .await?;
            Ble::handle_notifications(device, properties, channels, storage)
                .await?;
        }
        Ok(())
    }

    fn get_mac_from_properties(properties: &PeripheralProperties) -> String {
        properties.address.to_string()
    }

    async fn sync_time_for_device(device: &impl Peripheral) -> Result<i64, Box<dyn Error>> {
        let time_service_uuid = Uuid::parse_str("00001806-0000-1000-8000-00805f9b34fb")?;
        let time_characteristic_uuid = Uuid::parse_str("00002a2d-0000-1000-8000-00805f9b34fb")?;

        for service in device.services() {
            if service.uuid == time_service_uuid {
                for characteristic in &service.characteristics {
                    if characteristic.uuid == time_characteristic_uuid {
                        let time_to_set = Utc::now();
                        let data_to_set = time_to_ble_data(time_to_set);
                        device
                            .write(characteristic, &data_to_set, WriteType::WithoutResponse)
                            .await?;

                        let data_read = device.read(characteristic).await?;
                        let time_to_compare = Utc::now();

                        let time_read = ble_data_to_time(&data_read)?;

                        let rtt = time_to_compare.timestamp_micros() - time_read.timestamp_micros();

                        return Ok(rtt);
                    }
                }
            }
        }

        Err("Time service or characteristic not found".into())
    }

    async fn subscribe_to_data(device: &impl Peripheral) -> Result<(), Box<dyn Error>> {
        let data_uuid = Uuid::parse_str("DCF31A27-A904-F3A3-AA4E-5AE42F1217B6")?;

        for service in device.services() {
            if service.uuid == data_uuid {
                for characteristic in &service.characteristics {
                    device.subscribe(characteristic).await?;
                    println!("Subscribed to characteristic: {:?}", characteristic.uuid);
                }
            }
        }
        Ok(())
    }

    async fn get_battery_and_channel_mapping(
        device: &impl Peripheral,
    ) -> Result<(u8, String), Box<dyn Error>> {
        let characteristic_uuid_battery = Uuid::parse_str("00002a19-0000-1000-8000-00805f9b34fb")?;
        let characteristic_uuid_channel_mapping =
            Uuid::parse_str("0000180a-0000-1000-8000-00805f9b34fb")?;
        let mut battery = 0;
        let mut channel_mapping = "CNT,ECG,PPG,PPG,PPG".to_string(); // TODO: Replace with actual channel mapping

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
        properties: PeripheralProperties,
        battery: u8,
        drift: i64,
        channel_mapping: String,
        storage: Arc<RwLock<Storage>>,
    ) -> Result<Vec<Channel>, Box<dyn Error>> {
        let mac = Ble::get_mac_from_properties(&properties);

        let last_two_bytes: String = mac.split(':').skip(4).take(2).collect::<String>();
        let serial = u16::from_str_radix(&last_two_bytes, 16).unwrap();
        let name = properties
            .local_name
            .unwrap_or_else(|| "Unknown".to_string());

        let channels: Vec<Channel> = Ble::parse_channel_mapping(&mac, &channel_mapping);

        let mut storage = storage.write().await;

        let storage_device = Device {
            mac,
            serial,
            name,
            battery,
            drift_us: drift,
            connected: true,
            status: DeviceStatus::Ok,
            channels: channels.clone(),
        };

        storage.modify_devices(|devices| {
            devices.retain(|d| d.mac != storage_device.mac);
            devices.push(storage_device);
        });

        Ok(channels)
    }

    fn parse_channel_mapping(mac: &str, channel_mapping: &str) -> Vec<Channel> {
        channel_mapping
            .split(',')
            .enumerate()
            .map(|(idx, service_name)| match service_name {
                "CNT" => Channel {
                    id: format!("{}-{}", mac, idx),
                    name: format!("CNT{}", idx),
                    channel_type: ChannelType::CNT,
                    signal_max: 256,
                    signal_min: 0,
                    status: ChannelStatus::Ok,
                },
                "PPG" => Channel {
                    id: format!("{}-{}", mac, idx),
                    name: format!("PPG{}", idx),
                    channel_type: ChannelType::PPG,
                    signal_max: u16::MAX,
                    signal_min: 0,
                    status: ChannelStatus::Ok,
                },
                "ECG" => Channel {
                    id: format!("{}-{}", mac, idx),
                    name: format!("ECG{}", idx),
                    channel_type: ChannelType::ECG,
                    signal_max: u16::MAX,
                    signal_min: 0,
                    status: ChannelStatus::Ok,
                },
                _ => panic!("Unknown channel type: {}", service_name),
            })
            .collect()
    }

    async fn handle_notifications(
        device: &impl Peripheral,
        properties: PeripheralProperties,
        channels: Vec<Channel>,
        storage: Arc<RwLock<Storage>>,
    ) -> Result<(), Box<dyn Error>> {
        let device_mac = properties.address.to_string();

        let characteristic_uuid_battery = Uuid::parse_str("00002a19-0000-1000-8000-00805f9b34fb")?; // Example battery level UUID
        let characteristic_uuid_data = Uuid::parse_str("dcf31a27-a904-f4a3-a24e-5ae42f8617b6")?; // Custom data characteristic 1

        let mut notification_stream = device.notifications().await?;
        println!("Waiting for notifications...");

        while let Some(notification) = notification_stream.next().await {
            let ValueNotification { uuid, value, .. } = notification;
            match uuid {
                uuid if uuid == characteristic_uuid_battery => {
                    if let Some(&battery_level) = value.first() {
                        println!("Battery level update: {}%", battery_level);
                        Ble::update_battery_status(device_mac.clone(), battery_level, storage.clone())
                            .await?;
                    }
                }
                uuid if uuid == characteristic_uuid_data => {
                    //println!("Received data notification: {:?}", value);
                    let data_points: Vec<(String, u16)> =
                        Ble::parse_data_points(&value, channels.clone(), 3);
                    for data in data_points {
                        Ble::store_data_point(data.0.to_string(), data.1, storage.clone()).await;
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
        device_mac: String,
        battery_level: u8,
        storage: Arc<RwLock<Storage>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut storage = storage.write().await;
        storage.modify_devices(|devices| {
            for device in devices.iter_mut() {
                if device.mac == device_mac {
                    device.battery = battery_level;
                }
            }
        });
        drop(storage);
        Ok(())
    }

    // TODO: refactor this mess once we settle for a final data format
    fn parse_data_points(
        value: &[u8], // Count is u8, PPG is u16, ECG is u16
        channels: Vec<Channel>,
        data_points_per_message: usize,
    ) -> Vec<(String, u16)> {
        let mut data_points = Vec::new();
        let mut value_index = 0;

        for _ in 0..data_points_per_message {
            for channel in channels.clone() {
                let data_point = match channel.channel_type {
                    ChannelType::CNT => {
                        if value_index >= 1 {
                            continue;
                        }
                        
                        let data = value[value_index] as u16;
                        value_index += 1;
                        (channel.id, data)
                    }
                    ChannelType::PPG => {
                        let data = u16::from_le_bytes([value[value_index], value[value_index + 1]]);
                        value_index += 2;
                        (channel.id, data)
                    }
                    ChannelType::ECG => {
                        let data = u16::from_le_bytes([value[value_index], value[value_index + 1]]);
                        value_index += 2;
                        (channel.id, data)
                    }
                };
                data_points.push(data_point);
            }   
        }
        
        data_points
    }

    async fn store_data_point(channel_uuid: String, data_point: u16, storage: Arc<RwLock<Storage>>) {
        let mut storage = storage.write().await;
        storage.add_datapoint(channel_uuid, data_point);
        drop(storage);
    }

    async fn mark_device_as_disconnected(mac_address: &str, storage: Arc<RwLock<Storage>>) -> Result<(), Box<dyn Error>> {
        let mut storage = storage.write().await;
        storage.modify_devices(|devices| {
            for device in devices.iter_mut() {
                if device.mac == mac_address {
                    device.connected = false;
                    break;
                }
            }
        });
        Ok(())
    }
}
