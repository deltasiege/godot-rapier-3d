use std::io::Write;

use godot::{classes::Os, prelude::*};
use log::{Level, LevelFilter, Metadata, Record};

pub fn init_logger(log_level: LogLevel) {
    let logger = GR3DLogger {
        peer_id: None,
        user_data_dir: "".to_string(),
        log_dir: "".to_string(),
        log_file: "".to_string(),
    };

    log::set_max_level(LevelFilter::from(log_level));

    let boxed = Box::new(logger);

    if let Err(e) = log::set_boxed_logger(boxed) {
        godot_error!("Failed to set logger: {}", e);
    }
}

pub fn set_log_level(log_level: LogLevel) {
    log::set_max_level(LevelFilter::from(log_level));
}

pub struct GR3DLogger {
    peer_id: Option<i64>,
    user_data_dir: String,
    log_dir: String,
    log_file: String,
}

impl GR3DLogger {
    fn set_peer_id(&mut self, peer_id: i64) {
        self.peer_id = Some(peer_id);
        self.user_data_dir = Os::get_user_data_dir(&Os::singleton()).to_string();
        self.log_dir = format!("{}/godot-rapier-3d/logs/{}/", self.user_data_dir, peer_id);
        self.log_file = format!("{}log.txt", self.log_dir);

        self.create_log_dir();
    }

    fn create_log_dir(&self) {
        if self.log_dir.is_empty() {
            return;
        }

        if let Err(e) = std::fs::create_dir_all(&self.log_dir) {
            godot_error!("Failed to create log directory: {}", e);
        }
    }

    fn log_to_user_data_file(&self, record: &Record) {
        if self.log_file.is_empty() {
            return;
        }

        let mut file = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
        {
            Ok(file) => file,
            Err(e) => {
                godot_error!("Failed to open log file: {}", e);
                return;
            }
        };

        file.write(format!("[GR3D][{}]: {}\n", record.level(), record.args()).as_bytes())
            .unwrap();
    }
}

impl log::Log for GR3DLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Error => godot_error!("[GR3D]: {}", record.args()),
                Level::Warn => godot_warn!("[GR3D]: {}", record.args()),
                _ => godot_print!("[GR3D][{}]: {}", record.level(), record.args()),
            }
        }

        self.log_to_user_data_file(record);
    }

    fn flush(&self) {}
}

#[derive(Debug, Clone, GodotConvert, Var, Export)]
#[godot(via = GString)]
pub enum LogLevel {
    Off,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warning => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}
