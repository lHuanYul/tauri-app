use log::error;
use tauri::{AppHandle, Manager};
use crate::{mods::{uart_packet_mod::UartPacket, mcu_const}, GlobalState};

#[tauri::command]
pub async fn cmd_send_spd_stop(app: AppHandle) -> Result<(), String> {
    let global_state = app.state::<GlobalState>();
    let mut cmd = Vec::<u8>::new();
    cmd.push(mcu_const::CMD_CODE_DATA_TRRE);
    cmd.extend(mcu_const::CMD_RIGHT_SPEED_STOP.payload.to_vec());
    cmd.extend(mcu_const::CMD_RIGHT_ADC_STOP.payload.to_vec());
    let mut transmit_buffer = global_state.uart_transmit_buffer.lock().await;
    let packet = UartPacket::new(cmd).map_err(|e| {
        let message = format!("{}", e);
        error!("{}", message);
        message
    })?;
    transmit_buffer.push(packet).map_err(|e| format!("{}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn cmd_send_spd_once(app: AppHandle) -> Result<(), String> {
    let global_state = app.state::<GlobalState>();
    let mut cmd = Vec::<u8>::new();
    cmd.push(mcu_const::CMD_CODE_DATA_TRRE);
    cmd.extend(mcu_const::CMD_RIGHT_SPEED_ONCE.payload.to_vec());
    cmd.extend(mcu_const::CMD_RIGHT_ADC_ONCE.payload.to_vec());
    let mut transmit_buffer = global_state.uart_transmit_buffer.lock().await;
    let packet = UartPacket::new(cmd).map_err(|e| {
        let message = format!("{}", e);
        error!("{}", message);
        message
    })?;
    transmit_buffer.push(packet).map_err(|e| format!("{}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn cmd_send_spd_start(app: AppHandle) -> Result<(), String> {
    let global_state = app.state::<GlobalState>();
    let mut cmd = Vec::<u8>::new();
    cmd.push(mcu_const::CMD_CODE_DATA_TRRE);
    cmd.extend(mcu_const::CMD_RIGHT_SPEED_START.payload.to_vec());
    cmd.extend(mcu_const::CMD_RIGHT_ADC_START.payload.to_vec());
    let mut transmit_buffer = global_state.uart_transmit_buffer.lock().await;
    let packet = UartPacket::new(cmd).map_err(|e| {
        let message = format!("{}", e);
        error!("{}", message);
        message
    })?;
    transmit_buffer.push(packet).map_err(|e| format!("{}", e))?;
    Ok(())
}
