use super::socket_read_buffer::SocketReadBuffer;
use super::type_parsers::SocketPacketWriter;

#[derive(Debug)]
pub enum DataReaderContract {
    Ping,
    Pong,
    Greeting {
        name: String,
    },
    Subscribe {
        table_name: String,
    },
    InitTable {
        table_name: String,
        data: Vec<u8>,
    },
    InitPartition {
        table_name: String,
        partition_key: String,
        data: Vec<u8>,
    },
    UpdateRows {
        table_name: String,
        data: Vec<u8>,
    },
    DeleteRows {
        table_name: String,
        rows: Vec<(String, String)>,
    },
}

pub const PACKET_PING: u8 = 0;
pub const PACKET_PONG: u8 = 1;
pub const PACKET_GREETING: u8 = 2;
pub const PACKET_SUBSCRIBE: u8 = 3;
pub const PACKET_INIT_TABLE: u8 = 4;
pub const PACKET_INIT_PARTITION: u8 = 5;
pub const PACKET_UPDATE_ROWS: u8 = 6;
pub const PACKET_DELETE_ROWS: u8 = 7;

impl DataReaderContract {
    pub fn get_table_name(&self) -> Option<&str> {
        match self {
            DataReaderContract::Ping => None,
            DataReaderContract::Pong => None,
            DataReaderContract::Greeting { name: _ } => None,
            DataReaderContract::Subscribe { table_name } => Some(table_name),
            DataReaderContract::InitTable {
                table_name,
                data: _,
            } => Some(table_name),
            DataReaderContract::InitPartition {
                table_name,
                partition_key: _,
                data: _,
            } => Some(table_name),
            DataReaderContract::UpdateRows {
                table_name,
                data: _,
            } => Some(table_name),
            DataReaderContract::DeleteRows {
                table_name,
                rows: _,
            } => Some(table_name),
        }
    }

    pub fn deserialize(
        payload: &mut SocketReadBuffer,
    ) -> Result<Option<DataReaderContract>, String> {
        let packet = payload.read_byte();

        if packet.is_none() {
            return Ok(None);
        }

        let packet = packet.unwrap();

        return match packet {
            PACKET_PING => Ok(Some(DataReaderContract::Ping)),
            PACKET_PONG => Ok(Some(DataReaderContract::Pong)),

            PACKET_GREETING => Ok(parse_greeting_packet(payload)),
            PACKET_SUBSCRIBE => Ok(parse_subscribe_packet(payload)),
            PACKET_INIT_TABLE => Ok(parse_init_table_contract(payload)),
            PACKET_INIT_PARTITION => Ok(parse_init_partition_contract(payload)),
            PACKET_UPDATE_ROWS => Ok(parse_update_rows_contract(payload)),
            PACKET_DELETE_ROWS => Ok(parse_delete_rows_contract(payload)),

            _ => Err(format!("Unknown command type {}", packet)),
        };
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        match &self {
            &DataReaderContract::Ping => result.push(PACKET_PING),
            &DataReaderContract::Pong => result.push(PACKET_PONG),
            &DataReaderContract::Greeting { name } => {
                result.push(PACKET_GREETING);
                result.push_pascal_string(name.as_str());
            }
            &DataReaderContract::Subscribe { table_name } => {
                result.push(PACKET_SUBSCRIBE);
                result.push_pascal_string(table_name.as_str());
            }

            &DataReaderContract::InitTable { table_name, data } => {
                result.push(PACKET_INIT_TABLE);
                result.push_pascal_string(table_name.as_str());
                result.push_slice(data.as_slice());
            }

            &DataReaderContract::UpdateRows { table_name, data } => {
                result.push(PACKET_UPDATE_ROWS);
                result.push_pascal_string(table_name.as_str());
                result.push_slice(data.as_slice());
            }

            &DataReaderContract::InitPartition {
                table_name,
                partition_key,
                data,
            } => {
                result.push(PACKET_INIT_PARTITION);
                result.push_pascal_string(table_name.as_str());
                result.push_pascal_string(partition_key.as_str());
                result.push_slice(data.as_slice());
            }
            &DataReaderContract::DeleteRows { table_name, rows } => {
                result.push(PACKET_DELETE_ROWS);
                result.push_pascal_string(table_name.as_str());

                result.push_u32(rows.len() as u32);

                for (partition_key, row_key) in rows {
                    result.push_pascal_string(partition_key.as_str());
                    result.push_pascal_string(row_key.as_str());
                }
            }
        }

        return result;
    }
}

fn parse_greeting_packet(payload: &mut SocketReadBuffer) -> Option<DataReaderContract> {
    let name = payload.parse_pascal_string()?;

    return Some(DataReaderContract::Greeting { name });
}

fn parse_subscribe_packet(payload: &mut SocketReadBuffer) -> Option<DataReaderContract> {
    let table_name = payload.parse_pascal_string()?;

    return Some(DataReaderContract::Subscribe { table_name });
}

fn parse_init_table_contract(payload: &mut SocketReadBuffer) -> Option<DataReaderContract> {
    let table_name = payload.parse_pascal_string()?;

    let data = payload.parse_slice()?;

    return Some(DataReaderContract::InitTable { table_name, data });
}

fn parse_init_partition_contract(payload: &mut SocketReadBuffer) -> Option<DataReaderContract> {
    let table_name = payload.parse_pascal_string()?;
    let partition_key = payload.parse_pascal_string()?;

    let data = payload.parse_slice()?;

    return Some(DataReaderContract::InitPartition {
        table_name,
        partition_key,
        data,
    });
}

fn parse_update_rows_contract(payload: &mut SocketReadBuffer) -> Option<DataReaderContract> {
    let table_name = payload.parse_pascal_string()?;

    let data = payload.parse_slice()?;

    return Some(DataReaderContract::UpdateRows { table_name, data });
}

