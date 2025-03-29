use std::fmt::Write;
use std::fs::File;
use std::sync::Mutex;
use tracing::field::{Field, Visit};
use tracing::{debug, Subscriber};
use tracing_subscriber::layer::Layer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{fmt, Registry};

use godot::{
    classes::{Engine, Os},
    prelude::*,
};
use log::{Level, LevelFilter, Metadata, Record};

pub fn init_logger() {
    let log_file = match File::create("C:/Users/loki/Desktop/test/my_cool_trace.log") {
        Ok(file) => file,
        Err(e) => {
            godot_error!("Failed to open log file: {}", e);
            return;
        }
    };

    let subscriber = Registry::default()
        .with(fmt::Layer::default().with_writer(Mutex::new(log_file)))
        .with(GodotConsoleLogger::new());

    tracing::subscriber::set_global_default(subscriber).expect("Logger initialization failed");

    debug!("Hello from tracing!");
}

pub struct StringVisitor<'a> {
    string: &'a mut String,
}

impl<'a> Visit for StringVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        write!(self.string, "{} = {:?}; ", field.name(), value).unwrap();
    }
}

pub struct GodotConsoleLogger {}

impl GodotConsoleLogger {
    pub fn new() -> Self {
        GodotConsoleLogger {}
    }
}

impl<S: Subscriber> Layer<S> for GodotConsoleLogger {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let level = event.metadata().level();
        let visitor = &mut StringVisitor {
            string: &mut String::new(),
        };
        event.record(visitor);
        match *level {
            tracing::Level::ERROR => godot_error!("[GR3D]: {:?}", visitor.string),
            tracing::Level::WARN => godot_warn!("[GR3D]: {:?}", visitor.string),
            _ => godot_print!("[GR3D][{}]: {:?}", level, visitor.string),
        }
    }
}

// impl GodotConsoleLogger {
//     fn set_peer_id(&mut self, peer_id: i64) {
//         self.peer_id = Some(peer_id);
//         self.user_data_dir = Os::get_user_data_dir(&Os::singleton()).to_string();
//         self.log_dir = format!("{}/godot-rapier-3d/logs/{}/", self.user_data_dir, peer_id);
//         self.log_file = format!("{}log.txt", self.log_dir);

//         self.create_log_dir();
//     }

//     fn create_log_dir(&self) {
//         if self.log_dir.is_empty() {
//             return;
//         }

//         if let Err(e) = std::fs::create_dir_all(&self.log_dir) {
//             godot_error!("Failed to create log directory: {}", e);
//         }
//     }

//     fn log_to_user_data_file(&self, record: &Record) {
//         if self.log_file.is_empty() {
//             return;
//         }

//         let mut file = match std::fs::OpenOptions::new()
//             .create(true)
//             .append(true)
//             .open(&self.log_file)
//         {
//             Ok(file) => file,
//             Err(e) => {
//                 godot_error!("Failed to open log file: {}", e);
//                 return;
//             }
//         };

//         file.write(format!("[GR3D][{}]: {}\n", record.level(), record.args()).as_bytes())
//             .unwrap();
//     }
// }

// impl log::Log for GodotConsoleLogger {
//     fn enabled(&self, metadata: &Metadata) -> bool {
//         metadata.level() <= log::max_level()
//     }

//     fn log(&self, record: &Record) {
//         if self.enabled(record.metadata()) {
//             match record.level() {
//                 Level::Error => godot_error!("[GR3D]: {}", record.args()),
//                 Level::Warn => godot_warn!("[GR3D]: {}", record.args()),
//                 _ => godot_print!("[GR3D][{}]: {}", record.level(), record.args()),
//             }
//         }

//         self.log_to_user_data_file(record);
//     }

//     fn flush(&self) {}
// }

// #[derive(Debug, Clone, GodotConvert, Var, Export)]
// #[godot(via = GString)]
// pub enum LogLevel {
//     Off,
//     Error,
//     Warning,
//     Info,
//     Debug,
//     Trace,
// }

// impl Default for LogLevel {
//     fn default() -> Self {
//         LogLevel::Info
//     }
// }

// impl From<LogLevel> for LevelFilter {
//     fn from(level: LogLevel) -> Self {
//         match level {
//             LogLevel::Off => LevelFilter::Off,
//             LogLevel::Error => LevelFilter::Error,
//             LogLevel::Warning => LevelFilter::Warn,
//             LogLevel::Info => LevelFilter::Info,
//             LogLevel::Debug => LevelFilter::Debug,
//             LogLevel::Trace => LevelFilter::Trace,
//         }
//     }
// }
