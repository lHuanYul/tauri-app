use tauri::{AppHandle, Manager};
use std::{path::PathBuf, sync::Mutex as SyncMutex};
use tokio::sync::{Mutex as AsyncMutex};
use log::{info, LevelFilter};
use mods::{
    directory_mod, log_mod, loop_cmd_mod, map_mod, matlab_mod::{self}, mcu_control_mod, mcu_store_mod, packet_proc_mod, plotter_mod::{self}, uart_mod::{self}, wifi_mod::{self}
};

pub mod mods {
    pub mod plotter_mod;
    pub mod loop_cmd_mod;
    pub mod directory_mod;
    pub mod log_mod;
    pub mod map_mod;
    pub mod matlab_mod;
    pub mod packet_mod;
    pub mod uart_mod;
    pub mod mcu_const;
    pub mod mcu_control_mod;
    pub mod mcu_store_mod;
    pub mod wifi_mod;
    pub mod box_error_mod;
    pub mod packet_proc_mod;
}

/// Set const
/// ```
/// LevelFilter::{Trace, Debug, Info, Warn, Error, Off}
/// ```
pub const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;
pub const GENERATE_FOLDER_PATH: &str = "generate";
pub const GENERATE_BASE_FOLDER_PATH: &str = "generate_base";
pub const MATLAB_LIBENG_DLL_PATH: &str = "C:/Program Files/MATLAB/R2024b/bin/win64/libeng.dll";

pub struct GlobalState {
    pub root_path:          SyncMutex <PathBuf>,
    pub uart_manager:       AsyncMutex<uart_mod::UartAsyncManager>,
    pub uart_traf_buffer:   AsyncMutex<packet_proc_mod::TrceBuffer>,
    pub uart_recv_buffer:   AsyncMutex<packet_proc_mod::TrceBuffer>,
    pub wifi_manager:       AsyncMutex<wifi_mod::WifiAsyncManager>,
    pub matlab_engine:      SyncMutex <matlab_mod::MatlabEngine>,
    pub rand_datas:         AsyncMutex<plotter_mod::ChartRandDatas>,
    pub store_datas:        AsyncMutex<mcu_store_mod::DataStore>,
    // pub u32_data_points:  AsyncMutex<ChartDataPoints>,
    // pub u8_data_points:  AsyncMutex<ChartDataPoints>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log_mod::init();
    let global_state = GlobalState {
        root_path:          SyncMutex ::new(PathBuf::new()),
        uart_manager:       AsyncMutex::new(uart_mod::UartAsyncManager::new()),
        uart_traf_buffer:   AsyncMutex::new(packet_proc_mod::TrceBuffer::new(10)),
        uart_recv_buffer:   AsyncMutex::new(packet_proc_mod::TrceBuffer::new(10)),
        wifi_manager:       AsyncMutex::new(wifi_mod::WifiAsyncManager::new()),
        matlab_engine:      SyncMutex ::new(matlab_mod::MatlabEngine::new()),
        rand_datas:         AsyncMutex::new(plotter_mod::ChartRandDatas::new_rand("temp", "disp", 100)),
        store_datas:        AsyncMutex::new(mcu_store_mod::DataStore::new(100)),
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(global_state)
        .invoke_handler(tauri::generate_handler![
            mytest,
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
            directory_mod::setup(app);
            loop_cmd_mod::init_timer(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn mytest(app: AppHandle) -> Result<(), String> {
    let _global_state = app.state::<GlobalState>();
    // let mut state = _global_state.matlab_engine.lock().unwrap();
    // let _path = matlab_mod::run_engine_plot(&mut *state, 10.0, 20.0)?;
    let store_datas = _global_state.store_datas.lock().await;
    let data = store_datas.get(mcu_store_mod::DataType::MotorRight(mcu_store_mod::MotorDataType::SpeedPresent));
    info!("RightSpeed: {:?}", data);
    let data = store_datas.get(mcu_store_mod::DataType::MotorRight(mcu_store_mod::MotorDataType::AdcValue));
    info!("RightAdc: {:?}", data);
    // let _ = packet_proc_mod::gen_h_file(app.clone()).map_err(|e| {
    //     error!("{}", e);
    // });
    // let _ = wifi_mod::cmd_wifi_start(app.clone()).await;
    info!("Ok");
    Ok(())
}
