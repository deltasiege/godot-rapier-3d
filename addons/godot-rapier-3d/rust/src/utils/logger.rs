use std::fmt::Write;
use std::fs::File;
use std::sync::Mutex;
use tracing::field::{Field, Visit};
use tracing::level_filters::LevelFilter;
use tracing::{debug, Subscriber};
use tracing_subscriber::filter::Filtered;
use tracing_subscriber::fmt::format::{DefaultFields, Format, Writer};
use tracing_subscriber::fmt::{Layer, MakeWriter};
use tracing_subscriber::layer::{Layer as LayerTrait, Layered};
use tracing_subscriber::reload::{Handle, Layer as ReloadLayer};
use tracing_subscriber::{filter, fmt, Registry};
use tracing_subscriber::{prelude::*, reload};

use godot::{
    classes::{Engine, Os},
    prelude::*,
};

pub struct GR3DLogger {
    console_handle: Option<ConsoleHandle>,
    file_handle: Option<FileHandle>,
}

impl GR3DLogger {
    pub fn new() -> Self {
        GR3DLogger {
            console_handle: None,
            file_handle: None,
        }
    }

    pub fn init(&mut self) {
        let console_layer = GodotConsoleLogger::new().with_filter(filter::LevelFilter::OFF);
        let (console_layer, console_handle) = reload::Layer::new(console_layer);

        let file_layer = fmt::Layer::default()
            .with_writer(Mutex::new(DummyWriter))
            .with_filter(filter::LevelFilter::OFF);
        let (file_layer, file_handle) = reload::Layer::new(file_layer);

        let subscriber = Registry::default().with(console_layer).with(file_layer);

        tracing::subscriber::set_global_default(subscriber).expect("Logger initialization failed");

        debug!("Hello from tracing!");

        self.console_handle = Some(console_handle);
        self.file_handle = Some(file_handle);
    }

    // pub fn set_level(&mut self, level: LogLevel) {

    // let level_filter: LevelFilter = level.into();
    // self.file_reload_handle
    //     .as_ref()
    //     .unwrap()
    //     .reload_with(filter::LevelFilter::new(level_filter));
    // }

    pub fn update_log_file_path(&mut self, new_path: String) {
        match self.file_handle.as_ref() {
            Some(handle) => {
                let log_file = match File::create(new_path) {
                    Ok(file) => file,
                    Err(e) => {
                        godot_error!("Failed to open log file: {}", e);
                        return;
                    }
                };

                let file_layer = fmt::Layer::default()
                    .with_writer(Mutex::new(log_file))
                    .with_filter(filter::LevelFilter::TRACE);
                if let Err(e) = handle.reload(file_layer) {
                    godot_error!("Failed to update_log_file_path: {:?}", e);
                }
            }
            None => {
                godot_error!("File handle is not initialized");
            }
        }
    }
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

impl<S: Subscriber> LayerTrait<S> for GodotConsoleLogger {
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

type ConsoleHandle = Handle<Filtered<GodotConsoleLogger, LevelFilter, Registry>, Registry>;

type ReloadableConsoleLayer =
    Layered<ReloadLayer<Filtered<GodotConsoleLogger, LevelFilter, Registry>, Registry>, Registry>;

type FileHandle<T: MakeWriter> = Handle<
    Filtered<
        Layer<
            Layered<Layer<Filtered<GodotConsoleLogger, LevelFilter, Registry>, Registry>, Registry>,
            DefaultFields,
            Format,
            T,
        >,
        LevelFilter,
        Layered<Layer<Filtered<GodotConsoleLogger, LevelFilter, Registry>, Registry>, Registry>,
    >,
    Layered<Layer<Filtered<GodotConsoleLogger, LevelFilter, Registry>, Registry>, Registry>,
>;

struct DummyWriter;

impl std::io::Write for DummyWriter {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for DummyWriter {
    type Writer = Self;

    fn make_writer(&self) -> Self {
        DummyWriter
    }
}
