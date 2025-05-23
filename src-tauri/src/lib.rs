use tauri::{AppHandle, Manager};
use std::{path::PathBuf, sync::Mutex as SyncMutex};
use tokio::sync::{Mutex as AsyncMutex};
use log::{error, info, LevelFilter};
use mods::{
    directory_mod, log_mod, loop_cmd_mod, map_mod, matlab_mod::{self}, mcu_control_mod, packet_proc_mod, plotter_mod::{self}, uart_async_mod::{self}, wifi_mod::{self}
};

pub mod mods {
    pub mod plotter_mod;
    pub mod loop_cmd_mod;
    pub mod directory_mod;
    pub mod log_mod;
    pub mod map_mod;
    pub mod matlab_mod;
    pub mod packet_mod;
    pub mod uart_async_mod;
    pub mod mcu_const;
    pub mod mcu_control_mod;
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
    pub main_port:          AsyncMutex<uart_async_mod::PortAsyncManager>,
    pub wifi_tr_re:         AsyncMutex<wifi_mod::WifiReceive>,
    pub transfer_buffer:    AsyncMutex<mcu_control_mod::TrReBuffer>,
    pub receive_buffer:     AsyncMutex<mcu_control_mod::TrReBuffer>,
    pub matlab_engine:      SyncMutex <matlab_mod::MatlabEngine>,
    pub rand_datas:         AsyncMutex<plotter_mod::ChartRandDatas>,
    pub store_datas:        AsyncMutex<mcu_control_mod::DataStore>,
    // pub u32_data_points:  AsyncMutex<ChartDataPoints>,
    // pub u8_data_points:  AsyncMutex<ChartDataPoints>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log_mod::init();
    let global_state = GlobalState {
        root_path:          SyncMutex ::new(PathBuf::new()),
        main_port:          AsyncMutex::new(uart_async_mod::PortAsyncManager::new()),
        wifi_tr_re:         AsyncMutex::new(wifi_mod::WifiReceive::new()),
        transfer_buffer:    AsyncMutex::new(mcu_control_mod::TrReBuffer::new(5)),
        receive_buffer:     AsyncMutex::new(mcu_control_mod::TrReBuffer::new(5)),
        matlab_engine:      SyncMutex ::new(matlab_mod::MatlabEngine::new()),
        rand_datas:         AsyncMutex::new(plotter_mod::ChartRandDatas::new_rand("temp", "disp", 100)),
        store_datas:        AsyncMutex::new(mcu_control_mod::DataStore::new()),
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(global_state)
        .invoke_handler(tauri::generate_handler![
            mytest,
            uart_async_mod::cmd_available_port_async,
            uart_async_mod::cmd_check_port_open_async,
            uart_async_mod::cmd_open_port_async,
            uart_async_mod::cmd_close_port_async,
            mcu_control_mod::cmd_send_spd_stop,
            mcu_control_mod::cmd_send_spd_once,
            mcu_control_mod::cmd_send_spd_start,
            wifi_mod::cmd_wifi_test,
            map_mod::map_load,
            map_mod::map_save,
            plotter_mod::chart_generate,
        ])
        .setup(|app| {
            directory_mod::setup(app);
            wifi_mod::setup(app);
            loop_cmd_mod::init_timer(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn mytest(app: AppHandle) -> Result<(), String> {
    let global_state = app.state::<GlobalState>();
    // let mut state = global_state.matlab_engine.lock().unwrap();
    // let _path = matlab_mod::run_engine_plot(&mut *state, 10.0, 20.0)?;
    let mut store_datas = global_state.store_datas.lock().await;
    let data = store_datas.show_f32(mcu_control_mod::DataStoreSelF32::RightSpeed);
    info!("RightSpeed: {:?}", data);
    let data = store_datas.show_u16(mcu_control_mod::DataStoreSelU16::RightAdc);
    info!("RightAdc: {:?}", data);
    let _ = packet_proc_mod::gen_h_file(app.clone()).map_err(|e| {
        error!("{}", e);
    });
    info!("Ok");
    Ok(())
}