fn parse_delete_rows_contract(payload: &mut SocketReadBuffer) -> Option<DataReaderContract> {
    let table_name = payload.parse_pascal_string()?;

    let len = payload.parse_u32()?;

    let mut rows = Vec::new();

    for _ in 0..len {
        let partition_key = payload.parse_pascal_string()?;
        let row_key = payload.parse_pascal_string()?;
        rows.push((partition_key, row_key));
    }

    return Some(DataReaderContract::DeleteRows { table_name, rows });
}

#[cfg(test)]
mod tests {

    use super::*;

    fn serialize(src: &DataReaderContract) -> SocketReadBuffer {
        let mut buffer = SocketReadBuffer::new(1024);

        let bytes = src.serialize();

        let write_buffer = buffer.borrow_to_write().unwrap();

        write_buffer[0..bytes.len()].copy_from_slice(&bytes[..]);

        buffer.commit_written_size(bytes.len());

        buffer
    }

    #[test]
    fn test_ping_contract() {
        let src = DataReaderContract::Ping;

        let mut buffer = serialize(&src);

        let dest = DataReaderContract::deserialize(&mut buffer)
            .unwrap()
            .unwrap();

        if let DataReaderContract::Ping = dest {
        } else {
            panic!("Dest contract is wrong")
        }
    }

    #[test]
    fn test_pong_contract() {
        let src = DataReaderContract::Pong;

        let mut buffer = serialize(&src);

        let dest = DataReaderContract::deserialize(&mut buffer)
            .unwrap()
            .unwrap();

        if let DataReaderContract::Pong = dest {
        } else {
            panic!("Dest contract is wrong")
        }
    }

    #[test]
    fn test_greeting_contract() {
        let src = DataReaderContract::Greeting {
            name: "test".to_string(),
        };

        let mut buffer = serialize(&src);

        let dest = DataReaderContract::deserialize(&mut buffer)
            .unwrap()
            .unwrap();

        if let DataReaderContract::Greeting { name } = dest {
            assert_eq!("test", name);
        } else {
            panic!("Dest contract is wrong")
        }
    }

    #[test]
    fn test_init_subscribe_contract() {
        let src = DataReaderContract::Subscribe {
            table_name: "test".to_string(),
        };

        let mut buffer = serialize(&src);

        let dest = DataReaderContract::deserialize(&mut buffer)
            .unwrap()
            .unwrap();

        if let DataReaderContract::Subscribe { table_name } = dest {
            assert_eq!("test", table_name);
        } else {
            panic!("Dest contract is wrong")
        }
    }

    #[test]
    fn test_init_init_table_contract() {
        let src = DataReaderContract::InitTable {
            table_name: "test".to_string(),
            data: vec![0u8, 1u8, 2u8],
        };

        let mut buffer = serialize(&src);

        let dest = DataReaderContract::deserialize(&mut buffer)
            .unwrap()
            .unwrap();

        if let DataReaderContract::InitTable { table_name, data } = dest {
            assert_eq!("test", table_name);
            assert_eq!(3, data.len());
            assert_eq!(0, data[0]);
            assert_eq!(1, data[1]);
            assert_eq!(2, data[2]);
        } else {
            panic!("Dest contract is wrong {:?}", &dest);
        }
    }

    #[test]
    fn test_init_init_partition_contract() {
        let src = DataReaderContract::InitPartition {
            table_name: "test".to_string(),
            partition_key: "test1".to_string(),
            data: vec![0u8, 1u8, 2u8],
        };

        let mut buffer = serialize(&src);

        let dest = DataReaderContract::deserialize(&mut buffer)
            .unwrap()
            .unwrap();

        if let DataReaderContract::InitPartition {
            table_name,
            partition_key,
            data,
        } = dest
        {
            assert_eq!("test", table_name);
            assert_eq!("test1", partition_key);
            assert_eq!(3, data.len());
            assert_eq!(0, data[0]);
            assert_eq!(1, data[1]);
            assert_eq!(2, data[2]);
        } else {
            panic!("Dest contract is wrong {:?}", &dest);
        }
    }

    #[test]
    fn test_init_update_rows_contract() {
        let src = DataReaderContract::UpdateRows {
            table_name: "test".to_string(),
            data: vec![0u8, 1u8, 2u8],
        };

        let mut buffer = serialize(&src);

        let dest = DataReaderContract::deserialize(&mut buffer)
            .unwrap()
            .unwrap();

        if let DataReaderContract::UpdateRows { table_name, data } = dest {
            assert_eq!("test", table_name);
            assert_eq!(3, data.len());
            assert_eq!(0, data[0]);
            assert_eq!(1, data[1]);
            assert_eq!(2, data[2]);
        } else {
            panic!("Dest contract is wrong {:?}", &dest);
        }
    }
    #[test]
    fn test_init_delete_rows_contract() {
        let src = DataReaderContract::DeleteRows {
            table_name: "test".to_string(),
            rows: vec![
                ("pk1".to_string(), "rk1".to_string()),
                ("pk2".to_string(), "rk2".to_string()),
            ],
        };

        let mut buffer = serialize(&src);

        let dest = DataReaderContract::deserialize(&mut buffer)
            .unwrap()
            .unwrap();

        if let DataReaderContract::DeleteRows { table_name, rows } = dest {
            assert_eq!("test", table_name);
            assert_eq!(2, rows.len());
            assert_eq!("pk1", rows[0].0);
            assert_eq!("rk1", rows[0].1);

            assert_eq!("pk2", rows[1].0);
            assert_eq!("rk2", rows[1].1);
        } else {
            panic!("Dest contract is wrong {:?}", &dest);
        }
    }
}
