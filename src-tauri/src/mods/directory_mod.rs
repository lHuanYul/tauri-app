use std::{env, error::Error, fs::{self, File}, path::{Path, PathBuf}};
use log::{debug, info, warn};
use tauri::{App, Manager};
use crate::GlobalState;

/// 初始化工作目錄並回傳其 Mutex 包裝的 PathBuf <br>
/// Initialize the working directory and return it wrapped in a Mutex
pub fn setup(app: &mut App) {
    let global_state = app.state::<GlobalState>();
    let mut root_path = global_state.root_path.lock().unwrap();
    // 嘗試取得當前工作目錄，失敗時以 "." 作為預設 / Try to get current dir, fallback to "."
    let working_directory  = get_working_directory().unwrap_or_else(|_e| {
        warn!("Change to default: .");
        PathBuf::from(".")
    });
    info!("Rust working directory: {:?}", working_directory);
    *root_path = working_directory;
}

/// 取得當前工作目錄並回傳 PathBuf <br>
/// Get current working directory as PathBuf
pub fn get_working_directory() -> Result<PathBuf, Box<dyn Error>> {
    let working_directory = env::current_dir().map_err(|e| {
        format!("Get working directory failed: {}", e)
    })?;
    debug!("Current working directory: {:?}", working_directory);
    Ok(working_directory)
}

/// 將 Path 轉為標準化字串 (替換反斜線為正斜線) <br>
/// Convert a Path to a normalized String (backslashes to forward slashes)
pub fn path_to_string<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn Error>> {
    let path_ref = path.as_ref();
    if let Some(path_str) = path_ref.to_str() {
        // 直接替換 Windows 路徑分隔 / Replace Windows separators
        Ok(path_str.replace('\\', "/"))
    } else {
        // 若 to_str 失敗，使用 lossy 轉換並警告 / Fallback to lossy conversion
        let result = path_ref.to_string_lossy().into_owned();
        warn!("Convert path to string failed: {result}");
        Err(result.into())
    }
}

/// 在指定資料夾下建立檔案，已存在則不覆寫 <br>
/// Create a file in the specified folder, no overwrite if exists
pub fn create_file<P: AsRef<Path>>(folder_path: P, file_name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let folder = folder_path.as_ref().to_path_buf();
    fs::create_dir_all(&folder)?;
    let file_path = folder.join(format!("{file_name}"));

    // 檢查檔案是否已存在 / Check if file exists
    if file_path.exists() && file_path.is_file() {
        let message = format!("File already exists: {:?}", file_path);
        debug!("{}", message);
        return Ok(file_path);
    }
    // 嘗試建立新檔案 / Create new file
    File::create(&file_path).map_err(|e|
        format!("Fail to create file '{:?}': {}", file_path, e)
    )?;
    debug!("Created file: {:?}", file_path);
    Ok(file_path)
}
