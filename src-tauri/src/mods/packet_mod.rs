/// 定義 UART 封包的起始、結尾符號及資料長度常數<br>
/// Define constants for UART packet start code, end code, and data length
/* #region define_cmd */
const       PACKET_START_CODE:      u8 = b'{';
pub const   PACKET_END_CODE:        u8 = b'}';
const       PACKET_DATA_MAX_SIZE:   usize = 38;
pub const   PACKET_MAX_SIZE:        usize = PACKET_DATA_MAX_SIZE + 2;
/* #endregion */

/// UART 封包結構，包含起始碼、固定長度資料與結尾碼<br>
/// UartPacket struct representing a UART packet with start code, fixed-size data, and end code
#[derive(Debug, Clone)]
pub struct UartPacket {
    start: u8,              // 起始符號 / start code
    data: Vec<u8>,          // 資料陣列 / packet data array
    end: u8,                // 結尾符號 / end code
}
impl UartPacket {
    /// 創建新封包，設定資料與結尾碼<br>
    /// Creates a new UartPacket with the given data and sets the end code
    pub fn new(data: Vec<u8>) -> Result<Self, String> {
        if data.len() > PACKET_DATA_MAX_SIZE {
            let msg = format!("Data too long: length = {}, max = {PACKET_DATA_MAX_SIZE}", data.len());
            return Err(msg);
        }
        Ok(Self {
            start:  PACKET_START_CODE,
            data,
            end:    PACKET_END_CODE,
        })
    }

    /// 取得封包起始符號<br>
    /// Returns the packet start code
    pub fn start(&self) -> u8 { self.start }
    
    /// 取得封包資料參考<br>
    /// Returns a reference to the packet data array
    pub fn data(&self) -> Vec<u8> { self.data.clone() }
    
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

    /// 檢驗起始符號是否正確<br>
    /// Validates that the packet start code matches PACKET_START_CODE
    pub fn check_start_code(&self) -> Result<(), String> {
        if self.start != PACKET_START_CODE {
            let _msg = format!("Invalid packet start code (expected {}): {}", PACKET_START_CODE, self.start);
            return Err(_msg);
        }
        Ok(())
    }

    /// 檢驗結尾符號是否正確<br>
    /// Validates that the packet end code matches PACKET_END_CODE
    pub fn check_end_code(&self) -> Result<(), String> {
        if self.end != PACKET_END_CODE {
            let _msg = format!("Invalid packet end code (expected {}): {}", PACKET_END_CODE, self.end);
            return Err(_msg);
        }
        Ok(())
    }

    /// 從原始緩衝區解析封包並檢驗碼<br>
    /// Parses a raw buffer into a UartPacket and validates start/end codes
    pub fn pack(data: Vec<u8>) -> Result<Self, String> {
        if data.first() != Some(&PACKET_START_CODE) {
            let _msg = format!("Start byte invalid (expected {:?}): {:?}", PACKET_START_CODE, data.first());
            return Err(_msg);
        }
        if data.last() != Some(&PACKET_END_CODE) {
            let _msg = format!("End byte invalid (expected {:?}): {:?}", PACKET_END_CODE, data.last());
            return Err(_msg);
        }
        let packet = Self::new(data[1..(data.len()-1)].to_vec())?;
        Ok(packet)
    }

    /// 將封包反序列化為位元組向量<br>
    /// Serializes the UartPacket into a byte vector including start, data, and end codes
    pub fn unpack(&self) -> Result<Vec<u8>, String> {
        self.check_start_code()?;
        self.check_end_code()?;
        let mut buffer: Vec<u8> = Vec::with_capacity(self.data.len() + 2);
        buffer.push(self.start);
        buffer.extend_from_slice(&self.data);
        buffer.push(self.end);
        Ok(buffer)
    }
}
