use std::{error::Error, fs, iter, path::{self, PathBuf}};
use log::error;
use plotters::prelude::*;
use rand::{self, Rng};
use base64::{engine::general_purpose, Engine};
use tauri::{AppHandle, Manager};
use crate::{mods::directory_mod, GlobalState, GENERATE_FOLDER_PATH};

fn store_folder() -> String {
    format!("{}/chart", GENERATE_FOLDER_PATH)
}

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

    /// 推入一個新值，並移除超額的最舊值
    pub fn push(&mut self, value: i32) {
        self.data_points_1.push(value);
        if self.data_points_1.len() > self.max_length {
            self.data_points_1.remove(0);
        }
    }
    
    pub fn send_to_front(&self) -> Result<String, String> {
        let bytes = fs::read(self.file_path.clone())
            .map_err(|e| format!("Cannot read file: {}", e))?;
        Ok(general_purpose::STANDARD.encode(bytes))
    }

    pub fn line_chart_generate(&self) -> Result<(), String> {
        let (root_size_x, root_size_y) = (2160, 1215);
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
        let (data_min_x, data_max_x) =
            (0 as i32, self.max_length as i32);
        let data_all_vals = self.data_points_1.iter().chain(self.data_points_2.iter()).cloned();
        let (data_min_y, data_max_y) =
            (data_all_vals.clone().min().unwrap_or(0) - 1, data_all_vals.max().unwrap_or(0) + 1);

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
        chart.draw_series(
            iter::once(PathElement::new(
                vec![(data_min_x, 0), (data_max_x, 0)],
                BLACK.stroke_width(1),
            ))
        ).map_err(|e| e.to_string())?;

        let data_dot_size = 5;
        let legend_dot_size = 5;
        let legend_text_style = ("Serif", 25).into_text_style(&legend_area);
        let legend_line_len = 40;
        let legend_font_dist = 20;
        // SquareMarker
        {
            let color = RED;
            let data_style = color.stroke_width(3);
            let legend_style = color.stroke_width(2);
            let (legend_x, legend_y) = (120, 0);
            let legend_name = "Series 1";
            let (legend_name_x, legend_name_y) = (legend_x - 10, legend_y + legend_font_dist);
            let points: Vec<(i32, i32)> = self.data_points_1.iter()
                .enumerate().map(|(i, &v)| (i as i32, v)).collect();

            chart.draw_series(LineSeries::new(points.clone(), data_style))
                .map_err(|e| e.to_string())?;
            chart.draw_series(
                points.iter().map(|&p| Circle::new(p, data_dot_size, color.filled()))
            ).map_err(|e| e.to_string())?;

            legend_area.draw(&PathElement::new(
                    vec![(legend_x, legend_y), (legend_x + legend_line_len, legend_y)],
                    legend_style,
                ))
                .map_err(|e| e.to_string())?;
            legend_area.draw(&Circle::new(
                    (legend_x + legend_line_len / 2, legend_y),
                    legend_dot_size, color.filled()
                ))
                .map_err(|e| e.to_string())?;
            legend_area.draw_text(legend_name, &legend_text_style, (legend_name_x, legend_name_y))
                .map_err(|e| e.to_string())?;
        }
        {
            let color = BLUE;
            let data_style = color.stroke_width(3);
            let legend_style = color.stroke_width(2);
            let (legend_x, legend_y) = (220, 0);
            let legend_name = "Series 2";
            let (legend_name_x, legend_name_y) = (legend_x - 10, legend_y + legend_font_dist);
            let points: Vec<(i32, i32)> = self.data_points_2.iter()
                .enumerate().map(|(i, &v)| (i as i32, v)).collect();

            chart.draw_series(LineSeries::new(points.clone(), data_style))
                .map_err(|e| e.to_string())?;
            chart.draw_series(
                points.iter().map(|&p| TriangleMarker::new(p, data_dot_size, color.filled()))
            ).map_err(|e| e.to_string())?;

            legend_area.draw(&PathElement::new(
                vec![(legend_x, legend_y), (legend_x + legend_line_len, legend_y)],
                legend_style
                ))
                .map_err(|e| e.to_string())?;
            legend_area.draw(&TriangleMarker::new(
                    (legend_x + legend_line_len / 2, legend_y),
                    legend_dot_size, color.filled()
                ))
                .map_err(|e| e.to_string())?;
            legend_area.draw_text(legend_name, &legend_text_style, (legend_name_x, legend_name_y))
                .map_err(|e| e.to_string())?;
        }

        root.present().map_err(|e| e.to_string())
    }
}

