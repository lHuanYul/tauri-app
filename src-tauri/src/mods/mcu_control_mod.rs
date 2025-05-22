use log::error;
use tauri::{AppHandle, Manager};
use crate::{mods::{directory_mod, packet_mod::UartPacket, mcu_cmd_mod}, GlobalState};
use std::{error::Error, fs};
use regex::Regex;

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

const STORE_FOLDER: &str = "generate/base";

pub fn gen_h_file(app: AppHandle) -> Result<(), Box<dyn Error>> {
    let global_state = app.state::<GlobalState>();
    let root_path= {
        let root_path = global_state.root_path.lock().unwrap();
        root_path.clone()
    };
    let src_path = root_path.join("src/mods/mcu_cmd_mod.rs");
    let src = fs::read_to_string(&src_path)?;
    let out_h = 
        directory_mod::create_file(root_path.join(STORE_FOLDER), "mcu_cmd.h")?;
    
    let mut contents = format!("#ifndef MCU_CMD_H\n#define MCU_CMD_H\n\n");
    contents = format!("{}#include <stdint.h>\n\n", contents);
    
    // 4. 用正則擷取 define_cmd! 宏
    let pub_const = Regex::new(
        r#"pub\s+const\s+(?P<name>\w+)\s*:\s*[^=]+=\s*(?P<bytes>0x[0-9A-Fa-f]+|\d+)\s*;"#
    ).unwrap();
    for caps in pub_const.captures_iter(&src) {
        contents = format!("{}#define {} {}\n", contents, &caps["name"], &caps["bytes"]);
    }
    contents = format!("{}\n", contents);

    let define_cmd = Regex::new(
        r#"define_cmd!\(\s*(?P<const_name>\w+)\s*,\s*"(?P<name>[^"]+)"\s*,\s*\[(?P<bytes>[^\]]+)\]\s*\)\s*;"#
    ).unwrap();
    for caps in define_cmd.captures_iter(&src) {
        contents = format!("{}#define {} ((uint8_t[]){{{}}})\n", contents, &caps["name"], &caps["bytes"]);
    }

    contents = format!("{}\n#endif\n", contents);

    fs::write(&out_h, contents)?;
    Ok(())
}

pub async fn send_cmd(app: AppHandle) -> Result<(), String> {
    let global_state = app.state::<GlobalState>();
    let mut cmd = Vec::<u8>::new();
    cmd.push(mcu_cmd_mod::CMD_CODE_DATA_TRRE);
    cmd.extend(mcu_cmd_mod::RIGHT_SPEED_START.payload.to_vec());
    let mut transfer_buffer = global_state.transfer_buffer.lock().await;
    let packet = UartPacket::new(cmd)?;
    transfer_buffer.push(packet)
}

pub async fn re_pkt_proccess(app: AppHandle) {
    let global_state = app.state::<GlobalState>();
    for _ in 0..10 {
        let maybe_pkt = {
            let mut receive_buffer = global_state.receive_buffer.lock().await;
            receive_buffer.pop_front()
        };
        let mut data = match maybe_pkt {
            None => break,
            Some(p) => p.data(),
        };
        match data.remove(0) {
            cmd if cmd == mcu_cmd_mod::CMD_CODE_DATA_TRRE => re_pkt_data_store(app.clone(), data).await,
            cmd if cmd == mcu_cmd_mod::CMD_CODE_VECH_CONTROL => break,
            _ => break,
        };
    }
}

async fn re_pkt_data_store(app: AppHandle, mut data: Vec<u8>) {
    let global_state = app.state::<GlobalState>();
    loop {
        if        data.starts_with(mcu_cmd_mod::LEFT_SPEED_STORE.payload) {
            data.drain(..mcu_cmd_mod::LEFT_SPEED_STORE.payload.len());
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
            store_datas.push_f32(DataStoreSelF32::LeftSpeed, value);
        }
        else if data.starts_with(mcu_cmd_mod::RIGHT_SPEED_STORE.payload) {
            data.drain(..mcu_cmd_mod::RIGHT_SPEED_STORE.payload.len());
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
            store_datas.push_f32(DataStoreSelF32::RightSpeed, value);
        }
        else if data.starts_with(mcu_cmd_mod::LEFT_ADC_STORE.payload) {
            data.drain(..mcu_cmd_mod::LEFT_ADC_STORE.payload.len());
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
            store_datas.push_i16(DataStoreSelU16::LeftAdc, value);
        }
        else if data.starts_with(mcu_cmd_mod::RIGHT_ADC_STORE.payload) {
            data.drain(..mcu_cmd_mod::RIGHT_ADC_STORE.payload.len());
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
            store_datas.push_i16(DataStoreSelU16::RightAdc, value);
        }
        else { break; }
    }
}
