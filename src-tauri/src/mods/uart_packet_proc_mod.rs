use std::{error::Error, fs};
use log::error;
use regex::Regex;
use tauri::{AppHandle, Manager};
use crate::{mods::{directory_mod, mcu_const, mcu_store_mod::{DataType, MotorDataType}}, GlobalState, BASE_GEN_FILES_FOLDER, ROOT_GEN_FILES_FOLDER};

const CONST_RS_PATH: &str = include_str!(
    concat!(env!("CARGO_MANIFEST_DIR"), "/src/mods/mcu_const.rs")
);

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
            let mut uart_recv_buffer = global_state.uart_receive_buffer.lock().await;
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
