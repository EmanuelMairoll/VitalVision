use crate::storage::Storage;
use crate::*;
use async_std::sync::{Arc, RwLock};
use btleplug::api::Peripheral;
use btleplug::api::{
    bleuuid::BleUuid, Central, CentralEvent, Manager as _, ScanFilter, ValueNotification,
};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;

use uuid::Uuid;

#[derive(Clone)]
pub struct Ble {
    storage: Arc<RwLock<Storage>>,
}

impl Ble {
    pub fn new(storage: Arc<RwLock<Storage>>) -> Self {
        Self { storage }
    }

    pub async fn run_loop(&self, service_filter_uuid: String) -> Result<(), Box<dyn Error>> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().next().ok_or("No adapter found")?;
        let mut events = central.events().await?;
        let filter_uuid = Uuid::parse_str(&service_filter_uuid).unwrap();
        let characteristic_uuid = Uuid::parse_str("dcf31a27-a904-f4a3-a24e-5ae42f8617b6").unwrap();

        central.start_scan(ScanFilter { services: vec![] }).await?;
        println!("Scanning for devices...");

        while let Some(event) = events.next().await {
            match event {
                CentralEvent::DeviceDiscovered(id) => {
                    let device = central.peripheral(&id).await?;
                    let properties = device.properties().await?.unwrap();

                    println!("Found device: {:?}", id);
                    if let Some(l) = properties.local_name.clone() {
                        println!("Local name: {:?}", l);
                    }

                    // Connect to device if it advertises the specific service we're interested in
                    if properties.local_name == Some("Dialog Peripheral".to_string()) {
                        println!("Found device: {:?}", id);

                        if !device.is_connected().await? {
                            device.connect().await?;
                            println!("Connected to device: {:?}", id);

                            device.discover_services().await?;
                            let services = device.services();

                            // print out all services
                            for service in services.iter() {
                                println!("Service: {:?}", service.uuid);
                                if service.uuid == filter_uuid {
                                    for characteristic in service.characteristics.iter() {
                                        println!(
                                            "Subscribing to characteristic: {:?}",
                                            characteristic.uuid
                                        );
                                        device.subscribe(&characteristic).await?;
                                    }
                                }
                            }

                            {
                                let storage_device = Device {
                                    uuid: properties.address.to_string(),
                                    name: properties.local_name.unwrap_or("Unknown".to_string()),
                                    battery: 0,
                                    connected: true,
                                    status: crate::DeviceStatus::Ok,
                                    channels: vec![
                                        Channel {
                                            uuid: "uuid1".to_string(),
                                            name: "PPG1".to_string(),
                                            channel_type: ChannelType::PPG,
                                            status: ChannelStatus::Ok,
                                        },
                                        Channel {
                                            uuid: "uuid2".to_string(),
                                            name: "PPG2".to_string(),
                                            channel_type: ChannelType::PPG,
                                            status: ChannelStatus::Ok,
                                        },
                                    ],
                                };

                                let mut storage = self.storage.write().await;
                                storage.modify_devices(|devices| devices.push(storage_device));
                                drop(storage);
                            }
                            // Set up notification stream
                            let mut notification_stream = device.notifications().await.unwrap();
                            println!("Waiting for notifications...");
                            while let Some(notification) = notification_stream.next().await {
                                println!("Received notification: {:?}", notification);
                                match notification {
                                    ValueNotification { uuid, value, .. }
                                        if uuid == characteristic_uuid =>
                                    {
                                        let data1 = ((value[1] as u16) << 8) + 0; //value[1] as u16;
                                        let data2 = ((value[5] as u16) << 8) + 0; //value[4] as u16;
                                        let mut storage = self.storage.write().await;
                                        storage.add_datapoint("uuid1".to_string(), data1);
                                        storage.add_datapoint("uuid2".to_string(), data2);
                                        drop(storage);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}
