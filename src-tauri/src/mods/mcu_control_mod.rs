use log::{debug, info};
use tauri::{AppHandle, Manager};
use crate::{mods::{packet_mod::UartPacket, mcu_const}, GlobalState};

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
        let count = self.packets.len().min(n);
        if n > count {
            debug!("Ask for show {}, but only have {}", n, self.packets.len());
        }
        for (idx, pkt) in self.packets.iter().take(count).enumerate() {
            info!("TrReBuffer show[{}]:\n{}", idx, pkt.show());
        }
    }
}

pub struct DataStore {
    left_speed: Vec<f32>,
    right_speed: Vec<f32>,
    left_adc: Vec<u16>,
    right_adc: Vec<u16>,
}
pub enum DataStoreSelF32 {
    LeftSpeed,
    RightSpeed,
}
pub enum DataStoreSelU16 {
    LeftAdc,
    RightAdc,
}
macro_rules! push_then_truncate {
    ($vec:expr, $val:expr, $max:expr) => {{
        $vec.push($val);
        if $vec.len() > $max {
            $vec.remove(0);
        }
    }};
}
impl DataStore {
    pub fn new() -> Self {
        Self {
            left_speed: Vec::new(),
            right_speed: Vec::new(),
            left_adc: Vec::new(),
            right_adc: Vec::new(),
        }
    }

    pub fn push_f32(&mut self, select: DataStoreSelF32, value: f32) {
        match select {
            DataStoreSelF32::LeftSpeed =>    push_then_truncate!(self.left_speed, value, 100),
            DataStoreSelF32::RightSpeed =>   push_then_truncate!(self.right_speed, value, 100),
        }
    }
    pub fn push_i16(&mut self, select: DataStoreSelU16, value: u16) {
        match select {
            DataStoreSelU16::LeftAdc =>      push_then_truncate!(self.left_adc, value, 100),
            DataStoreSelU16::RightAdc =>     push_then_truncate!(self.right_adc, value, 100),
        }
    }

    pub fn show_f32(&mut self, select: DataStoreSelF32) -> &[f32] {
        match select {
            DataStoreSelF32::LeftSpeed => &self.left_speed,
            DataStoreSelF32::RightSpeed => &self.right_speed,
        }
    }
    pub fn show_u16(&mut self, select: DataStoreSelU16) -> &[u16] {
        match select {
            DataStoreSelU16::LeftAdc => &self.left_adc,
            DataStoreSelU16::RightAdc => &self.right_adc,
        }
    }
}

#[tauri::command]
pub async fn cmd_send_spd_stop(app: AppHandle) -> Result<(), String> {
    let global_state = app.state::<GlobalState>();
    let mut cmd = Vec::<u8>::new();
    cmd.push(mcu_const::CMD_CODE_DATA_TRRE);
    cmd.extend(mcu_const::CMD_RIGHT_SPEED_STOP.payload.to_vec());
    cmd.extend(mcu_const::CMD_RIGHT_ADC_STOP.payload.to_vec());
    let mut transfer_buffer = global_state.transfer_buffer.lock().await;
    let packet = UartPacket::new(cmd)?;
    transfer_buffer.push(packet)?;
    Ok(())
}

#[tauri::command]
pub async fn cmd_send_spd_once(app: AppHandle) -> Result<(), String> {
    let global_state = app.state::<GlobalState>();
    let mut cmd = Vec::<u8>::new();
    cmd.push(mcu_const::CMD_CODE_DATA_TRRE);
    cmd.extend(mcu_const::CMD_RIGHT_SPEED_ONCE.payload.to_vec());
    cmd.extend(mcu_const::CMD_RIGHT_ADC_ONCE.payload.to_vec());
    let mut transfer_buffer = global_state.transfer_buffer.lock().await;
    let packet = UartPacket::new(cmd)?;
    transfer_buffer.push(packet)?;
    Ok(())
}

#[tauri::command]
pub async fn cmd_send_spd_start(app: AppHandle) -> Result<(), String> {
    let global_state = app.state::<GlobalState>();
    let mut cmd = Vec::<u8>::new();
    cmd.push(mcu_const::CMD_CODE_DATA_TRRE);
    cmd.extend(mcu_const::CMD_RIGHT_SPEED_START.payload.to_vec());
    cmd.extend(mcu_const::CMD_RIGHT_ADC_START.payload.to_vec());
    let mut transfer_buffer = global_state.transfer_buffer.lock().await;
    let packet = UartPacket::new(cmd)?;
    transfer_buffer.push(packet)?;
    Ok(())
}

