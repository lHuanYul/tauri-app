use log::{error, info, trace};
use tauri::{AppHandle, Manager};
use crate::{mods::{mcu_control_mod, wifi_mod}, GlobalState};

#[tauri::command]
pub async fn cmd_1kms_loop(app: AppHandle) -> Result<String, String> {
    trace!("1kms loop start");
    let _global_state = app.state::<GlobalState>();
    // let _ = wifi_mod::udp_send_packet().await;
    let _ = wifi_mod::tcp_send_packet().await;
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
