use crate::ffi::{GlobalCallbacks, Sensor};

#[swift_bridge::bridge]
mod ffi {

    extern "Rust" {
        fn newVitalVisionCore() -> VitalVisionCore;

        type VitalVisionCore;

        fn start(&self);

        fn get_sensors(&self) -> Vec<Sensor>;
    }

    #[swift_bridge(swift_repr = "struct")]
    struct Sensor {
        id: u64,
        battery: u8,
        current_time_millis: u64,
        //data_channel_ids: Vec<u16>,
    }

    enum ChannelType {
        PPG,
        ECG,
    }

    #[swift_bridge(swift_repr = "struct")]
    struct Channel {
        id: u64,
        channel_type: ChannelType,
        values: Vec<u16>,
    }


    extern "Swift" {
        type GlobalCallbacks;
        #[swift_bridge(swift_name = "newSensorDiscovered")]
        fn new_sensor_discovered(&self, sensor: &Sensor);
        #[swift_bridge(swift_name = "newDataAvailable")]
        fn new_data_available(&self, sensor_id: u64);
        #[swift_bridge(swift_name = "batteryLow")]
        fn battery_low(&self, sensor_id: u64);
        #[swift_bridge(swift_name = "signalIssue")]
        fn signal_issue(&self, sensor_id: u64);

    }
}

struct VitalVisionCore {
    //sensors: Vec<Sensor>,
    //callbacks: GlobalCallbacks,
}

impl VitalVisionCore {
    fn new() -> VitalVisionCore {
        VitalVisionCore {
            //sensors: Vec::new(),
            //callbacks: GlobalCallbacks::new(),
        }
    }

    pub(crate) fn start(&self) {
        println!("Starting VitalVisionCore");
        /*
        for sensor in &self.sensors {
            self.callbacks.new_sensor_discovered(sensor);
        }
        */
    }

    pub(crate) fn get_sensors(&self) -> Vec<Sensor> {
        vec![Sensor {
            id: 1,
            battery: 100,
            current_time_millis: 0,
        }]
        //self.sensors.clone()
    }


}

pub(crate) fn newVitalVisionCore() -> VitalVisionCore {
VitalVisionCore::new()
}