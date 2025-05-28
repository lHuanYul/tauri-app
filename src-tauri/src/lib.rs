use tauri::AppHandle;
use std::{path::PathBuf, sync::Mutex as SyncMutex};
use tokio::sync::{Mutex as AsyncMutex};
use log::LevelFilter;
use mods::{
    directory_mod, log_mod, loop_cmd_mod, map_mod, matlab_mod::{self}, mcu_control_mod, mcu_store_mod, plotter_mod::{self}, tauri_test_mod, uart_mod::{self}, uart_packet_proc_mod, wifi_mod::{self}, wifi_packet_mod, wifi_packet_proc_mod
};

pub mod mods {
    pub mod log_mod;
    pub mod tauri_test_mod;
    pub mod directory_mod;
    pub mod loop_cmd_mod;
    pub mod user_vec_mod;
    pub mod uart_packet_mod;
    pub mod uart_packet_proc_mod;
    pub mod uart_mod;
    pub mod wifi_mod;
    pub mod wifi_packet_mod;
    pub mod wifi_packet_proc_mod;
    pub mod mcu_const;
    pub mod mcu_control_mod;
    pub mod mcu_store_mod;
    pub mod plotter_mod;
    pub mod map_mod;
    pub mod matlab_mod;
}

/// Set const
/// ```
/// LevelFilter::{Trace, Debug, Info, Warn, Error, Off}
/// ```
pub const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;
pub const ROOT_GEN_FILES_FOLDER: &str = "generate";
pub const BASE_GEN_FILES_FOLDER: &str = "base";
pub const MAP_GEN_FILES_FOLDER: &str = "map";
pub const GENERATE_BASE_FOLDER_PATH: &str = "generate_base";
pub const MATLAB_LIBENG_DLL_PATH: &str = "C:/Program Files/MATLAB/R2024b/bin/win64/libeng.dll";

pub struct GlobalState {
    pub root_path:          SyncMutex <PathBuf>,
    pub uart_manager:       AsyncMutex<uart_mod::UartAsyncManager>,
    pub uart_recv_buffer:   AsyncMutex<uart_packet_proc_mod::UartTrceBuffer>,
    pub uart_traf_buffer:   AsyncMutex<uart_packet_proc_mod::UartTrceBuffer>,
    pub wifi_manager:       AsyncMutex<wifi_mod::WifiAsyncManager>,
    pub wifi_tcp_recv_buf:  AsyncMutex<wifi_packet_proc_mod::WifiTrceBuffer>,
    pub wifi_tcp_traf_buf:  AsyncMutex<wifi_packet_proc_mod::WifiTrceBuffer>,
    pub wifi_udp_recv_buf:  AsyncMutex<wifi_packet_proc_mod::WifiTrceBuffer>,
    pub wifi_udp_traf_buf:  AsyncMutex<wifi_packet_proc_mod::WifiTrceBuffer>,
    pub store_datas:        AsyncMutex<mcu_store_mod::DataStore>,
    pub matlab_engine:      SyncMutex <matlab_mod::MatlabEngine>,
    pub rand_datas:         AsyncMutex<plotter_mod::ChartRandDatas>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log_mod::init();
    let global_state = GlobalState {
        root_path:          SyncMutex ::new(PathBuf::new()),
        uart_manager:       AsyncMutex::new(uart_mod::UartAsyncManager::new()),
        uart_recv_buffer:   AsyncMutex::new(uart_packet_proc_mod::UartTrceBuffer::new(10)),
        uart_traf_buffer:   AsyncMutex::new(uart_packet_proc_mod::UartTrceBuffer::new(10)),
        wifi_manager:       AsyncMutex::new(wifi_mod::WifiAsyncManager::new()),
        wifi_tcp_recv_buf:  AsyncMutex::new(wifi_packet_proc_mod::WifiTrceBuffer::new(10, wifi_packet_mod::WIFI_TCP_PACKET_MAX_SIZE)),
        wifi_tcp_traf_buf:  AsyncMutex::new(wifi_packet_proc_mod::WifiTrceBuffer::new(10, wifi_packet_mod::WIFI_TCP_PACKET_MAX_SIZE)),
        wifi_udp_recv_buf:  AsyncMutex::new(wifi_packet_proc_mod::WifiTrceBuffer::new(10, wifi_packet_mod::WIFI_UDP_PACKET_MAX_SIZE)),
        wifi_udp_traf_buf:  AsyncMutex::new(wifi_packet_proc_mod::WifiTrceBuffer::new(10, wifi_packet_mod::WIFI_UDP_PACKET_MAX_SIZE)),
        store_datas:        AsyncMutex::new(mcu_store_mod::DataStore::new(100)),
        matlab_engine:      SyncMutex ::new(matlab_mod::MatlabEngine::new()),
        rand_datas:         AsyncMutex::new(plotter_mod::ChartRandDatas::new_rand("temp", "disp", 100)),
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(global_state)
        .invoke_handler(tauri::generate_handler![
            tauri_test_mod::mytest,
            uart_mod::cmd_available_port_async,
            uart_mod::cmd_check_port_open_async,
            uart_mod::cmd_open_port_async,
            uart_mod::cmd_close_port_async,
            mcu_control_mod::cmd_send_spd_stop,
            mcu_control_mod::cmd_send_spd_once,
            mcu_control_mod::cmd_send_spd_start,
            wifi_mod::cmd_wifi_start,
            map_mod::map_load,
            map_mod::map_save,
            plotter_mod::chart_generate,
        ])
        .setup(|app| {
            setup(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
fn setup(app: AppHandle) {
    directory_mod::setup(app.clone());
    loop_cmd_mod::setup(app);
}
