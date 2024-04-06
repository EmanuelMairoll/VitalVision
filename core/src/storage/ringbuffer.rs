pub struct SliceableRingBuffer<T: Clone> {
    buffer: Vec<T>,
    write_position: usize,
    capacity: usize,
}

impl<T: Clone> SliceableRingBuffer<T> {
    // Initialize a new SliceableRingBuffer with the specified capacity.
    pub fn new(capacity: usize, default_value: T) -> Self {
        SliceableRingBuffer {
            buffer: vec![default_value; capacity * 2],
            write_position: 0,
            capacity,
        }
    }

    // Write data into the buffer, duplicating the write to handle wrapping.
    pub fn write(&mut self, data: T) {
        let adjusted_position = self.write_position % self.capacity;
        self.buffer[adjusted_position] = data.clone();
        self.buffer[adjusted_position + self.capacity] = data;

        self.write_position = (self.write_position + 1) % self.capacity;
    }

    pub fn get_slice(&self) -> &[T] {
        self.get_slice_with_len(self.capacity)
    }
    
    // Returns a slice of the last `self.capacity` elements written to the buffer.
    pub fn get_slice_with_len(&self, len: usize) -> &[T] {
        let start = (self.write_position + (self.capacity - len)) % self.capacity;
        &self.buffer[start..start + len]
    }
}

#[cfg(test)]
mod tests {
    use super::SliceableRingBuffer;

    #[test]
    fn it_initializes_correctly() {
        let rb = SliceableRingBuffer::new(5, 0);
        assert_eq!(rb.get_slice(), &[0, 0, 0, 0, 0]);
    }

    #[test]
    fn writing_and_reading() {
        let mut rb = SliceableRingBuffer::new(5, 0);
        for i in 1..=5 {
            rb.write(i);
        }
        assert_eq!(rb.get_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn overwriting_elements() {
        let mut rb = SliceableRingBuffer::new(5, 0);
        for i in 1..=13 {
            rb.write(i);
        }
        // With capacity 5, we expect to have the last 5 elements written
        assert_eq!(rb.get_slice(), &[9, 10, 11, 12, 13]);
    }
    
    #[test]
    fn get_slice_with_len() {
        let mut rb = SliceableRingBuffer::new(5, 0);
        for i in 1..=13 {
            rb.write(i);
        }
        // With capacity 5, we expect to have the last 5 elements written
        assert_eq!(rb.get_slice_with_len(3), &[11, 12, 13]);
    }
}
