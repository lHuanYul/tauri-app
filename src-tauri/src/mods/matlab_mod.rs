use libloading::{Library, Symbol};
use libc::{c_char, c_double, c_int};
use log::{debug, error, info};
use std::{ffi::{c_void, CString, CStr}, ptr};
use crate::{mods::directory_mod::{create_file, path_to_string}, MATLAB_LIBENG_DLL_PATH};

const MATLAB_LIBMX_DLL_PATH: &str = "C:/Program Files/MATLAB/R2024b/bin/win64/libmx.dll";
const CHARTS_FOLDER_PATH: &str = "./generate/chart";

/// Opaque MATLAB engine and array types
#[repr(C)] pub struct Engine { _private: [u8;0], _phantom: std::marker::PhantomData<*mut c_void> }
#[repr(C)] pub struct MxArray { _private: [u8;0], _phantom: std::marker::PhantomData<*mut c_void> }

/// Function pointer types matching libeng.dll and libmx.dll exports
type EngOpenFn       = unsafe extern "C" fn(*const c_char) -> *mut Engine;
type EngSetVisibleFn = unsafe extern "C" fn(*mut Engine, c_int) -> c_int;
type EngEvalStringFn = unsafe extern "C" fn(*mut Engine, *const c_char) -> c_int;
type EngGetVarFn     = unsafe extern "C" fn(*mut Engine, *const c_char) -> *mut MxArray;
type MxGetStringFn   = unsafe extern "C" fn(*mut MxArray, *mut c_char, usize) -> c_int;
type MxGetPrFn       = unsafe extern "C" fn(*mut MxArray) -> *mut c_double;

/// Wrapper around MATLAB Engine with graceful fallback
pub struct MatlabEngine {
    eng_lib: Option<Library>,
    mx_lib: Option<Library>,
    eng_open: Option<EngOpenFn>,
    eng_set_visible: Option<EngSetVisibleFn>,
    eng_eval_string: Option<EngEvalStringFn>,
    eng_get_var: Option<EngGetVarFn>,
    mx_get_string: Option<MxGetStringFn>,
    mx_get_pr: Option<MxGetPrFn>,
    engine: *mut Engine,
    enabled: bool,
}

impl MatlabEngine {
    pub fn new() -> Self {
        info!("嘗試動態載入 MATLAB 引擎庫…");
        let mut me = MatlabEngine {
            eng_lib: None,
            mx_lib: None,
            eng_open: None,
            eng_set_visible: None,
            eng_eval_string: None,
            eng_get_var: None,
            mx_get_string: None,
            mx_get_pr: None,
            engine: ptr::null_mut(),
            enabled: false,
        };
        // 載入引擎庫 libeng.dll
        match unsafe { Library::new(MATLAB_LIBENG_DLL_PATH) } {
            Ok(lib) => {
                let (open_fn, set_vis_fn, eval_fn, get_var_fn) = unsafe {
                    let eng_open_sym: Symbol<EngOpenFn> = lib.get(b"engOpen\0").expect("找不到 engOpen");
                    let eng_set_visible_sym: Symbol<EngSetVisibleFn> = lib.get(b"engSetVisible\0").expect("找不到 engSetVisible");
                    let eng_eval_string_sym: Symbol<EngEvalStringFn> = lib.get(b"engEvalString\0").expect("找不到 engEvalString");
                    let eng_get_var_sym: Symbol<EngGetVarFn> = lib.get(b"engGetVariable\0").expect("找不到 engGetVariable");
                    (*eng_open_sym, *eng_set_visible_sym, *eng_eval_string_sym, *eng_get_var_sym)
                };
                me.eng_lib = Some(lib);
                me.eng_open = Some(open_fn);
                me.eng_set_visible = Some(set_vis_fn);
                me.eng_eval_string = Some(eval_fn);
                me.eng_get_var = Some(get_var_fn);
            }
            Err(e) => {
                error!("載入 libeng.dll 失敗，MATLAB 功能將略過：  {}", e);
            }
        }
        // 載入資料庫 libmx.dll
        match unsafe { Library::new(MATLAB_LIBMX_DLL_PATH) } {
            Ok(lib) => {
                let (get_str_fn, get_pr_fn) = unsafe {
                    let mx_get_string_sym: Symbol<MxGetStringFn> = lib.get(b"mxGetString\0").expect("找不到 mxGetString");
                    let mx_get_pr_sym: Symbol<MxGetPrFn> = lib.get(b"mxGetPr\0").expect("找不到 mxGetPr");
                    (*mx_get_string_sym, *mx_get_pr_sym)
                };
                me.mx_lib = Some(lib);
                me.mx_get_string = Some(get_str_fn);
                me.mx_get_pr = Some(get_pr_fn);
            }
            Err(e) => {
                error!("載入 libmx.dll 失敗，MATLAB 功能將略過：  {}", e);
            }
        }
        // 初始化引擎
        if let Some(open_fn) = me.eng_open {
            let eng = unsafe { open_fn(ptr::null()) };
            if eng.is_null() {
                error!("engOpen 回傳 null，MATLAB 功能停用");
            } else {
                me.engine = eng;
                me.enabled = true;
                if let Some(set_vis_fn) = me.eng_set_visible {
                    unsafe { set_vis_fn(eng, 0); }
                }
                info!("成功啟動 MATLAB 引擎");
            }
        }
        me
    }

