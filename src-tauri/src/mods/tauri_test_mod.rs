use log::info;
use tauri::{AppHandle, Manager};
use crate::{mods::mcu_store_mod, GlobalState};

#[tauri::command]
pub async fn mytest(app: AppHandle) -> Result<(), String> {
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
