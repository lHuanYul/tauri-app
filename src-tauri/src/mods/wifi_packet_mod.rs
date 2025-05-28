use std::{error::Error, mem, net::IpAddr};

use log::{debug, info};

use crate::mods::user_vec_mod::UserVecU8;

pub const WIFI_TCP_PACKET_MAX_SIZE: usize = 1024;
pub const WIFI_UDP_PACKET_MAX_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct WifiPacket {
    target_ip: IpAddr,
    data: UserVecU8,
}
impl WifiPacket {
    pub fn new<T: AsRef<[u8]>>(target_ip: IpAddr, data: T) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut user_vec = UserVecU8::new(WIFI_TCP_PACKET_MAX_SIZE);
        user_vec.extend(data)?;
        Ok( Self {
            target_ip,
            data: user_vec,
        })
    }

    pub fn target_ip(&self) -> IpAddr { self.target_ip.clone() }

    /// 取得封包資料參考<br>
    /// Returns a reference to the packet data array
    pub fn data(&self) -> Vec<u8> { self.data.show() }
    
    /// 顯示封包內容為字串<br>
    /// Formats packet contents into a string for display
    pub fn show(&self) -> String {
        format!(
            ">>>   {:?}",
            self.data.show(),
        )
    }

    // pub fn pack(data: Vec<u8>) -> Result<Self, Box<dyn Error + Send + Sync>> {
    //     if data.first() != Some(&UART_PACKET_START_CODE) {
    //         let message = format!("Start byte invalid (expected {:?}): {:?}", UART_PACKET_START_CODE, data.first());
    //         return Err(message.into());
    //     }
    //     if data.last() != Some(&UART_PACKET_END_CODE) {
    //         let message = format!("End byte invalid (expected {:?}): {:?}", UART_PACKET_END_CODE, data.last());
    //         return Err(message.into());
    //     }
    //     let packet = Self::new(data[1..(data.len()-1)].to_vec())?;
    //     Ok(packet)
    // }

    // pub fn unpack(&self) -> Vec<u8> {
    //     let mut buffer= Vec::<u8>::new();
    //     buffer.push(self.start);
    //     buffer.extend_from_slice(&self.data.show());
    //     buffer.push(self.end);
    //     buffer
    // }
}

#[derive(Debug)]
pub struct WifiTrceBuffer {
    packets: Vec<WifiPacket>,       // 真正的槽位 / storage for packets
    max_pkt_length: usize,          // 最大槽位數 / maximum number of slots
    _max_pkt_data_length: usize,
}
impl WifiTrceBuffer {
    pub fn new(max_pkt_length: usize, max_pkt_data_length: usize) -> Self {
        Self {
            packets: Vec::new(),
            max_pkt_length,
            _max_pkt_data_length: max_pkt_data_length,
        }
    }

    pub fn get_length(&self) -> usize {
        self.packets.len()
    }

    pub fn is_full(&self) -> bool {
        self.packets.len() >= self.max_pkt_length
    }

    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    /// 將 UserVecU8 封包推入尾端；若容量已滿則回傳 Err
    /// Pushes a UserVecU8 packet to the end; returns Err if the buffer is full
    pub fn push(&mut self, packet: WifiPacket) -> Result<(), Box<dyn Error>> {
        if self.is_full() {
            return Err(format!("Buffer is full (max: {})", self.max_pkt_length).into());
        }
        self.packets.push(packet);
        Ok(())
    }

    /// 從前端彈出並回傳 UserVecU8 封包；若為空則回傳 None
    /// Removes and returns the UserVecU8 packet at the front; returns None if empty
    pub fn pop_front(&mut self) -> Result<WifiPacket, Box<dyn Error + Send + Sync>> {
        if self.is_empty() {
            let message = format!("Buffer is empty");
            return Err(message.into());
        }
        Ok(self.packets.remove(0))
    }

    /// 取出所有封包並清空緩衝區
    /// Takes all packets and clears the buffer
    pub fn take_all(&mut self) -> Vec<WifiPacket> {
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
