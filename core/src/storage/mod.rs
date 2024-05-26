use super::*;

use std::collections::HashMap;

mod ringbuffer;

use ringbuffer::SliceableRingBuffer;

pub type DeviceStorage = HashMap<String, Device>;

pub struct DataStorage {
    hist_size: usize,
    ret_a_len: usize,
    ret_b_len: usize,
    data: HashMap<String, ChannelData>,
}

pub struct ChannelData {
    pub data: SliceableRingBuffer<Option<i32>>,
    pub data_type: ChannelType,
    pub datapoint_counter: u32,
}

impl DataStorage {
    pub fn new(ret_a_len: usize, ret_b_len: usize) -> Self {
        let max = ret_a_len.max(ret_b_len);
        
        Self {
            hist_size: max,
            ret_a_len,
            ret_b_len,
            data: HashMap::new(),
        }
    }
    
    pub fn add_channel(&mut self, uuid: String, c_type: ChannelType) {
        self.data
            .insert(uuid.clone(), ChannelData {
                data: SliceableRingBuffer::new(self.hist_size, None),
                data_type: c_type,
                datapoint_counter: 0,
            });
    }
    
    pub fn remove_channel(&mut self, uuid: String) {
        self.data.remove(&uuid);
    }
    
    pub fn add_datapoint(&mut self, uuid: String, data_points: Vec<i32>) -> Option<(&[Option<i32>], &[Option<i32>], ChannelType, u32)> {
        if !self.data.contains_key(&uuid) {
            return None;
        }
        
        let channel_data = self.data.get_mut(&uuid).unwrap();
        for data_point in data_points.iter() {
            channel_data.data.write(Some(*data_point));
            channel_data.datapoint_counter += 1;
        }
        
        let ret_a = channel_data.data.get_slice_with_len(self.ret_a_len);
        let ret_b = channel_data.data.get_slice_with_len(self.ret_b_len);
        return Some((ret_a, ret_b, channel_data.data_type.clone(), channel_data.datapoint_counter));
    }
    
    pub fn reset_counter(&mut self, uuid: String) {
        if let Some(channel_data) = self.data.get_mut(&uuid) {
            channel_data.datapoint_counter = 0;
        }
    }
}
