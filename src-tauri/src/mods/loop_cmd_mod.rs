use log::trace;
use tauri::{AppHandle, Manager};
use crate::{mods::{mcu_control_mod, wifi_mod}, GlobalState};

#[tauri::command]
pub async fn cmd_1kms_loop(app: AppHandle) {
    trace!("1kms loop running");
    let _global_state = app.state::<GlobalState>();
    // let _ = wifi_mod::udp_send_packet().await;
    let _ = wifi_mod::tcp_send_packet().await;
}

#[tauri::command]
pub async fn cmd_50ms_loop(app: AppHandle) {
    trace!("50ms loop running");
    mcu_control_mod::receive_packet_proccess(app).await;
}
