use super::socket_read_buffer::SocketReadBuffer;

impl SocketReadBuffer {
    pub fn parse_pascal_string(&mut self) -> Option<String> {
        let byte = self.read_byte()?;

        let buffer = self.read_bytes(byte as usize)?;

        let result = std::str::from_utf8(buffer).unwrap();

        return Some(result.to_string());
    }

    pub fn parse_slice(&mut self) -> Option<Vec<u8>> {
        let data_size = self.parse_u32()?;

        let buffer = self.read_bytes(data_size as usize)?;

        return Some(buffer.to_vec());
    }
    pub fn parse_u32(&mut self) -> Option<u32> {
        let mut array = [0u8; 4];

        array.copy_from_slice(self.read_bytes(4)?);

        let result = u32::from_le_bytes(array);

        Some(result)
    }
}

pub trait SocketPacketWriter {
    fn push_pascal_string(&mut self, value: &str);

    fn push_u32(&mut self, value: u32);

    fn push_slice(&mut self, value: &[u8]);
}

impl SocketPacketWriter for Vec<u8> {
    fn push_pascal_string(&mut self, value: &str) {
        let buffer = value.as_bytes();

        let string_size = buffer.len() as u8;
        self.push(string_size);

        self.extend(buffer);
    }

    fn push_u32(&mut self, value: u32) {
        let arr = value.to_le_bytes();
        self.extend(&arr[..]);
    }

    fn push_slice(&mut self, value: &[u8]) {
        self.push_u32(value.len() as u32);
        self.extend(value);
    }
}