pub fn line_chart_generate<P: AsRef<path::Path>>(
    folder_path: P,
    file_name: &str,
    chart_name: &str,
    data_points: Vec<i32>,
    x_name: &str,
    y_name: &str,
) -> Result<(), Box<dyn Error>> {
    let (root_size_x, root_size_y) = (2160, 1215);

    let file_path = directory_mod::create_file(folder_path, file_name).unwrap();
    // 1. 建立繪圖區並填背景
    let root = BitMapBackend::new(&file_path, (root_size_x, root_size_y))
        .into_drawing_area();
    root.fill(&WHITE).map_err(|e| e.to_string())?;

    let legend_area_size = 100;
    let (data_area, legend_area) = 
        root.split_vertically(root_size_y - legend_area_size);

    // 3. 留邊
    let root_mergin = 40;
    let data_area   = data_area  .margin(root_mergin, 0, root_mergin, root_mergin);
    let legend_area = legend_area.margin(0, root_mergin, root_mergin, root_mergin);

    // 4. 計算座標範圍
    let (data_min_x, data_max_x) =
        (0 as i32, data_points.len() as i32);
    let data_all_vals = data_points.iter().cloned();
    let (data_min_y, data_max_y) =
        (data_all_vals.clone().min().unwrap_or(0) - 1, data_all_vals.max().unwrap_or(0) + 1);

    // 5. 建立 ChartContext
    let mut chart = ChartBuilder::on(&data_area)
        .caption(chart_name, ("Serif", 40))
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(data_min_x..data_max_x, data_min_y..data_max_y)
        .map_err(|e| e.to_string())?;

    // 6. 繪製網格與軸
    chart.configure_mesh()
        .x_desc(x_name)
        .y_desc(y_name)
        .axis_desc_style(("Serif", 30))
        .label_style(("Serif", 20))
        .draw()
        .map_err(|e| e.to_string())?;
    chart.draw_series(
        iter::once(PathElement::new(
            vec![(data_min_x, 0), (data_max_x, 0)],
            BLACK.stroke_width(1),
        ))
    ).map_err(|e| e.to_string())?;

    let data_dot_size = 5;
    let legend_dot_size = 5;
    let legend_text_style = ("Serif", 25).into_text_style(&legend_area);
    let legend_line_len = 40;
    let legend_font_dist = 20;
    {
        let color = BLUE;
        let data_style = color.stroke_width(3);
        let legend_style = color.stroke_width(2);
        let (legend_x, legend_y) = (220, 0);
        let legend_name = "Series 2";
        let (legend_name_x, legend_name_y) = (legend_x - 10, legend_y + legend_font_dist);
        let points: Vec<(i32, i32)> = data_points.iter()
            .enumerate().map(|(i, &v)| (i as i32, v)).collect();

        chart.draw_series(LineSeries::new(points.clone(), data_style))
            .map_err(|e| e.to_string())?;
        chart.draw_series(
            points.iter().map(|&p| TriangleMarker::new(p, data_dot_size, color.filled()))
        ).map_err(|e| e.to_string())?;

        legend_area.draw(&PathElement::new(
            vec![(legend_x, legend_y), (legend_x + legend_line_len, legend_y)],
            legend_style
            ))
            .map_err(|e| e.to_string())?;
        legend_area.draw(&TriangleMarker::new(
                (legend_x + legend_line_len / 2, legend_y),
                legend_dot_size, color.filled()
            ))
            .map_err(|e| e.to_string())?;
        legend_area.draw_text(legend_name, &legend_text_style, (legend_name_x, legend_name_y))
            .map_err(|e| e.to_string())?;
    }

    Ok(root.present().map_err(|e| e)?)
}

#[tauri::command]
pub async fn chart_generate(app: AppHandle) -> Result<String, String> {
    let global_state = app.state::<GlobalState>();
    let rand_data_points = global_state.rand_datas.lock().await;
    rand_data_points.line_chart_generate()?;
    rand_data_points.send_to_front()
}
