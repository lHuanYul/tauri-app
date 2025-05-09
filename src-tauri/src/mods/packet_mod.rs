/// 定義 UART 封包的起始、結尾符號及資料長度常數<br>
/// Define constants for UART packet start code, end code, and data length
const       PACKET_START_CODE:      u8 = b'{';
pub const   PACKET_END_CODE:        u8 = b'}';
const       PACKET_DATA_MAX_SIZE:   usize = 38;
pub const   PACKET_MAX_SIZE:        usize = PACKET_DATA_MAX_SIZE + 2;

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
            let msg = format!("Data too long: length = {}, max = {}", data.len(), PACKET_DATA_MAX_SIZE);
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
    pub fn data(&self) -> &[u8] { &self.data }
    
    /// 取得封包結尾符號<br>
    /// Returns the packet end code
    pub fn end(&self) -> u8 { self.end }

    /// 顯示封包內容為字串<br>
    /// Formats packet contents into a string for display
    pub fn show(&self) -> String {
        format!(
            "\n>>> {}\n>>>   Data: {:?}\n>>> {}",
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
        return Ok(());
    }

    /// 檢驗結尾符號是否正確<br>
    /// Validates that the packet end code matches PACKET_END_CODE
    pub fn check_end_code(&self) -> Result<(), String> {
        if self.end != PACKET_END_CODE {
            let _msg = format!("Invalid packet end code (expected {}): {}", PACKET_END_CODE, self.end);
            return Err(_msg);
        }
        return Ok(());
    }

    /// 從原始緩衝區解析封包並檢驗結尾碼<br>
    /// Parses a raw buffer into a UartPacket and validates end code
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

    /// 將封包序列化為位元組向量<br>
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

#[derive(Debug)]
pub struct TrReBuffer {
    packets: Vec<UartPacket>,  // 真正的槽位
    max_length: usize,                // 最大槽位數
}
impl TrReBuffer {
    /// 建一個空的 TransferBuffer，並指定最大槽數
    pub fn new(max_length: usize) -> Self {
        Self {
            packets:    Vec::new(),
            max_length,
        }
    }

    /// 目前已放入幾個有效封包
    pub fn get_length(&self) -> usize { self.packets.len() }

    /// 是否已經滿了（沒有空槽）
    pub fn is_full(&self) -> bool { self.packets.len() >= self.max_length }

    /// 是否為空（沒有任何封包）
    pub fn is_empty(&self) -> bool { self.packets.is_empty() }

    /// 將 packet push 到尾端，若超過容量則回 Err
    pub fn push(
        &mut self,
        packet: UartPacket
    ) -> Result<(), String> {
        if self.is_full() {
            let _msg = format!("Buffer is full (max: {})", self.max_length);
            return Err(_msg);
        }
        self.packets.push(packet);
        Ok(())
    }

    /// 從指定索引取出封包（移除並回傳），若該槽為空則回 None
    pub fn pop_front(&mut self) -> Option<UartPacket> {
        if self.packets.is_empty() {
            None
        } else {
            Some(self.packets.remove(0))
        }
    }

    /// 取出所有封包並清空
    pub fn take_all(&mut self) -> Vec<UartPacket> { std::mem::take(&mut self.packets) }
}
