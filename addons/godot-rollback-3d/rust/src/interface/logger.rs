use flexi_logger::writers::{FileLogWriter, LogWriter};
use flexi_logger::{DeferredNow, FileSpec, LogSpecification, Logger as FlexiLogger, LoggerHandle};
use godot::classes::Os;
use godot::prelude::*;
use log::LevelFilter;
use std::cell::RefCell;

thread_local! {
    static LOCAL_PEER_IDX: RefCell<String> = RefCell::new(String::from("?"));
}

pub struct Logger {
    handle: LoggerHandle,
}

impl Logger {
    pub fn new() -> Self {
        let log_file_path = format!("{}/unconnected.log", get_log_file_dir());
        let file_spec = FileSpec::try_from(log_file_path).expect("Failed to initialize logger");

        let handle = FlexiLogger::with(LogSpecification::from(LevelFilter::Trace))
            .log_to_file_and_writer(file_spec, Box::new(GodotConsoleWriter))
            .start()
            .expect("Failed to initialize logger");

        log::debug!("Logger initialized");
        Self { handle }
    }

    pub fn set_peer_id(&mut self, peer_id: i64, peer_idx: u8) {
        let log_file_path = format!("{}/peer_{}.log", get_log_file_dir(), peer_id);
        self._update_log_file_path(&log_file_path);

        LOCAL_PEER_IDX.with(|stored_peer_idx| {
            stored_peer_idx.replace(format!("{}", peer_idx));
        });
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.handle
            .set_new_spec(LogSpecification::from(LevelFilter::from(&level)));
        log::debug!("Log level updated to: {:?}", level);
    }

    fn _update_log_file_path(&mut self, new_path: &String) {
        if let Err(e) = self.handle.reset_flw(
            &FileLogWriter::builder(
                FileSpec::try_from(new_path)
                    .expect(format!("Failed to write log file to path: {}", new_path).as_str()),
            )
            .max_level(LevelFilter::Debug)
            .append(),
        ) {
            godot_error!("Failed to update log file path: {:?}", e);
        } else {
            log::debug!("Log file path updated to: {}", new_path);
        }
    }
}

pub fn get_log_file_dir() -> String {
    let user_data_dir = Os::singleton().get_user_data_dir();
    format!("{}/godot-rollback-3d_logs", user_data_dir)
}

struct GodotConsoleWriter;

impl LogWriter for GodotConsoleWriter {
    fn write(&self, _now: &mut DeferredNow, record: &log::Record) -> std::io::Result<()> {
        match record.level() {
            log::Level::Error => godot_error!("[GR3D]: {}", record.args()),
            log::Level::Warn => godot_warn!("[GR3D]: {}", record.args()),
            log::Level::Info | log::Level::Debug => {
                LOCAL_PEER_IDX.with(|idx| {
                    let idx = idx.borrow();
                    godot_print!("[GR3D][{}][{}]: {}", *idx, record.level(), record.args())
                });
            }
            _ => {}
        }
        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(GodotConvert, Debug, Clone, Var, Export)]
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

impl From<&LogLevel> for LevelFilter {
    fn from(level: &LogLevel) -> Self {
        match level {
            &LogLevel::Off => LevelFilter::Off,
            &LogLevel::Error => LevelFilter::Error,
            &LogLevel::Warning => LevelFilter::Warn,
            &LogLevel::Info => LevelFilter::Info,
            &LogLevel::Debug => LevelFilter::Debug,
            &LogLevel::Trace => LevelFilter::Trace,
        }
    }
}
