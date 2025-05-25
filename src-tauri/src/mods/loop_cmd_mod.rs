use std::time::Duration;
use tauri::AppHandle;
use tokio::time::interval;
use crate::mods::uart_packet_proc_mod;

pub fn setup(app: AppHandle) {
    let app_10ms = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut ticker = interval(Duration::from_millis(10));
        loop {
            uart_packet_proc_mod::re_pkt_proccess(app_10ms.clone()).await;
            ticker.tick().await;
        }
    });
}
