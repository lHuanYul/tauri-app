use log::info;

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

/// 由UartPacket組成的傳送接收緩存區，包含UartPacket及緩存區大小<br>
/// Transmission/reception buffer composed of UartPacket elements, including the packets and buffer capacity
#[derive(Debug)]
pub struct TrReBuffer {
    packets: Vec<UartPacket>,  // 真正的槽位 / storage for packets
    max_length: usize,         // 最大槽位數 / maximum number of slots
}
impl TrReBuffer {
    /// 建立 Transfer Buffer，並指定最大容量<br>
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
    /// Returns true if the buffer contains no packets
    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    /// 將封包推入尾端；若容量已滿則回傳 Err <br>
    /// Pushes a packet to the end; returns Err if the buffer is full
    pub fn push(&mut self, packet: UartPacket) -> Result<(), String> {
        if self.is_full() {
            let _msg = format!("Buffer is full (max: {})", self.max_length);
            return Err(_msg);
        }
        self.packets.push(packet);
        Ok(())
    }

    /// 從前端彈出並回傳封包；若為空則回傳 None <br>
    /// Removes and returns the packet at the front; returns None if empty
    pub fn pop_front(&mut self) -> Option<UartPacket> {
        if self.packets.is_empty() {
            None
        } else {
            Some(self.packets.remove(0))
        }
    }

    /// 取出所有封包並清空緩衝區<br>
    /// Takes all packets and clears the buffer
    pub fn take_all(&mut self) -> Vec<UartPacket> {
        std::mem::take(&mut self.packets)
    }

    
    /// 顯示前 n 個封包，不會從緩衝區移除
    pub fn show(&self, n: usize) {
        // 實際要顯示的數量：取 self.packets.len() 和 n 的最小值
        let count = self.packets.len().min(n);

        for (idx, pkt) in self.packets.iter().take(count).enumerate() {
            info!("TrReBuffer show[{}]:\n{}", idx, pkt.show());
        }

        // 如果 n 超過目前封包數，也可以選擇再 warn 一下
        if n > count {
            info!(
                "要求顯示 {} 個，但目前只有 {} 個封包",
                n, self.packets.len()
            );
        }
    }
}
