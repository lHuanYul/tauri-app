use std::error::Error;
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
            self.data,
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
