pub struct SocketReadBuffer {
    buffer: Vec<u8>,
    write_pos: usize,
    read_pos: usize,
}

impl SocketReadBuffer {

    pub fn to_string(&self)->{
        return format!("read:{}; write;{}", self.read_pos, self.write_pos);
    }
    
    pub fn new(capacity: usize) -> Self {
        let mut result = Self {
            buffer: Vec::with_capacity(capacity),
            write_pos: 0,
            read_pos: 0,
        };

        result.buffer.resize(capacity, 0u8);

        result
    }

    pub fn borrow_to_write(&mut self) -> Option<&mut [u8]> {
        if self.write_pos == self.buffer.len() {
            return None;
        }

        return Some(&mut self.buffer[self.write_pos..]);
    }

    pub fn commit_written_size(&mut self, size: usize) {
        self.write_pos += size;
    }

    pub fn reset_read_pos(&mut self) {
        self.read_pos = 0;
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        if self.read_pos == self.write_pos {
            return None;
        }

        let result = Some(self.buffer[self.read_pos]);

        self.read_pos += 1;

        return result;
    }

    pub fn read_bytes(&mut self, size: usize) -> Option<&[u8]> {
        if self.read_pos + size > self.write_pos {
            return None;
        }

        let result = Some(&self.buffer[self.read_pos..self.read_pos + size]);

        self.read_pos += size;

        return result;
    }

    pub fn confirm_read_package(&mut self) {
        let new_data_len = self.write_pos - self.read_pos;

        if new_data_len == 0 {
            self.write_pos = 0;
            self.read_pos = 0;
            return;
        }

        let mut mem: Vec<u8> = Vec::with_capacity(new_data_len);

        mem.extend(&self.buffer[self.read_pos..self.write_pos]);

        self.buffer[0..new_data_len].copy_from_slice(mem.as_slice());

        self.write_pos = self.write_pos - self.read_pos;

        self.read_pos = 0;
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn arrays_has_the_same_elements(src: &[u8], dest: &[u8]) {
        assert_eq!(src.len(), dest.len());

        for i in 0..src.len() {
            assert_eq!(src[i], dest[i]);
        }
    }

    #[test]
    fn test_write_and_commit() {
        let mut buffer = SocketReadBuffer::new(1024);

        let write = buffer.borrow_to_write().unwrap();

        write[0] = 1;
        write[1] = 2;
        write[2] = 3;

        buffer.commit_written_size(3);

        let write = buffer.borrow_to_write().unwrap();

        write[0] = 4;
        write[1] = 5;
        write[2] = 6;

        buffer.commit_written_size(3);

        let result_data = buffer.read_bytes(6).unwrap();

        arrays_has_the_same_elements(result_data, &[1u8, 2u8, 3u8, 4u8, 5u8, 6u8]);
    }

    #[test]
    fn test_read_more_then_we_get() {
        let mut buffer = SocketReadBuffer::new(1024);

        let write = buffer.borrow_to_write().unwrap();

        write[0] = 1;
        write[1] = 2;
        write[2] = 3;

        buffer.commit_written_size(3);

        let result_data = buffer.read_bytes(4);

        assert_eq!(true, result_data.is_none());
    }

    #[test]
    fn test_write_and_commit_some_data_read() {
        let mut buffer = SocketReadBuffer::new(1024);

        let write = buffer.borrow_to_write().unwrap();

        write[0] = 1;
        write[1] = 2;
        write[2] = 3;

        buffer.commit_written_size(3);

        let result_data = buffer.read_bytes(2).unwrap();
        arrays_has_the_same_elements(result_data, &[1u8, 2u8]);

        buffer.confirm_read_package();

        let write = buffer.borrow_to_write().unwrap();

        write[0] = 4;
        write[1] = 5;
        write[2] = 6;
        buffer.commit_written_size(3);

        let result_data = buffer.read_bytes(4).unwrap();
        arrays_has_the_same_elements(result_data, &[3u8, 4u8, 5u8, 6u8]);
    }
}
