use std::time::Duration;
use tauri::AppHandle;
use tokio::time::interval;
use crate::mods::mcu_control_mod;

pub fn init_timer(app: AppHandle) {
    let app_10ms = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut ticker = interval(Duration::from_millis(10));
        loop {
            mcu_control_mod::re_pkt_proccess(app_10ms.clone()).await;
            ticker.tick().await;
        }
    });
}
