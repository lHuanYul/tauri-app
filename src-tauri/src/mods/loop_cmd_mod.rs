use log::trace;
use tauri::{AppHandle, Manager};
use crate::{mods::mcu_control_mod::{self}, GlobalState};

#[tauri::command]
pub async fn cmd_1kms_loop(app: AppHandle) -> Result<String, String> {
    let _global_state = app.state::<GlobalState>();
    let message = "1kms loop finish".into();
    trace!("{}", message);
    Ok(message)
}

#[tauri::command]
pub async fn cmd_50ms_loop(app: AppHandle) -> Result<String, String> {
    mcu_control_mod::receive_packet_proccess(app).await;
    let message = "50ms loop finish".into();
    trace!("{}", message);
    Ok(message)
}
