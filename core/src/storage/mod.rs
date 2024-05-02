use super::*;

use std::collections::HashMap;
use ndarray::Array1;

mod ringbuffer;

use ringbuffer::SliceableRingBuffer;

pub struct Storage {
    devices: Vec<Device>,
    hist_size: usize,
    delegate_hist_size: usize,
    data: HashMap<String, SliceableRingBuffer<Option<u16>>>,
    delegate: Arc<dyn VVCoreDelegate>,
}

impl Storage {
    pub fn new(hist_size: usize, delegate_hist_size: usize, delegate: Arc<dyn VVCoreDelegate>) -> Self {
        if hist_size < delegate_hist_size {
            panic!("hist_size must be greater than or equal to delegate_hist_size");
        }
        
        Self {
            devices: Vec::new(),
            hist_size,
            delegate_hist_size,
            data: HashMap::new(),
            delegate,
        }
    }

    pub fn get_devices(&self) -> Vec<Device> {
        self.devices.clone()
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
                .insert(uuid.clone(), SliceableRingBuffer::new(self.hist_size, None));
        }

        let data = self.data.get_mut(&uuid).unwrap();
        data.write(Some(data_point));

        let delegate1 = self.delegate.clone();
        let vec = data.get_slice_with_len(self.delegate_hist_size).to_vec();
        let copy = vec.clone();
        
        let array = Array1::from_vec(vec.iter().filter(|x| x.is_some()).map(|x| x.unwrap()).collect());
        tokio::spawn(async move {
            delegate1.new_data(uuid, vec);
        });
    }
    
    pub fn get_data_for_all_channels(&self) -> HashMap<String, Vec<Option<u16>>> {
        self.data.iter().map(|(k, v)| (k.clone(), v.get_slice_with_len(self.delegate_hist_size).to_vec())).collect()
    } 
}
