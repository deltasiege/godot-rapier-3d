use godot::prelude::*;
use log::{Level, LevelFilter, Metadata, Record};

pub fn init_logger() {
    let logger = GR3DLogger {};
    log::set_max_level(LevelFilter::from(LogLevel::default()));
    log::set_boxed_logger(Box::new(logger)).unwrap();
}

pub fn set_log_level(log_level: LogLevel) {
    log::set_max_level(LevelFilter::from(log_level));
}

pub struct GR3DLogger {}

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
