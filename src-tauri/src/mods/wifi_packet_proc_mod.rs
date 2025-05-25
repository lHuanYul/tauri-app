use std::{error::Error, mem};
use log::{debug, info};
use crate::mods::user_vec_mod::UserVecU8;

pub const WIFI_PACKET_MAX_SIZE: usize = 1024;

#[derive(Debug)]
pub struct WifiTrceBuffer {
    packets: Vec<UserVecU8>,  // 真正的槽位 / storage for packets
    max_length: usize,         // 最大槽位數 / maximum number of slots
}
impl WifiTrceBuffer {
    pub fn new(max_length: usize) -> Self {
        Self {
            packets: Vec::new(),
            max_length,
        }
    }

    pub fn get_length(&self) -> usize {
        self.packets.len()
    }

    pub fn is_full(&self) -> bool {
        self.packets.len() >= self.max_length
    }

    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    /// 將 UserVecU8 封包推入尾端；若容量已滿則回傳 Err
    /// Pushes a UserVecU8 packet to the end; returns Err if the buffer is full
    pub fn push<T: AsRef<[u8]>>(&mut self, data: T) -> Result<(), Box<dyn Error>> {
        if self.is_full() {
            return Err(format!("Buffer is full (max: {})", self.max_length).into());
        }
        let mut packet = UserVecU8::new(WIFI_PACKET_MAX_SIZE);
        packet.extend(data).map_err(|e| -> Box<dyn Error> { e })?;
        self.packets.push(packet);
        Ok(())
    }

    /// 從前端彈出並回傳 UserVecU8 封包；若為空則回傳 None
    /// Removes and returns the UserVecU8 packet at the front; returns None if empty
    pub fn pop_front(&mut self) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        if self.is_empty() {
            let message = format!("Buffer is empty");
            return Err(message.into());
        }
        Ok(self.packets.remove(0).show())
    }

    /// 取出所有封包並清空緩衝區
    /// Takes all packets and clears the buffer
    pub fn take_all(&mut self) -> Vec<UserVecU8> {
        mem::take(&mut self.packets)
    }

    /// 顯示最多 n 個封包的內容
    /// Logs up to n packets
    pub fn show(&self, n: usize) {
        let count = self.packets.len().min(n);
        if n > count {
            debug!("Ask for show {}, but only have {}", n, self.packets.len());
        }
        for (idx, pkt) in self.packets.iter().take(count).enumerate() {
            info!("WifiTrceBuffer show[{}]: {:?}", idx, pkt.show());
        }
    }
}
