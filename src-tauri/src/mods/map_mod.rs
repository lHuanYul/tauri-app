use std::fs;
use log::{error, info};
use serde::{Deserialize, Serialize};
use crate::mods::directory_mod::{create_file, path_to_string};

/// 常數：用於 C 程式碼縮排  
/// Constant: indentation for generated C code
const TAB_SPACE: &str = "    ";

/// 型別別名：位置與長度資料型態  
/// Type alias: data types for position and length
type PosType = u16;
type LenType = u32;

/// 常數：儲存檔案的資料夾路徑  
/// Constant: folder path to store generated files
const STORE_FOLDER: &str = "generate/map";
const MAP_BASE_H: &str = include_str!(
    concat!(env!("CARGO_MANIFEST_DIR"), "/generate_base/map/map_base.h")
);

/// 結構：MapConnect，定義 pos 和 len 欄位  
/// Struct: MapConnect with pos and len fields
#[derive(Serialize)]
struct MapConnect {
    pos:    PosType,
    len:    LenType,
}

/// 結構：MapItem，包含 id、名稱與多個連接  
/// Struct: MapItem with id, name, and list of connects
#[derive(Serialize)]
struct MapItem {
    id:         PosType,
    name:       String,
    connect:    Vec<MapConnect>,
}

/// 輸入連接結構：InConnect，用於反序列化 JSON  
/// Input struct: InConnect for JSON deserialization
#[derive(Deserialize)]
struct InConnect {
    pos:    PosType,
    len:    LenType,
}

/// 輸入項目結構：InItem，用於反序列化 JSON  
/// Input struct: InItem for JSON deserialization
#[derive(Deserialize)]
struct InItem {
    id:         PosType,
    name:       String,
    connect:    Vec<InConnect>,
}

/// Tauri 命令：載入現存的 JSON 檔案  
/// Tauri command: load existing JSON file
#[tauri::command]
pub fn map_load() -> Result<String, String> {
    // 建立檔案（存在則不覆寫），並讀取內容 / Create or reuse file, then read its content
    let json_path = create_file(STORE_FOLDER, "map_info.json")?;
    let result = fs::read_to_string(&json_path)
        .map_err(|e| {
            let message = format!("Read JSON failed: {}", e);
            info!("{}", message);
            message
        })?;

    info!("Read JSON succeed");
    Ok(result)
}

/// 解析輸入的 JSON 字串，並生成 C/C++ 初始值陣列和新的 JSON 檔案。  
/// Parses incoming JSON `data` and generates a C initializer array plus a JSON file.
#[tauri::command]
pub fn map_save(
    data: String
) -> Result<String, String> {
    info!("Received data: {}", data);

    // 反序列化 JSON 並轉成 InItem 向量
    // Deserialize JSON into Vec<InItem>
    let items: Vec<InItem> = serde_json::from_str(&data)
        .map_err(|e| {
            let msg = format!("JSON parse error: {}", e);
            error!("{}", msg);
            msg
        })?;
    info!("Parsed {} items", items.len());

    // 準備輸出檔案路徑與緩衝
    // Prepare file paths and buffers
    let c_path = create_file(STORE_FOLDER, "map_info.c")?;
    let h_path = create_file(STORE_FOLDER, "map_base.h")?;
    let json_path = create_file(STORE_FOLDER, "map_info.json")?;
    let mut c_code = String::new();
    let mut json_items: Vec<MapItem> = Vec::with_capacity(items.len());

    // 建立 C 程式碼與 JSON 結構
    // Build C code and JSON structures
    c_code.push_str("#include \"principal/map_base.h\"\n\n");
    c_code.push_str("LOCATION locations_info[] = {\n");
    for item in items.iter() {
        let id = item.id;  // 使用輸入的 id / use id from input
        c_code.push_str(&format!("{}{{{}, {{\n", TAB_SPACE, id));

        let mut connects: Vec<MapConnect> = Vec::with_capacity(item.connect.len());
        for conn in &item.connect {
            let pos: PosType = conn.pos;  // 連接位置 / pos value
            let len: LenType = conn.len;  // 連接長度 / len value

            c_code.push_str(&format!("{}{}{{{}, {}}},\n", TAB_SPACE, TAB_SPACE, pos, len));
            connects.push(MapConnect { pos, len });
        }

        c_code.push_str(&format!("{}}}}},\n", TAB_SPACE));
        json_items.push(MapItem {
            id,
            name: item.name.clone(),
            connect: connects,
        });
    }
    c_code.push_str("};\n");

    // 寫入 C 檔案
    // Write C source file
    fs::write(&h_path, MAP_BASE_H).map_err(|e| format!("Write .h file error: {}", e))?;
    fs::write(&c_path, c_code    ).map_err(|e| format!("Write .c file error: {}", e))?;
    let c_path_str = path_to_string(&c_path).unwrap_or_else(|e| e.clone());
    info!("Wrote C file to {}", c_path_str);

    // 序列化並寫入 JSON 檔案
    // Serialize and write JSON file
    let json_text = serde_json::to_string_pretty(&json_items)
        .map_err(|e| format!("Serialize JSON error: {}", e))?;
    fs::write(&json_path, json_text)
        .map_err(|e| format!("Write JSON file error: {}", e))?;
    let json_path_str = path_to_string(&json_path).unwrap_or_else(|e| e.clone());
    info!("Wrote JSON file to {}", json_path_str);

    Ok(c_path_str)
}
