use std::{error::Error, fs, mem};
use log::{debug, error, info};
use regex::Regex;
use tauri::{AppHandle, Manager};
use crate::{mods::{directory_mod, mcu_const, mcu_store_mod::{DataType, MotorDataType}, uart_packet_mod::UartPacket}, GlobalState, BASE_GEN_FILES_FOLDER, ROOT_GEN_FILES_FOLDER};

const CONST_RS_PATH: &str = include_str!(
    concat!(env!("CARGO_MANIFEST_DIR"), "/src/mods/mcu_const.rs")
);

/// 由UartPacket組成的傳送接收緩存區，包含UartPacket及緩存區大小<br>
/// Transmission/reception buffer composed of UartPacket elements, including the packets and buffer capacity
#[derive(Debug)]
pub struct UartTrceBuffer {
    packets: Vec<UartPacket>,  // 真正的槽位 / storage for packets
    max_length: usize,         // 最大槽位數 / maximum number of slots
}
impl UartTrceBuffer {
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

/// 生成 MCU 常量的 C 標頭檔案<br>
/// Generates a C header file containing MCU constant definitions
pub fn gen_h_file(app: AppHandle) -> Result<(), Box<dyn Error>> {
    let global_state = app.state::<GlobalState>();
    let folder_path= {
        let root_path = global_state.root_path.lock().unwrap().clone();
        root_path.join(ROOT_GEN_FILES_FOLDER).join(BASE_GEN_FILES_FOLDER)
    };
    let src = CONST_RS_PATH.to_string();
    let out_h = 
        directory_mod::create_file(folder_path, "mcu_const.h")?;
    
    let mut contents = format!(
        "/**\n * ! Generate by code, do not edit !\n */\n#ifndef MCU_CONST_H\n#define MCU_CONST_H\n\n#include <stdint.h>\n\n"
    );
    
    // 4. 用正則擷取 define_cmd! 宏
    let pub_const = Regex::new(
        r#"pub\s+const\s+(?P<name>\w+)\s*:\s*[^=]+=\s*(?P<bytes>0x[0-9A-Fa-f]+|\d+)\s*;"#
    ).unwrap();
    for caps in pub_const.captures_iter(&src) {
        contents = format!("{}#define {} {}\n", contents, &caps["name"], &caps["bytes"]);
    }
    contents = format!("{}\n", contents);

    let define_cmd = Regex::new(
    r#"define_cmd!\(\s*(?P<name>\w+)\s*,\s*\[\s*(?P<args>[^\]]+)\s*\]\s*\)\s*;"#
    ).unwrap();
    for caps in define_cmd.captures_iter(&src) {
        contents = format!("{}#define {} ((uint8_t[]){{{}}})\n", contents, &caps["name"], &caps["args"]);
    }

    contents = format!("{}\n#endif\n", contents);

    fs::write(&out_h, contents)?;
    Ok(())
}

/// 處理 UART 接收緩衝區的封包，依據命令碼執行對應動作，最多處理n筆<br>
/// Processes up to n UART packets from the receive buffer and dispatches actions based on command codes
pub async fn re_pkt_proccess(app: AppHandle) {
    let global_state = app.state::<GlobalState>();
    for _ in 0..10 {
        let maybe_pkt = {
            let mut uart_recv_buffer = global_state.uart_recv_buffer.lock().await;
            uart_recv_buffer.pop_front()
        };
        let mut data = match maybe_pkt {
            Err(_) => break,
            Ok(p) => p.data(),
        };
        match data.remove(0) {
            cmd if cmd == mcu_const::CMD_CODE_DATA_TRRE => re_pkt_data_store(app.clone(), data).await,
            cmd if cmd == mcu_const::CMD_CODE_VECH_CONTROL => break,
            _ => break,
        };
    }
}

/// 解析資料封包並將值存入全域狀態<br>
/// Parses data packets and stores the extracted values into the global state
async fn re_pkt_data_store(app: AppHandle, mut data: Vec<u8>) {
    let global_state = app.state::<GlobalState>();
    loop {
        if        data.starts_with(mcu_const::CMD_LEFT_SPEED_STORE.payload) {
            data.drain(..mcu_const::CMD_LEFT_SPEED_STORE.payload.len());
            let value = match data[..size_of::<f32>()].try_into() {
                Ok(bytes) => {
                    data.drain(..size_of::<f32>());
                    f32::from_bits(u32::from_be_bytes(bytes))
                },
                Err(e) => {
                    error!("ERR: {}", e);
                    break;
                }
            };
            let mut store_datas = global_state.store_datas.lock().await;
            store_datas.push(DataType::MotorLeft(MotorDataType::SpeedPresent), value as u64);
        }
        else if data.starts_with(mcu_const::CMD_RIGHT_SPEED_STORE.payload) {
            data.drain(..mcu_const::CMD_RIGHT_SPEED_STORE.payload.len());
            let value = match data[..size_of::<f32>()].try_into() {
                Ok(bytes) => {
                    data.drain(..size_of::<f32>());
                    f32::from_bits(u32::from_be_bytes(bytes))
                },
                Err(e) => {
                    error!("ERR: {}", e);
                    break;
                }
            };
            let mut store_datas = global_state.store_datas.lock().await;
            store_datas.push(DataType::MotorRight(MotorDataType::SpeedPresent), value as u64);
        }
        else if data.starts_with(mcu_const::CMD_LEFT_ADC_STORE.payload) {
            data.drain(..mcu_const::CMD_LEFT_ADC_STORE.payload.len());
            let value = match data[..size_of::<u16>()].try_into() {
                Ok(bytes) => {
                    data.drain(..size_of::<u16>());
                    u16::from_be_bytes(bytes)
                },
                Err(e) => {
                    error!("ERR: {}", e);
                    break;
                }
            };
            let mut store_datas = global_state.store_datas.lock().await;
            store_datas.push(DataType::MotorLeft(MotorDataType::AdcValue), value);
        }
        else if data.starts_with(mcu_const::CMD_RIGHT_ADC_STORE.payload) {
            data.drain(..mcu_const::CMD_RIGHT_ADC_STORE.payload.len());
            let value = match data[..size_of::<u16>()].try_into() {
                Ok(bytes) => {
                    data.drain(..size_of::<u16>());
                    u16::from_be_bytes(bytes)
                },
                Err(e) => {
                    error!("ERR: {}", e);
                    break;
                }
            };
            let mut store_datas = global_state.store_datas.lock().await;
            store_datas.push(DataType::MotorRight(MotorDataType::AdcValue), value);
        }
        else { break; }
    }
}
