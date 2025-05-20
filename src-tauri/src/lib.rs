use tauri::{AppHandle, Manager};
use std::{path::PathBuf, sync::Mutex as SyncMutex};
use tokio::sync::Mutex as AsyncMutex;
use log::LevelFilter;
pub mod mods {
    pub mod plotter_mod;
    pub mod loop_cmd_mod;
    pub mod directory_mod;
    pub mod log_mod;
    pub mod map_mod;
    pub mod matlab_mod;
    pub mod packet_mod;
    pub mod port_async_mod;
    pub mod mcu_control_mod;
    pub mod wifi_mod;
    pub mod box_error_mod;
}
use mods::{
    directory_mod, log_mod, loop_cmd_mod::{cmd_1kms_loop, cmd_50ms_loop,
    }, map_mod::{map_load,  map_save,
    }, matlab_mod::{self, run_engine_plot
    },packet_mod, plotter_mod::{self, chart_generate
    }, port_async_mod::{self, cmd_available_port_async, cmd_check_port_open_async, cmd_close_port_async, cmd_open_port_async, cmd_serial_test
    }, wifi_mod,
};

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
    pub main_port:          AsyncMutex<port_async_mod::PortAsyncManager>,
    pub wifi_tr_re:         AsyncMutex<wifi_mod::WifiReceive>,
    pub transfer_buffer:    AsyncMutex<packet_mod::TrReBuffer>,
    pub receive_buffer:     AsyncMutex<packet_mod::TrReBuffer>,
    pub matlab_engine:      SyncMutex <matlab_mod::MatlabEngine>,
    pub rand_datas:         AsyncMutex<plotter_mod::ChartRandDatas>,
    pub speed_datas:        AsyncMutex<plotter_mod::ChartSpeedDatas>,
    // pub u32_data_points:  AsyncMutex<ChartDataPoints>,
    // pub u8_data_points:  AsyncMutex<ChartDataPoints>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log_mod::init();
    let global_state = GlobalState {
        root_path:          SyncMutex ::new(PathBuf::new()),
        main_port:          AsyncMutex::new(port_async_mod::PortAsyncManager::new()),
        wifi_tr_re:         AsyncMutex::new(wifi_mod::WifiReceive::new()),
        transfer_buffer:    AsyncMutex::new(packet_mod::TrReBuffer::new(5)),
        receive_buffer:     AsyncMutex::new(packet_mod::TrReBuffer::new(5)),
        matlab_engine:      SyncMutex ::new(matlab_mod::MatlabEngine::new()),
        rand_datas:         AsyncMutex::new(plotter_mod::ChartRandDatas::new_rand("temp", "disp", 100)),
        speed_datas:        AsyncMutex::new(plotter_mod::ChartSpeedDatas::new("speed", "Speed", 100)),
    };
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(global_state)
        .invoke_handler(tauri::generate_handler![
            mytest,
            cmd_1kms_loop,
            cmd_50ms_loop,
            cmd_available_port_async,
            cmd_check_port_open_async,
            cmd_open_port_async,
            cmd_close_port_async,
            cmd_serial_test,
            map_load,
            map_save,
            chart_generate,
        ])
        .setup(|app| {
            directory_mod::setup(app);
            wifi_mod::setup(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn mytest(app: AppHandle) -> Result<String, String> {
    let global_state = app.state::<GlobalState>();
    let mut state = global_state.matlab_engine.lock().unwrap();
    let _path = run_engine_plot(&mut *state, 10.0, 20.0)?;
    Ok("OK".to_string())
}
