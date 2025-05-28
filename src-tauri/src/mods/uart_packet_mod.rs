use std::{error::Error, mem};
use log::{debug, info};

use crate::mods::user_vec_mod::UserVecU8;

/// 定義 UART 封包的起始、結尾符號及資料長度常數<br>
/// Define constants for UART packet start code, end code, and data length
/* #region define_cmd */
const       UART_PACKET_START_CODE:      u8 = b'{';
pub const   UART_PACKET_END_CODE:        u8 = b'}';
pub const   UART_PACKET_MAX_SIZE:        usize = 255;
const       UART_PACKET_DATA_MAX_SIZE:   usize = UART_PACKET_MAX_SIZE - 2;
/* #endregion */

/// UART 封包結構，包含起始碼、固定長度資料與結尾碼<br>
/// UartPacket struct representing a UART packet with start code, fixed-size data, and end code
#[derive(Debug, Clone)]
pub struct UartPacket {
    start: u8,              // 起始符號 / start code
    data: UserVecU8,          // 資料陣列 / packet data array
    end: u8,                // 結尾符號 / end code
}
impl UartPacket {
    /// 創建新封包，設定資料與結尾碼<br>
    /// Creates a new UartPacket with the given data and sets the end code
    pub fn new<T: AsRef<[u8]>>(data: T) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let mut user_vec = UserVecU8::new(UART_PACKET_DATA_MAX_SIZE);
        user_vec.extend(data)?;
        Ok( Self {
            start: UART_PACKET_START_CODE,
            data: user_vec,
            end: UART_PACKET_END_CODE,
        })
    }

    /// 取得封包起始符號<br>
    /// Returns the packet start code
    pub fn start(&self) -> u8 { self.start }
    
    /// 取得封包資料參考<br>
    /// Returns a reference to the packet data array
    pub fn data(&self) -> Vec<u8> { self.data.show() }
    
    /// 取得封包結尾符號<br>
    /// Returns the packet end code
    pub fn end(&self) -> u8 { self.end }

    /// 顯示封包內容為字串<br>
    /// Formats packet contents into a string for display
    pub fn show(&self) -> String {
        format!(
            ">>> {}\n>>>   {:?}\n>>> {}",
            self.start as char,
            self.data.show(),
            self.end as char
        )
    }

    /// 從原始緩衝區解析封包並檢驗碼<br>
    /// Parses a raw buffer into a UartPacket and validates start/end codes
    pub fn pack(data: Vec<u8>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        if data.first() != Some(&UART_PACKET_START_CODE) {
            let message = format!("Start byte invalid (expected {:?}): {:?}", UART_PACKET_START_CODE, data.first());
            return Err(message.into());
        }
        if data.last() != Some(&UART_PACKET_END_CODE) {
            let message = format!("End byte invalid (expected {:?}): {:?}", UART_PACKET_END_CODE, data.last());
            return Err(message.into());
        }
        let packet = Self::new(data[1..(data.len()-1)].to_vec())?;
        Ok(packet)
    }

    /// 將封包反序列化為位元組向量<br>
    /// Serializes the UartPacket into a byte vector including start, data, and end codes
    pub fn unpack(&self) -> Vec<u8> {
        let mut buffer= Vec::<u8>::new();
        buffer.push(self.start);
        buffer.extend_from_slice(&self.data.show());
        buffer.push(self.end);
        buffer
    }
}


/// 由UartPacket組成的傳送接收緩存區，包含UartPacket及緩存區大小<br>
/// Transmission/reception buffer composed of UartPacket elements, including the packets and buffer capacity
#[derive(Debug)]
pub struct UartTransceiveBuffer {
    packets: Vec<UartPacket>,  // 真正的槽位 / storage for packets
    max_length: usize,         // 最大槽位數 / maximum number of slots
}
impl UartTransceiveBuffer {
    /// 建立 Transceive Buffer，並指定最大容量<br>
    /// Creates a new TrReBuffer with a specified maximum capacity
    pub fn new(max_length: usize) -> Self {
        Self {
            packets:    Vec::new(),
            max_length,
        }
    }

    /// 取得目前已儲存封包數<br>
    /// Returns the current number of stored packets
    pub fn get_length(&self) -> usize {
        self.packets.len()
    }

    /// 檢查緩衝區是否已滿<br>
    /// Returns true if the buffer has reached its maximum capacity
    pub fn is_full(&self) -> bool {
        self.packets.len() >= self.max_length
    }

    /// 檢查緩衝區是否為空<br>
    /// Returns true if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    /// 將封包推入尾端；若容量已滿則回傳 Err <br>
    /// Pushes a packet to the end; returns Err if the buffer is full
    pub fn push(&mut self, packet: UartPacket) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.is_full() {
            let message = format!("Buffer is full (max: {})", self.max_length);
            return Err(message.into());
        }
        self.packets.push(packet);
        Ok(())
    }

    // 從前端彈出並回傳封包；若為空則回傳 None <br>
    /// Removes and returns the packet at the front; returns None if empty
    pub fn pop_front(&mut self) -> Result<UartPacket, Box<dyn Error + Send + Sync>> {
        if self.is_empty() {
            let message = format!("Buffer is empty");
            return Err(message.into());
        }
        Ok(self.packets.remove(0))
    }

    /// 取出所有封包並清空緩衝區<br>
    /// Takes all packets and clears the buffer
    pub fn take_all(&mut self) -> Vec<UartPacket> {
        mem::take(&mut self.packets)
    }

    /// 顯示前 n 個封包，不會從緩衝區移除<br>
    /// Shows the first n packets without removing them from the buffer
    pub fn show(&self, n: usize) {
        let count = self.packets.len().min(n);
        if n > count {
            debug!("Ask for show {}, but only have {}", n, self.packets.len());
        }
        for (idx, pkt) in self.packets.iter().take(count).enumerate() {
            info!("TrReBuffer show[{}]:\n{}", idx, pkt.show());
        }
    }
}
