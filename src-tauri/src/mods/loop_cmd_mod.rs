use log::trace;
use tauri::{AppHandle, Manager};
use crate::{mods::mcu_control_mod::{self}, GlobalState};

#[tauri::command]
pub async fn cmd_1kms_loop(app: AppHandle) -> Result<String, String> {
    let global_state = app.state::<GlobalState>();
    let receive_buffer = global_state.receive_buffer.lock().await;
    receive_buffer.show(5);
    let message = format!("1kms loop finish");
    trace!("{}", message);
    Ok(message)
}

#[tauri::command]
pub async fn cmd_50ms_loop(app: AppHandle) -> Result<String, String> {
    mcu_control_mod::receive_packet_proccess(app).await;
    let message = format!("50ms loop finish");
    trace!("{}", message);
    Ok(message)
}
