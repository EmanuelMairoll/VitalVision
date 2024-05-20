pub mod plotters;//#[cfg(test)]
mod tests {
    use btleplug::api::{Central, Manager as _, Peripheral, ValueNotification};
    use btleplug::platform::Manager;
    use futures::future::join_all;
    use futures::StreamExt;
    use std::error::Error;
    use tokio::time::{sleep, Duration};
    use uuid::Uuid;

    pub async fn start_scanning() -> Result<(), Box<dyn Error>> {
        let manager = Manager::new().await?;

        // Get the first adapter
        let adapters = manager.adapters().await?;
        let central = adapters.into_iter().next().ok_or("No adapter found")?;

        // Start scanning for devices
        let filter = btleplug::api::ScanFilter { services: vec![] };
        central.start_scan(filter).await?;
        sleep(Duration::from_secs(2)).await; // Wait a bit for devices to be discovered

        // Fetch all peripherals
        let peripherals = central.peripherals().await?;

        // Fetch properties of all peripherals asynchronously and filter for "Dialog Peripheral"
        let peripheral_futures = peripherals
            .into_iter()
            .map(|p| async move { p.properties().await.ok().flatten().map(|props| (p, props)) });
        let found_peripherals = join_all(peripheral_futures).await;
        let dialog_peripheral = found_peripherals
            .into_iter()
            .filter_map(|x| x)
            .find(|(_, props)| {
                props
                    .local_name
                    .as_ref()
                    .map_or(false, |name| name.contains("Dialog Peripheral"))
            })
            .map(|(p, _)| p)
            .ok_or("Device not found")?;

        // Connect to the device
        if !dialog_peripheral.is_connected().await? {
            dialog_peripheral.connect().await?;
            println!("Connected to Dialog Peripheral");
        }

        // Discover services
        dialog_peripheral.discover_services().await?;
        let services = dialog_peripheral.services();

        // Find the specific service and characteristic
        let data_service_uuid = Uuid::parse_str("DCF31A27-A904-F3A3-AA4E-5AE42F1217B6")?;
        let data_characteristic_uuid = Uuid::parse_str("DCF31A27-A904-F4A3-A24E-5AE42F8617B6")?;

        if let Some(service) = services.iter().find(|&s| s.uuid == data_service_uuid) {
            if let Some(characteristic) = service
                .characteristics
                .iter()
                .find(|&c| c.uuid == data_characteristic_uuid)
            {
                // Subscribe to the characteristic
                dialog_peripheral.subscribe(characteristic).await?;
                println!("Subscribed to Characteristic");
            }
        }

        // Set up notification stream
        let mut notification_stream = dialog_peripheral.notifications().await.unwrap();
        while let Some(notification) = notification_stream.next().await {
            match notification {
                ValueNotification { uuid, value, .. } if uuid == data_characteristic_uuid => {
                    println!(
                        "Received notification on {:?}: {:?}",
                        uuid,
                        value
                            .iter()
                            .map(|byte| format!("{:02X}", byte))
                            .collect::<Vec<String>>()
                            .join(" ")
                    );
                }
                _ => {}
            }
        }

        Ok(())
    }

    //#[tokio::test]
    async fn test_start_scanning() {
        start_scanning().await.unwrap();
    }
}
