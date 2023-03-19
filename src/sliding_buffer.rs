use std::io::{self, Seek, SeekFrom};

use std::io::Read;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct SlidingBuffer {
    buffer: Arc<Mutex<Vec<u8>>>,
    position: Arc<Mutex<usize>>,
}

impl SlidingBuffer {
    pub fn new() -> Self {
        SlidingBuffer {
            buffer: Arc::new(Mutex::new(Vec::new())),
            position: Arc::new(Mutex::new(0)),
        }
    }

    pub fn append(&self, data: &[u8]) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(data);
    }
    pub fn len(&self) -> usize {
        let buffer = self.buffer.lock().unwrap();
        buffer.len()
    }
    pub fn trim() {}
}

impl Read for SlidingBuffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let buffer = self.buffer.lock().unwrap();
        let mut position = self.position.lock().unwrap();

        let bytes_to_read = buf.len().min(buffer.len() - *position);
        buf[..bytes_to_read].copy_from_slice(&buffer[*position..*position + bytes_to_read]);
        *position += bytes_to_read;
        Ok(bytes_to_read)
    }
}

impl Seek for SlidingBuffer {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let buffer = self.buffer.lock().unwrap();
        let mut position = self.position.lock().unwrap();

        *position = match pos {
            SeekFrom::Start(offset) => offset as usize,
            SeekFrom::Current(offset) => ((*position as i64) + offset) as usize,
            SeekFrom::End(offset) => ((buffer.len() as i64) + offset) as usize,
        };

        Ok(*position as u64)
    }
}
