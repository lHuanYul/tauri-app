use log::error;
use tauri::{AppHandle, Manager};
use crate::{mods::{packet_mod::UserPacket, mcu_const}, GlobalState};

#[tauri::command]
pub async fn cmd_send_spd_stop(app: AppHandle) -> Result<(), String> {
    let global_state = app.state::<GlobalState>();
    let mut cmd = Vec::<u8>::new();
    cmd.push(mcu_const::CMD_CODE_DATA_TRRE);
    cmd.extend(mcu_const::CMD_RIGHT_SPEED_STOP.payload.to_vec());
    cmd.extend(mcu_const::CMD_RIGHT_ADC_STOP.payload.to_vec());
    let mut transfer_buffer = global_state.uart_traf_buffer.lock().await;
    let packet = UserPacket::new(cmd).map_err(|e| {
        let message = format!("{}", e);
        error!("{}", message);
        message
    })?;
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
    let mut transfer_buffer = global_state.uart_traf_buffer.lock().await;
    let packet = UserPacket::new(cmd).map_err(|e| {
        let message = format!("{}", e);
        error!("{}", message);
        message
    })?;
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
    let mut transfer_buffer = global_state.uart_traf_buffer.lock().await;
    let packet = UserPacket::new(cmd).map_err(|e| {
        let message = format!("{}", e);
        error!("{}", message);
        message
    })?;
    transfer_buffer.push(packet)?;
    Ok(())
}
