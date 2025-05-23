use std::{error::Error, fs};
use log::error;
use regex::Regex;
use tauri::{AppHandle, Manager};
use crate::{GlobalState, mods::{directory_mod, mcu_const, mcu_control_mod::{DataStoreSelF32, DataStoreSelU16}}};

const H_FILE_GEN_FOLDER: &str = "generate/base";

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

pub async fn re_pkt_proccess(app: AppHandle, count: u8) {
    let global_state = app.state::<GlobalState>();
    for _ in 0..count {
        let maybe_pkt = {
            let mut receive_buffer = global_state.receive_buffer.lock().await;
            receive_buffer.pop_front()
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
            store_datas.push_f32(DataStoreSelF32::LeftSpeed, value);
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
            store_datas.push_f32(DataStoreSelF32::RightSpeed, value);
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
            store_datas.push_i16(DataStoreSelU16::LeftAdc, value);
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
            store_datas.push_i16(DataStoreSelU16::RightAdc, value);
        }
        else { break; }
    }
}
