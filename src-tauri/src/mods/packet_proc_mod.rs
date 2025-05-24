use std::{error::Error, fs};
use log::{debug, error, info};
use regex::Regex;
use tauri::{AppHandle, Manager};
use crate::{GlobalState, mods::{directory_mod, mcu_const, mcu_store_mod::{DataType, MotorDataType}, packet_mod::UserPacket}};

const H_FILE_GEN_FOLDER: &str = "generate/base";

/// 由UartPacket組成的傳送接收緩存區，包含UartPacket及緩存區大小<br>
/// Transmission/reception buffer composed of UartPacket elements, including the packets and buffer capacity
#[derive(Debug)]
pub struct TrceBuffer {
    packets: Vec<UserPacket>,  // 真正的槽位 / storage for packets
    max_length: usize,         // 最大槽位數 / maximum number of slots
}
impl TrceBuffer {
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
    pub fn push(&mut self, packet: UserPacket) -> Result<(), String> {
        if self.is_full() {
            let _msg = format!("Buffer is full (max: {})", self.max_length);
            return Err(_msg);
        }
        self.packets.push(packet);
        Ok(())
    }

    /// 從前端彈出並回傳封包；若為空則回傳 None <br>
    /// Removes and returns the packet at the front; returns None if empty
    pub fn pop_front(&mut self) -> Option<UserPacket> {
        if self.packets.is_empty() {
            None
        } else {
            Some(self.packets.remove(0))
        }
    }

    /// 取出所有封包並清空緩衝區<br>
    /// Takes all packets and clears the buffer
    pub fn take_all(&mut self) -> Vec<UserPacket> {
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

pub fn gen_h_file(app: AppHandle) -> Result<(), Box<dyn Error>> {
    let global_state = app.state::<GlobalState>();
    let root_path= {
        let root_path = global_state.root_path.lock().unwrap();
        root_path.clone()
    };
    let src_path = root_path.join("src/mods/mcu_const.rs");
    let src = fs::read_to_string(&src_path)?;
    let out_h = 
        directory_mod::create_file(root_path.join(H_FILE_GEN_FOLDER), "mcu_const.h")?;
    
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

pub async fn re_pkt_proccess(app: AppHandle) {
    let global_state = app.state::<GlobalState>();
    let count = {
        let uart_recv_buffer = global_state.uart_recv_buffer.lock().await;
        uart_recv_buffer.get_length()
    };
    for _ in 0..count {
        let maybe_pkt = {
            let mut uart_recv_buffer = global_state.uart_recv_buffer.lock().await;
            uart_recv_buffer.pop_front()
        };
        let mut data = match maybe_pkt {
            None => break,
            Some(p) => p.data(),
        };
        match data.remove(0) {
            cmd if cmd == mcu_const::CMD_CODE_DATA_TRRE => re_pkt_data_store(app.clone(), data).await,
            cmd if cmd == mcu_const::CMD_CODE_VECH_CONTROL => break,
            _ => break,
        };
    }
}

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
