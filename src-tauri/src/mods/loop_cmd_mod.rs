use log::trace;
use tauri::{AppHandle, Manager};
use crate::{mods::mcu_control_mod, GlobalState};

#[tauri::command]
pub async fn cmd_1kms_loop(app: AppHandle) {
    trace!("1kms loop running");
    let _global_state = app.state::<GlobalState>();
}

#[tauri::command]
pub async fn cmd_50ms_loop(app: AppHandle) {
    trace!("50ms loop running");
    mcu_control_mod::re_pkt_proccess(app).await;
}
