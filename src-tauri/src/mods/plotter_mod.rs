use std::{fs, path::PathBuf};
use log::error;
use num_traits::ToPrimitive;
use plotters::prelude::*;
use rand::{self, Rng};
use base64::{engine::general_purpose, Engine};
use tauri::{AppHandle, Manager};
use crate::{mods::directory_mod, GlobalState, GENERATE_FOLDER_PATH};

fn store_folder() -> String {
    format!("{}/chart", GENERATE_FOLDER_PATH)
}

/// ChartDataPoints: 儲存資料點並維持最大長度
/// Stores data points and maintains a maximum length
pub struct ChartRandDatas {
    file_path: PathBuf,
    display_name: String,
    data_points_1: Vec<i32>,
    data_points_2: Vec<i32>,
    max_length: usize,
}
impl ChartRandDatas {
    /// 建立新的 ChartDataPoints
    pub fn new(name: &str, display_name: &str, max_length: usize) -> Self {
        let file_path = directory_mod::create_file(store_folder(), &format!("{name}.png"))
            .map_err(|e| error!("{}", e)).unwrap();
        Self {
            file_path,
            display_name: display_name.to_owned(),
            data_points_1: Vec::new(),
            data_points_2: Vec::new(),
            max_length,
        }
    }

    /// 建立並推入隨機值
    pub fn new_rand(name: &str, display_name: &str, max_length: usize) -> Self {
        let mut new = Self::new(name, display_name, max_length);
        for _ in 0..new.max_length {
            new.data_points_1.push(rand::rng().random_range(-10..=10));
            new.data_points_2.push(rand::rng().random_range(-10..=10));
        }
        new
    }

    pub fn path(&self) -> PathBuf {
        self.file_path.clone()
    }

    /// 推入一個新值，並移除超額的最舊值
    pub fn push(&mut self, value: i32) {
        self.data_points_1.push(value);
        if self.data_points_1.len() > self.max_length {
            self.data_points_1.remove(0);
        }
    }

    pub fn line_chart_generate(&self) -> Result<(), String> {
        let root_size_x = 2160;
        let root_size_y = 1215;
        // 1. 建立繪圖區並填背景
        let root = BitMapBackend::new(&self.file_path, (root_size_x, root_size_y))
            .into_drawing_area();
        root.fill(&WHITE).map_err(|e| e.to_string())?;

        let legend_area_size = 100;
        let (data_area, legend_area) = 
            root.split_vertically((root_size_y - legend_area_size) as u32);

        // 3. 留邊
        let root_mergin = 40;
        let data_area   = data_area  .margin(root_mergin, 0, root_mergin, root_mergin);
        let legend_area = legend_area.margin(0, root_mergin, root_mergin, root_mergin);

        // 4. 計算座標範圍
        let data_min_x = 0 as i32;
        let data_max_x = self.max_length.to_i32().unwrap();
        let data_all_vals = self.data_points_1.iter().chain(self.data_points_2.iter()).cloned();
        let data_min_y = data_all_vals.clone().min().unwrap_or(0) - 1;
        let data_max_y = data_all_vals.max().unwrap_or(0) + 1;

        // 5. 建立 ChartContext
        let mut chart = ChartBuilder::on(&data_area)
            .caption(&self.display_name, ("Serif", 40))
            .x_label_area_size(60)
            .y_label_area_size(80)
            .build_cartesian_2d(data_min_x..data_max_x, data_min_y..data_max_y)
            .map_err(|e| e.to_string())?;

        // 6. 繪製網格與軸
        chart.configure_mesh()
            .x_desc("Index")
            .y_desc("Value")
            .axis_desc_style(("Serif", 30))
            .label_style(("Serif", 20))
            .draw()
            .map_err(|e| e.to_string())?;

        // 7. 準備資料點
        let points1: Vec<(i32, i32)> = self.data_points_1.iter()
            .enumerate().map(|(i, &v)| (i as i32, v)).collect();
        let points2: Vec<(i32, i32)> = self.data_points_2.iter()
            .enumerate().map(|(i, &v)| (i as i32, v)).collect();

        let data_dot_size = 2;
        // 8. 繪製序列 1 (紅色)
        chart.draw_series(LineSeries::new(points1.clone(), &RED))
            .map_err(|e| e.to_string())?;
        chart.draw_series(
            points1.iter().map(|&p| Circle::new(p, data_dot_size, RED.filled()))
        ).map_err(|e| e.to_string())?;
        // 9. 繪製序列 2 (藍色)
        chart.draw_series(LineSeries::new(points2.clone(), &BLUE))
            .map_err(|e| e.to_string())?;
        chart.draw_series(
            points2.iter().map(|&p| Circle::new(p, data_dot_size, BLUE.filled()))
        ).map_err(|e| e.to_string())?;

        // 10. 手動繪製圖例於 legend_area
        legend_area.fill(&WHITE).map_err(|e| e.to_string())?;
        // 準備文字樣式
        let text_style = ("Serif", 25).into_text_style(&legend_area);
        // Series 1 標示
        let legend_line_size = 40;
        let legend_font_dist = 10;
        legend_area.draw(&PathElement::new(vec![(120, 0), (120 + legend_line_size, 0)], &RED,))
            .map_err(|e| e.to_string())?;
        legend_area.draw_text("Series 1", &text_style, (110, 0 + legend_font_dist),)
            .map_err(|e| e.to_string())?;
        // Series 2 標示
        legend_area.draw(&PathElement::new(vec![(220, 0), (220 + legend_line_size, 0)], &BLUE,))
            .map_err(|e| e.to_string())?;
        legend_area.draw_text("Series 2", &text_style, (210, 0 + legend_font_dist),)
            .map_err(|e| e.to_string())?;

        // 11. 輸出圖檔
        root.present().map_err(|e| e.to_string())
    }

    pub fn send_to_front(&self) -> Result<String, String> {
        let bytes = fs::read(self.file_path.clone())
            .map_err(|e| format!("Cannot read file: {}", e))?;
        Ok(general_purpose::STANDARD.encode(bytes))
    }
}

#[tauri::command]
pub async fn chart_generate(app: AppHandle) -> Result<String, String> {
    let global_state = app.state::<GlobalState>();
    let rand_data_points = global_state.rand_data_points.lock().await;
    rand_data_points.line_chart_generate()?;
    rand_data_points.send_to_front()
}
