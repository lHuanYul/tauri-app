use log::trace;
use tauri::AppHandle;

#[tauri::command]
pub async fn cmd_1kms_loop(_app: AppHandle) -> Result<String, String> {
    let message = format!("1kms loop finish");
    trace!("{}", message);
    Ok(message)
}

#[tauri::command]
pub async fn cmd_50ms_loop(_app: AppHandle) -> Result<String, String> {
    let message = format!("50ms loop finish");
    trace!("{}", message);
    Ok(message)
}
