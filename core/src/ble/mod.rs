use std::collections::HashMap;
use super::*;
use ble_date_converter::*;
use btleplug::api::{
    Central, CentralEvent, Manager as _, Peripheral,
    ScanFilter, ValueNotification, WriteType,
};
use btleplug::platform::{Manager};
use chrono::Utc;
use futures::stream::{StreamExt, select};
use std::error::Error;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex};
use tokio_stream::Stream;
use uuid::Uuid;
use tokio_stream::wrappers::ReceiverStream;

mod ble_date_converter;
pub mod mock;

pub struct Ble {
    max_initial_rtt_ms: u32,
    event_publisher: Sender<ExternalBleEvent>,
    tx: Arc<Mutex<Option<Sender<InternalBleEvent>>>>,
}

#[derive(Clone, Debug)]
enum InternalBleEvent {
    CentralEvent(CentralEvent),
    SyncTime,
}

#[derive(Clone, Debug)]
pub enum ExternalBleEvent {
    DeviceConnected(Device),
    DeviceDisconnected(String),
    BatteryLevelChanged(String, u8),
    DriftChanged(String, i64),
    DataReceived(HashMap<String, Vec<i32>>),
}

type DatapointDecoder = Box<dyn Fn(Vec<u8>) -> HashMap<String, Vec<i32>> + Send + Sync>;

