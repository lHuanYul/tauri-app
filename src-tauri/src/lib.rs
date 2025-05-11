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
}
use mods::{
    plotter_mod::{
        chart_generate, ChartState
    }, directory_mod, log_mod, loop_cmd_mod::{
        cmd_1kms_loop,
        cmd_50ms_loop,
    }, map_mod::{
        map_load,
        map_save,
    }, matlab_mod::{
        run_engine_plot, MatlabEngine
    }, packet_mod::TrReBuffer, port_async_mod::{
        cmd_available_port_async, cmd_check_port_open_async, cmd_close_port_async, cmd_open_port_async, cmd_serial_test, PortAsyncManager
    }
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
    pub root_path:          SyncMutex<PathBuf>,
    pub main_port:          AsyncMutex<PortAsyncManager>,
    pub transfer_buffer:    AsyncMutex<TrReBuffer>,
    pub receive_buffer:     AsyncMutex<TrReBuffer>,
    pub matlab_engine:      SyncMutex<MatlabEngine>,
    pub chart_state:        SyncMutex<ChartState>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log_mod::init();
    let mut global_state = GlobalState {
        root_path:          SyncMutex::new(PathBuf::new()),
        main_port:          AsyncMutex::new(PortAsyncManager::new()),
        transfer_buffer:    AsyncMutex::new(TrReBuffer::new(5)),
        receive_buffer:     AsyncMutex::new(TrReBuffer::new(5)),
        matlab_engine:      SyncMutex::new(MatlabEngine::new()),
        chart_state:        SyncMutex::new(ChartState::new()),
    };
    global_state.root_path = directory_mod::init();
    
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
        .setup(|_app| {
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
