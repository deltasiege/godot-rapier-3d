use flexi_logger::{
    writers::{FileLogWriter, LogWriter},
    DeferredNow, FileSpec, LogSpecification, Logger, LoggerHandle,
};
use godot::{
    classes::{Engine, Os},
    prelude::*,
};
use log::LevelFilter;

/// Use the GR3DNet singleton to share physics + action data between network clients
#[derive(GodotClass)]
#[class(base = Object)]
pub struct GR3DLogger {
    handle: Option<LoggerHandle>,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GR3DLogger {
    fn init(base: Base<Object>) -> Self {
        let user_data_dir = Os::singleton().get_user_data_dir();
        let log_file_path = format!("{}/godot-rapier-3d_logs/unconnected.log", user_data_dir);
        let file_spec = FileSpec::try_from(log_file_path).expect("Failed to initialize logger");

        let handle = Logger::with(LogSpecification::from(LevelFilter::Trace))
            .log_to_file_and_writer(file_spec, Box::new(GodotConsoleWriter))
            .start()
            .expect("Failed to initialize logger");

        log::debug!("Logger initialized");

        Self {
            handle: Some(handle),
            base,
        }
    }
}

#[godot_api]
impl GR3DLogger {
    #[func]
    pub fn receive_peer_id(&mut self, peer_id: i64) {
        let user_data_dir = Os::singleton().get_user_data_dir();
        let log_file_path = format!(
            "{}/godot-rapier-3d_logs/peer_{}.log",
            user_data_dir, peer_id
        );
        self._update_log_file_path(&log_file_path);
    }

    #[func]
    pub fn set_level(&mut self, level: LogLevel) {
        match self.handle {
            Some(ref mut handle) => {
                handle.set_new_spec(LogSpecification::from(LevelFilter::from(&level)));
            }
            None => {
                godot_print!("Logger not initialized");
                return;
            }
        };

        log::debug!("Log level updated to: {:?}", level);
    }

    pub fn _update_log_file_path(&mut self, new_path: &String) {
        match self.handle {
            Some(ref mut handle) => {
                if let Err(e) = handle.reset_flw(
                    &FileLogWriter::builder(FileSpec::try_from(new_path).expect(
                        format!("Failed to write log file to path: {}", new_path).as_str(),
                    ))
                    .max_level(LevelFilter::Debug)
                    .append(),
                ) {
                    godot_error!("Failed to update log file path: {:?}", e);
                }
            }
            None => {
                godot_error!("Logger not initialized");
                return;
            }
        };

        log::debug!("Log file path updated to: {}", new_path);
    }
}

const NAME: &str = "GR3DLogger";

pub fn register() {
    Engine::singleton().register_singleton(NAME, &GR3DLogger::new_alloc());
}

pub fn unregister() {
    let mut engine = Engine::singleton();
    if let Some(my_singleton) = engine.get_singleton(NAME) {
        engine.unregister_singleton(NAME);
        my_singleton.free();
    } else {
        log::error!("Failed to get {} singleton", NAME);
    }
}

struct GodotConsoleWriter;

impl LogWriter for GodotConsoleWriter {
    fn write(&self, _now: &mut DeferredNow, record: &log::Record) -> std::io::Result<()> {
        match record.level() {
            log::Level::Error => godot_error!("[GR3D]: {}", record.args()),
            log::Level::Warn => godot_warn!("[GR3D]: {}", record.args()),
            log::Level::Debug => godot_print!("[GR3D][{}]: {}", record.level(), record.args()),
            _ => {}
        }
        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }
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