    /// 在引擎不可用時跳過執行
    pub fn eval(&self, cmd: &str) -> Result<(), String> {
        if !self.enabled {
            debug!("MATLAB disabled: skip eval `{}`", cmd);
            return Ok(());
        }
        let c_cmd = CString::new(cmd).unwrap();
        let status = unsafe { (self.eng_eval_string.unwrap())(self.engine, c_cmd.as_ptr()) };
        if status != 0 {
            let msg = format!("MATLAB eval 失敗，status = {}", status);
            error!("{}", msg);
            return Err(msg);
        }
        Ok(())
    }

    /// 取得 double 變數值
    pub fn get_var_f64(&self, name: &str) -> Result<f64, String> {
        if !self.enabled {
            debug!("MATLAB disabled: skip get_var_f64 `{}`", name);
            return Err("MATLAB 未啟動".into());
        }
        let c_name = CString::new(name).unwrap();
        let arr = unsafe { (self.eng_get_var.unwrap())(self.engine, c_name.as_ptr()) };
        if arr.is_null() {
            let err = format!("找不到變數 `{}`", name);
            error!("{}", err);
            return Err(err);
        }
        let pr = unsafe { (self.mx_get_pr.unwrap())(arr) };
        if pr.is_null() {
            let err = "mxGetPr 回傳 null".to_string();
            error!("{}", err);
            return Err(err);
        }
        let value = unsafe { *pr };
        info!("取得 {} = {}", name, value);
        Ok(value)
    }

    /// 取得字串型變數
    pub fn get_var_string(&self, name: &str) -> Result<String, String> {
        if !self.enabled {
            debug!("MATLAB disabled: skip get_var_string `{}`", name);
            return Err("MATLAB 未啟動".into());
        }
        let c_name = CString::new(name).unwrap();
        let arr = unsafe { (self.eng_get_var.unwrap())(self.engine, c_name.as_ptr()) };
        if arr.is_null() {
            let err = format!("找不到變數 `{}`", name);
            error!("{}", err);
            return Err(err);
        }
        let mut buf = vec![0 as c_char; 1024];
        let status = unsafe { (self.mx_get_string.unwrap())(arr, buf.as_mut_ptr(), buf.len()) };
        if status != 0 {
            let msg = format!("MATLAB mxGetString 失敗，status = {}", status);
            error!("{}", msg);
            return Err(msg);
        }
        let s = unsafe { CStr::from_ptr(buf.as_ptr()) }.to_string_lossy().into_owned();
        Ok(s)
    }
}

unsafe impl Send for MatlabEngine {}
unsafe impl Sync for MatlabEngine {}

pub fn run_engine_plot(
    engine: &mut MatlabEngine,
    a: f64,
    b: f64,
) -> Result<String, String> {
    //let file_name = format!("chart_{}.png", chrono::Local::now().format("%Y%m%d%H%M%S"));
    let file_path = create_file(CHARTS_FOLDER_PATH, &format!("matlab.png")).map_err(|e| {
        error!("{}", e);
    }).unwrap();
    let file_path_str= path_to_string(file_path)?;

    info!("在 MATLAB 裡處理參數 a={}, b={}", a, b);
    // 先做簡單運算
    engine.eval(&format!("sum = {} + {};", a, b))?;
    engine.eval(&format!("sub = {} - {};", a, b))?;

    // 產生示例資料
    engine.eval("x = 0:0.1:2*pi;")?;
    engine.eval("y = sin(x) + randn(size(x))*0.1;")?;

    // 不顯示視窗
    engine.eval("fig = figure('Visible','off');")?;
    // 畫折線
    engine.eval("plot(x, y, '-o', 'LineWidth', 1.5);")?;
    engine.eval("xlabel('x'); ylabel('sin(x) + noise'); title('MATLAB + Rust Engine Plot');")?;
    // 存成 PNG
    engine.eval(&format!("print(fig, '{}', '-dpng', '-r150');", file_path_str))?;

    // 關閉 figure
    engine.eval("close(fig);")?;

    info!("已生成圖檔：{}", file_path_str);
    Ok(file_path_str)
}

pub fn get_engine_cwd(engine: &mut MatlabEngine,) -> Result<String, String> {
    engine.eval("cwd = pwd;")?;
    
    let path = engine.get_var_string("cwd")?;
    let _path = path.replace('\\', "/");

    info!("MATLAB Engine working directory: {}", _path);
    Ok(path)
}
