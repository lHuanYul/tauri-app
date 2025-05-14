use std::{fs, path::{Path, PathBuf}, time::Instant};
use log::{error, info};
use plotters::prelude::*;
use rand;
use base64::{engine::general_purpose, Engine};
use tauri::{AppHandle, Manager};
use crate::{
    mods::directory_mod::{self}, GlobalState, GENERATE_FOLDER_PATH
};

fn store_folder() -> String {
    return format!("{}/chart", GENERATE_FOLDER_PATH)
}

/// ChartDataPoints: 儲存資料點並維持最大長度 <br>
/// Stores data points and maintains a maximum length
pub struct ChartDataPoints {
    data_points: Vec<f32>,
    max_length: usize,
}
impl ChartDataPoints {
    /// 建立新的 ChartDataPoints
    /// Create a new ChartDataPoints with specified capacity
    pub fn new(max_length: usize) -> Self {
        Self {
            data_points: Vec::new(),
            max_length,
        }
    }

    /// 建立並推入一個隨機值
    /// Create a new ChartDataPoints and push one random value
    pub fn new_rand(max_length: usize) -> Self {
        let mut new = Self::new(max_length);
        for _ in 0..new.max_length {
            new.data_points.push(rand::random::<f32>() * 10.0);
        }
        new
    }

    /// 取得所有資料點的複本
    /// Return a clone of all data points
    pub fn data(&self) -> Vec<f32> {
        self.data_points.clone()
    }

    /// 推入一個新值，並移除超額的最舊值
    /// Push a new value and remove oldest if over capacity
    pub fn push(&mut self, value: f32) {
        self.data_points.push(value);
        // 如果超過預設長度，移除最前面的元素
        if self.data_points.len() > self.max_length {
            self.data_points.remove(0);
        }
    }
}

pub fn line_chart_generate(
    chart_data_points: &mut ChartDataPoints,
    chart_name: &str
) -> Result<PathBuf, String> {
    let file_path = directory_mod::create_file(store_folder(), &format!("{}.png", chart_name))?;
    let _file_path = file_path.clone();

    let drawing_area = BitMapBackend::new(&_file_path, (960, 540)).into_drawing_area();
    drawing_area.fill(&WHITE).map_err(|e| e.to_string())?;

    let max_index = chart_data_points.data_points.len() as f32;
    let mut chart = ChartBuilder::on(&drawing_area)
        .caption("Line Chart (f32)", ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(30).y_label_area_size(30)
        .build_cartesian_2d(0f32..(max_index/5.0), 0f32..10f32)
        .map_err(|e| format!("Generate line chart failed: {}", e) )?;

    chart.configure_mesh()
        .x_desc("X value").y_desc("Y value")
        .draw()
        .map_err(|e| format!("Generate line chart failed: {}", e) )?;

    let data: Vec<(f32, f32)> = chart_data_points.data_points
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f32 / 5.0, v))
        .collect();

    chart.draw_series(LineSeries::new(data, &BLUE))
        .map_err(|e| format!("Generate line chart failed: {}", e) )?;
    
    drawing_area.present()
        .map_err(|e| format!("Generate line chart failed: {}", e) )?;

    Ok(file_path)
}

pub fn scatter_chart_generate(
    chart_data_points: &mut ChartDataPoints,
    chart_name: &str
) -> Result<PathBuf, String> {
    let file_path = directory_mod::create_file(store_folder(), &format!("{}.png", chart_name))?;
    let _file_path = file_path.clone();

    let drawing_area = BitMapBackend::new(&_file_path, (960, 540))
        .into_drawing_area();
    drawing_area
        .fill(&WHITE)
        .map_err(|e| format!("Generate line chart failed: {}", e) )?;

    let max_index = chart_data_points.data_points.len() as f32;
    let mut chart = ChartBuilder::on(&drawing_area)
        .caption("Scatter Chart (f32)", ("sans-serif", 20))
        .margin(10)
        .x_label_area_size(30).y_label_area_size(30)
        .build_cartesian_2d(0f32..(max_index / 5.0), 0f32..10f32)
        .map_err(|e| format!("Generate line chart failed: {}", e) )?;

    chart.configure_mesh()
        .x_desc("X value").y_desc("Y value")
        .draw()
        .map_err(|e| format!("Generate line chart failed: {}", e) )?;

    let scatter_data: Vec<(f32, f32)> = chart_data_points
        .data_points
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f32 / 5.0, v))
        .collect();

    // 繪製散點：圓點半徑 5，紅色實心
    chart.draw_series(
            scatter_data
                .into_iter()
                .map(|(x, y)| Circle::new((x, y), 5, RED.filled()))
        )
        .map_err(|e| format!("Generate line chart failed: {}", e) )?;

    drawing_area.present()
        .map_err(|e| format!("Generate line chart failed: {}", e) )?;

    Ok(file_path)
}

pub fn chart_send<P: AsRef<Path>>(chart_path: P) -> Result<String, String> {
    let bytes = fs::read(chart_path.as_ref())
        .map_err(|e| format!("Cannot read file: {}", e))?;
    let b64 = general_purpose::STANDARD.encode(bytes);
    Ok(b64)
}

#[tauri::command]
pub async fn chart_generate(app: AppHandle) -> Result<String, String> {
    let start = Instant::now();

    let global_state = app.state::<GlobalState>();
    let mut state = global_state.speed_data_points.lock().await;
    let path = line_chart_generate(&mut *state, "current_chart").map_err(|e| {
        error!("{}", e);
        e
    })?;
    let b64 = chart_send(&path).map_err(|e| {
        error!("{}", e);
        e
    })?;
    
    info!("耗時: {:.2?}", start.elapsed());
    Ok(b64)
}
