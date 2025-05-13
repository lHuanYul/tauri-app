use std::{fs, path::{Path, PathBuf}, time::Instant};
use log::{error, info};
use plotters::prelude::*;
use rand;
use base64::{engine::general_purpose, Engine as _};
use tauri::{AppHandle, Manager};
use crate::{
    mods::directory_mod::create_file, GlobalState, GENERATE_FOLDER_PATH
};

fn store_folder() -> String {
    return format!("{}/chart", GENERATE_FOLDER_PATH)
}

pub struct ChartState {
    pub data_points: Vec<f64>,
}

impl ChartState {
    pub fn new() -> Self {
        let mut data_points = Vec::new();
        for _i in 0..49 {
            data_points.push(rand::random::<f64>() * 10.0);
        }
        Self { data_points }
    }
}

pub fn line_chart_generate(state: &mut ChartState, chart_name: &str) -> Result<PathBuf, String> {
    let file_path = create_file(store_folder(), &format!("{}.png", chart_name))?;
    
    {
        let drawing_area = BitMapBackend::new(&file_path, (960, 540)).into_drawing_area();
        drawing_area.fill(&WHITE).map_err(|e| e.to_string())?;

        let max_index = state.data_points.len() as f64;
        let mut chart = ChartBuilder::on(&drawing_area)
            .caption("Line Chart (f64)", ("sans-serif", 20))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0f64..(max_index/5.0), 0f64..10f64)
            .map_err(|e| {
                format!("Generate line chart failed: {}", e)
            })?;

        chart.configure_mesh()
            .x_desc("X value")
            .y_desc("Y value")
            .draw()
            .map_err(|e| {
                format!("Generate line chart failed: {}", e)
            })?;

        let data: Vec<(f64, f64)> = state
            .data_points
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64 / 5.0, v))
            .collect();

        chart.draw_series(
                LineSeries::new(data, &BLUE)
            )
            .map_err(|e| {
                format!("Generate line chart failed: {}", e)
            })?;
        
        drawing_area.present()
            .map_err(|e| {
                format!("Generate line chart failed: {}", e)
            })?;
    }
    Ok(file_path)
}

pub fn scatter_chart_generate(state: &mut ChartState, chart_name: &str) -> Result<PathBuf, String> {
    let file_path = create_file(store_folder(), &format!("{}.png", chart_name))?;

    let new_data = rand::random::<f64>() * 10.0;
    state.data_points.push(new_data);
    if state.data_points.len() > 500 {
        state.data_points.remove(0);
    }

    {
        let drawing_area = BitMapBackend::new(&file_path, (960, 540))
            .into_drawing_area();
        drawing_area
            .fill(&WHITE)
            .map_err(|e| {
                format!("Generate line chart failed: {}", e)
            })?;

        let max_index = state.data_points.len() as f64;
        let mut chart = ChartBuilder::on(&drawing_area)
            .caption("Scatter Chart (f64)", ("sans-serif", 20))
            .margin(10)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(0f64..(max_index / 5.0), 0f64..10f64)
            .map_err(|e| {
                format!("Generate line chart failed: {}", e)
            })?;

        chart.configure_mesh()
            .x_desc("X value")
            .y_desc("Y value")
            .draw()
            .map_err(|e| {
                format!("Generate line chart failed: {}", e)
            })?;

        let scatter_data: Vec<(f64, f64)> = state
            .data_points
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64 / 5.0, v))
            .collect();

        // 繪製散點：圓點半徑 5，紅色實心
        chart.draw_series(
                scatter_data
                    .into_iter()
                    .map(|(x, y)| Circle::new((x, y), 5, RED.filled()))
            )
            .map_err(|e| {
                format!("Generate line chart failed: {}", e)
            })?;

        drawing_area.present()
            .map_err(|e| {
                format!("Generate line chart failed: {}", e)
            })?;
    }

    Ok(file_path)
}

pub fn chart_send<P: AsRef<Path>>(chart_path: P) -> Result<String, String> {
    let bytes = fs::read(chart_path.as_ref())
        .map_err(|e| format!("Cannot read file: {}", e))?;
    let b64 = general_purpose::STANDARD.encode(bytes);
    Ok(b64)
}

#[tauri::command]
pub fn chart_generate(app: AppHandle) -> Result<String, String> {
    let start = Instant::now();
    let global_state = app.state::<GlobalState>();
    let mut state = global_state
        .chart_state
        .lock()
        .map_err(|e| format!("鎖住 chart_state 失敗：{}", e))?;
    let path = line_chart_generate(&mut *state, "current_chart").map_err(|e| {
        let message = format!("Chart generate failed: {}", e);
        error!("{}", message);
        message
    })?;
    let b64 = chart_send(&path)?;
    
    info!("耗時: {:.2?}", start.elapsed());

    Ok(b64)
}
