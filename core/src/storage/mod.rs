use super::*;

use std::collections::HashMap;
use tokio::sync::watch::{self, Receiver, Sender};

mod ringbuffer;

use ringbuffer::SliceableRingBuffer;

pub struct Storage {
    devices: Vec<Device>,
    hist_size: usize,
    data: HashMap<String, SliceableRingBuffer<u16>>,
    delegate: Arc<dyn VVCoreDelegate>,
}

impl Storage {
    pub fn new(hist_size: usize, delegate: Arc<dyn VVCoreDelegate>) -> Self {
        Self {
            devices: Vec::new(),
            hist_size,
            data: HashMap::new(),
            delegate,
        }
    }

    pub fn modify_devices(&mut self, f: impl FnOnce(&mut Vec<Device>)) {
        f(&mut self.devices);
        println!("sending Devices: {:?}", self.devices);

        let delegate1 = self.delegate.clone();
        let devices = self.devices.clone();
        tokio::spawn(async move {
            delegate1.devices_changed(devices.clone());
        });
    }

    pub fn add_datapoint(&mut self, uuid: String, data_point: u16) {
        if !self.data.contains_key(&uuid) {
            self.data
                .insert(uuid.clone(), SliceableRingBuffer::new(self.hist_size, 0));
        }

        let data = self.data.get_mut(&uuid).unwrap();
        data.write(data_point);

        let delegate1 = self.delegate.clone();
        let vec = data.get_slice().to_vec();
        tokio::spawn(async move {
            delegate1.new_data(uuid, vec);
        });
    }
}