impl Ble {
    pub fn new(event_publisher: Sender<ExternalBleEvent>, max_initial_rtt_ms: u32) -> Self {
        Self {
            max_initial_rtt_ms,
            event_publisher,
            tx: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn run_loop(&self) -> Result<(), Box<dyn Error>> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().next().ok_or("No adapter found")?;

        central
            .start_scan(ScanFilter { services: vec!["DCF31A27-A904-F3A3-AA4E-5AE42F1217B6".parse().unwrap()] })
            .await?;
        println!("Scanning for devices...");

        let event_stream = central.events().await?.map(InternalBleEvent::CentralEvent);

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let notification_stream = ReceiverStream::new(rx);

        let mut tx_lock = self.tx.lock().await;
        *tx_lock = Some(tx);
        drop(tx_lock);

        let mut combined_stream = select(event_stream, notification_stream);

        let event_publisher = self.event_publisher.clone();
        let max_initial_rtt_ms = self.max_initial_rtt_ms;
        while let Some(event) = combined_stream.next().await {
            let event_publisher = event_publisher.clone();
            match event {
                InternalBleEvent::CentralEvent(CentralEvent::DeviceDiscovered(id)) => {
                    println!("Device discovered: {:?}", id);
                    let device = central.peripheral(&id).await.unwrap();
                    tokio::spawn(async move {
                        Ble::handle_discovered_device(&device, event_publisher, max_initial_rtt_ms)
                            .await.unwrap();
                        println!("Device handled");
                    });
                }
                InternalBleEvent::CentralEvent(CentralEvent::DeviceDisconnected(id)) => {
                    println!("Device disconnected: {:?}", id);
                    event_publisher.send(ExternalBleEvent::DeviceDisconnected(id.to_string())).await.unwrap();
                }
                InternalBleEvent::SyncTime => {
                    println!("Syncing Time");

                    for device in central.peripherals().await.unwrap() {
                        if !device.is_connected().await.unwrap() {
                            continue;
                        }

                        let event_publisher = event_publisher.clone();
                        tokio::spawn(async move {
                            let id = device.id().to_string();
                            println!("Syncing time for device: {:?}", id);
                            let rtt = Ble::sync_time_for_device(&device, max_initial_rtt_ms).await.unwrap();

                            event_publisher.send(ExternalBleEvent::DriftChanged(id, rtt)).await.unwrap();
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub async fn sync_time(&self) {
        let tx = self.tx.lock().await;
        if let Some(tx) = &*tx {
            tx.send(InternalBleEvent::SyncTime).await.unwrap();
        }
    }

    async fn handle_value_notification(device_id: String, decoder: Arc<DatapointDecoder>, uuid: Uuid, value: Vec<u8>, event_publisher: Sender<ExternalBleEvent>) {
        match uuid {
            uuid if uuid == Uuid::parse_str("00002a19-0000-1000-8000-00805f9b34fb").unwrap() => {
                if let Some(&battery_level) = value.first() {
                    println!("Battery level update: {}%", battery_level);
                    event_publisher.send(ExternalBleEvent::BatteryLevelChanged(device_id, battery_level)).await.unwrap();
                }
            }
            uuid if uuid == Uuid::parse_str("dcf31a27-a904-f4a3-a24e-5ae42f8617b6").unwrap() => {
                //println!("Received data notification, HEX DUMP: {:?}", value.iter().map(|x| format!("{:02x} ", x)).collect::<String>());
                let decoded = decoder(value);
                
                event_publisher.send(ExternalBleEvent::DataReceived(decoded)).await.unwrap_or_else(|e| {
                    println!("Error sending data to event publisher: {:?}", e.to_string());
                    // if is SendError{}, print details
                    
                });
            }
            _ => println!(
                "Received notification from an unknown characteristic: {:?}",
                uuid
            ),
        }
    }

    async fn handle_discovered_device(
        device: &impl Peripheral,
        event_publisher: Sender<ExternalBleEvent>,
        max_initial_rtt_ms: u32,
    ) -> Result<(), Box<dyn Error>> {
        device.connect().await?;
        //println!("Connected to device: {:?}", device.id());

        device.discover_services().await?;

        let id = device.id().to_string();

        let drift = Ble::sync_time_for_device(device, max_initial_rtt_ms).await?;

        let (serial, model, battery, first_data) =
            Ble::get_device_information_and_subscribe(device).await?;

        // TODO: move channel mapping to a separate characteristic
        // for now, we hardcode the channel mapping and differentiate ECG from non-ECG devices by
        // reading the initial data value, whose ECG data is constant 0 for non-ECG devices
        let (channels, datapoint_decoder) = Ble::temp_create_channels(id.clone(), &first_data).unwrap();

        let device_struct = Device {
            id: id.clone(),
            serial,
            name: model,
            battery,
            drift_us: drift,
            connected: true,
            channels,
        };
        
        let datapoint_decoder = Arc::new(datapoint_decoder);

        event_publisher.send(ExternalBleEvent::DeviceConnected(device_struct)).await?;

        // handle notifications, blocking the task until device disconnects
        let mut notification_stream: Pin<Box<dyn Stream<Item=ValueNotification> + Send>> = device.notifications().await?;

        while let Some(notification) = notification_stream.next().await {
            let ValueNotification { uuid, value, .. } = notification;
            Self::handle_value_notification(id.clone(), datapoint_decoder.clone(), uuid, value, event_publisher.clone()).await;
        }
        
        //println!("Setup notification handling");

        Ok(())
    }

    async fn sync_time_for_device(device: &impl Peripheral, max_initial_rtt_ms: u32) -> Result<i64, Box<dyn Error>> {
        let time_service_uuid = Uuid::parse_str("00001806-0000-1000-8000-00805f9b34fb")?;
        let time_characteristic_uuid = Uuid::parse_str("00002a2d-0000-1000-8000-00805f9b34fb")?;
        
        //return Ok(42);

        if !device.is_connected().await? {
            return Err("Device disconnected".into());
        }

        for service in device.services() {
            if service.uuid == time_service_uuid {
                for characteristic in &service.characteristics {
                    if characteristic.uuid == time_characteristic_uuid {
                        let mut rtt = -1;
                        
                        // try at max 5 times
                        for _ in 0..5 {
                            let time_to_set = Utc::now();
                            let data_to_set = time_to_ble_data(time_to_set);
                            //println!("Writing wo response");
                            //println!("HEX: {:?}", data_to_set.iter().map(|x| format!("{:02X}", x)).collect::<String>());
                            device
                                .write(characteristic, &data_to_set, WriteType::WithoutResponse)
                                .await?;
                            //print!("done Writing");

                            let data_read = device.read(characteristic).await?;
                            let time_to_compare = Utc::now();

                            let time_read = ble_data_to_time(&data_read)?;

                            rtt = time_to_compare.timestamp_micros() - time_read.timestamp_micros();

                            if rtt.abs() < (max_initial_rtt_ms * 1000) as i64 {
                                return Ok(rtt);
                            }
                        }

                        return Ok(rtt);
                    }
                }
            }
        }

        Err("Time service or characteristic not found".into())
    }

    async fn get_device_information_and_subscribe(
        device: &impl Peripheral,
    ) -> Result<(u16, String, u8, Vec<u8>), Box<dyn Error>> {
        let service_uuid_device_info = Uuid::parse_str("0000180a-0000-1000-8000-00805f9b34fb")?;
        let characteristic_uuid_serial = Uuid::parse_str("00002a25-0000-1000-8000-00805f9b34fb")?;
        let mut serial: u16 = 0;
        let characteristic_uuid_model = Uuid::parse_str("00002a24-0000-1000-8000-00805f9b34fb")?;
        let mut model = "".to_string();

        let service_uuid_battery = Uuid::parse_str("0000180f-0000-1000-8000-00805f9b34fb")?; // todo: replace with actual service uuid
        let characteristic_uuid_battery = Uuid::parse_str("00002a19-0000-1000-8000-00805f9b34fb")?;
        let mut battery: u8 = 0;

        // TODO: move channel mapping to a separate characteristic
        // for now, we hardcode the channel mapping and differentiate ECG from non-ECG devices by
        // reading the initial data value, whose ECG data is constant 0 for non-ECG devices
        let service_uuid_data = Uuid::parse_str("dcf31a27-a904-f3a3-aa4e-5ae42f1217b6")?;
        let characteristic_uuid_data = Uuid::parse_str("dcf31a27-a904-f4a3-a24e-5ae42f8617b6")?;
        let mut first_data = vec![];

        for service in device.services() {
            if service.uuid == service_uuid_device_info {
                for characteristic in &service.characteristics {
                    if characteristic.uuid == characteristic_uuid_serial {
                        let serial_data = device.read(characteristic).await?;
                        let serial_str = String::from_utf8(serial_data);
                        serial = u16::from_str_radix(&serial_str.unwrap(), 10).unwrap();

                        println!("Serial: {:?}", serial);
                    }
                    if characteristic.uuid == characteristic_uuid_model {
                        let model_data = device.read(characteristic).await?;
                        model = String::from_utf8(model_data)?;
                        println!("Model: {:?}", model);
                    }
                }
            }

            if service.uuid == service_uuid_battery {
                for characteristic in &service.characteristics {
                    if characteristic.uuid == characteristic_uuid_battery {
                        let battery_data = device.read(characteristic).await?;
                        battery = battery_data[0];
                        println!("Battery: {:?}", battery);

                        device.subscribe(characteristic).await?;
                    }
                }
            }

            if service.uuid == service_uuid_data {
                for characteristic in &service.characteristics {
                    if characteristic.uuid == characteristic_uuid_data {
                        // HACK: first_data should REALLY be readable, dummy value for now
                        first_data = if serial == 72 {vec![1; 25]} else {vec![0; 25]};
                        
                        //first_data = device.read(characteristic).await?;
                        println!("First data: {:?}", first_data);
                        
                        device.subscribe(characteristic).await?;
                    }
                }
            }
        }
        Ok((serial, model, battery, first_data))
    }

    fn temp_create_channels(id: String, first_data: &Vec<u8>) -> Result<(Vec<Channel>, DatapointDecoder), Box<dyn Error>> {
        if first_data.len() != 25 {
            println!("First data length: {}", first_data.len());
            return Err("Invalid first data length".into());
        }

        // we differentiate ECG from non-ECG devices by the initial ECG data value
        let has_ecg = first_data[1] == 0 && first_data[2] == 0 && first_data[9] == 0 && first_data[10] == 0 && first_data[17] == 0 && first_data[18] == 0;

        return if has_ecg {
            let channels = vec![
                Channel {
                    id: format!("{}-0", id),
                    name: "ECG".to_string(),
                    channel_type: ChannelType::ECG,
                    signal_quality: None,
                },
                Channel {
                    id: format!("{}-1", id),
                    name: "PPG green".to_string(),
                    channel_type: ChannelType::PPG,
                    signal_quality: None,
                },
                Channel {
                    id: format!("{}-2", id),
                    name: "PPG red".to_string(),
                    channel_type: ChannelType::PPG,
                    signal_quality: None,
                },
                Channel {
                    id: format!("{}-3", id),
                    name: "PPG IR".to_string(),
                    channel_type: ChannelType::PPG,
                    signal_quality: None,
                },
            ];

            let decoder = Box::new(move |value: Vec<u8>| {
                let mut data_points = HashMap::new();

                let mut ecg = vec![];
                ecg.push(i16::from_le_bytes([value[1], value[2]]) as i32);
                ecg.push(i16::from_le_bytes([value[9], value[10]]) as i32);
                ecg.push(i16::from_le_bytes([value[17], value[18]]) as i32);
                data_points.insert(format!("{}-0", id), ecg);

                let mut ppg_green = vec![];
                ppg_green.push(u16::from_le_bytes([value[3], value[4]]) as i32);
                ppg_green.push(u16::from_le_bytes([value[11], value[12]]) as i32);
                ppg_green.push(u16::from_le_bytes([value[19], value[20]]) as i32);
                data_points.insert(format!("{}-1", id), ppg_green);

                let mut ppg_red = vec![];
                ppg_red.push(u16::from_le_bytes([value[5], value[6]]) as i32);
                ppg_red.push(u16::from_le_bytes([value[13], value[14]]) as i32);
                ppg_red.push(u16::from_le_bytes([value[21], value[22]]) as i32);
                data_points.insert(format!("{}-2", id), ppg_red);

                let mut ppg_ir = vec![];
                ppg_ir.push(u16::from_le_bytes([value[7], value[8]]) as i32);
                ppg_ir.push(u16::from_le_bytes([value[15], value[16]]) as i32);
                ppg_ir.push(u16::from_le_bytes([value[23], value[24]]) as i32);
                data_points.insert(format!("{}-3", id), ppg_ir);

                data_points
            });

            Ok((channels, decoder))
        } else {
            let channels = vec![
                Channel {
                    id: format!("{}-0", id),
                    name: "PPG green".to_string(),
                    channel_type: ChannelType::PPG,
                    signal_quality: None,
                },
                Channel {
                    id: format!("{}-1", id),
                    name: "PPG red".to_string(),
                    channel_type: ChannelType::PPG,
                    signal_quality: None,
                },
                Channel {
                    id: format!("{}-2", id),
                    name: "PPG IR".to_string(),
                    channel_type: ChannelType::PPG,
                    signal_quality: None,
                },
            ];

            let decoder = Box::new(move |value: Vec<u8>| {
                let mut data_points = HashMap::new();
                
                let mut ppg_grn = vec![];
                ppg_grn.push(u16::from_le_bytes([value[3], value[4]]) as i32);
                ppg_grn.push(u16::from_le_bytes([value[11], value[12]]) as i32);
                ppg_grn.push(u16::from_le_bytes([value[19], value[20]]) as i32);
                data_points.insert(format!("{}-0", id), ppg_grn);

                let mut ppg_red = vec![];
                ppg_red.push(u16::from_le_bytes([value[5], value[6]]) as i32);
                ppg_red.push(u16::from_le_bytes([value[13], value[14]]) as i32);
                ppg_red.push(u16::from_le_bytes([value[21], value[22]]) as i32);
                data_points.insert(format!("{}-1", id), ppg_red);

                let mut ppg_ir = vec![];
                ppg_ir.push(u16::from_le_bytes([value[7], value[8]]) as i32);
                ppg_ir.push(u16::from_le_bytes([value[15], value[16]]) as i32);
                ppg_ir.push(u16::from_le_bytes([value[23], value[24]]) as i32);
                data_points.insert(format!("{}-2", id), ppg_ir);

                data_points
            });

            Ok((channels, decoder))
        };
    }
}
