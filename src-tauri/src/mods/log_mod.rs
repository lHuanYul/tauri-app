use std::io;
use log::Level;
use chrono::Local;
use ansi_term::Colour::{Green, Yellow, Red, Blue, Purple};
use fern::Dispatch;
use crate::DEFAULT_LOG_LEVEL;

pub const   CODE_TRACE: &'static str = "-trc";

pub fn init() {
    Dispatch::new()
        .level(DEFAULT_LOG_LEVEL)
        .format(|out, message, record| {
            let level = match record.level() {
                Level::Error => Red     .bold().paint("ERROR>>"),
                Level::Warn  => Yellow  .bold().paint(" WARN>>"),
                Level::Info  => Green   .bold().paint(" INFO>>"),
                Level::Debug => Blue    .bold().paint("DEBUG>>"),
                Level::Trace => Purple  .bold().paint("TRACE>>"),
            };
            out.finish(format_args!(
                "[{}] {} {}",
                Local::now().format("%y-%m-%d %H:%M:%S"),
                level,
                message
            ));
        })
        .chain(io::stdout())
        // 如需同時輸出到檔案，可再加一條 chain
        // .chain(fern::log_file("output.log").unwrap())
        .apply()
        .expect("Failed to initialize logger");
}

