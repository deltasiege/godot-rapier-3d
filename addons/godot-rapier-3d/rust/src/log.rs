use godot::builtin::GString;
use godot::log::{ godot_error, godot_print, godot_warn };
use godot::prelude::{ Export, GodotConvert, Var };

pub struct Logger {
    pub level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        if level <= self.level {
            match level {
                LogLevel::Error => godot_error!("{}", message),
                LogLevel::Warning => godot_warn!("{}", message),
                LogLevel::Info => godot_print!("{}", message),
                LogLevel::Debug => godot_print!("{}", message),
            }
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug, GodotConvert, Var, Export)]
#[godot(via = GString)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
}

// #[macro_export]
// macro_rules! error {
//     ($logger:expr; $($arg:expr),*) => {
//       $logger.log(crate::log::LogLevel::Error, &format!($($arg),*));
//     };
//     ($($arg:expr),*) => {
//       let singleton = crate::utils::get_engine_singleton();
//       match singleton {
//         Some(singleton) => singleton.bind().logger.log(crate::log::LogLevel::Error, &format!($($arg),*)),
//         None => godot_error!("Failed to print error message: {}", &format!($($arg),*)),
//       }
//     };
// }

// #[macro_export]
// macro_rules! warn {
//     ($logger:expr; $($arg:expr),*) => {
//       $logger.log(crate::log::LogLevel::Warning, &format!($($arg),*));
//     };
//     ($($arg:expr),*) => {
//       let singleton = crate::utils::get_engine_singleton();
//       match singleton {
//         Some(singleton) => singleton.bind().logger.log(crate::log::LogLevel::Warning, &format!($($arg),*)),
//         None => godot_warn!("Failed to print warning message: {}", &format!($($arg),*)),
//       }
//     };
// }

// #[macro_export]
// macro_rules! info {
//     ($logger:expr; $($arg:expr),*) => {
//       $logger.log(crate::log::LogLevel::Info, &format!($($arg),*));
//     };
//     ($($arg:expr),*) => {
//       let singleton = crate::utils::get_engine_singleton();
//       match singleton {
//         Some(singleton) => singleton.bind().logger.log(crate::log::LogLevel::Info, &format!($($arg),*)),
//         None => godot_print!("Failed to print info message: {}", &format!($($arg),*)),
//       }
//     }
// }

// #[macro_export]
// macro_rules! debug {
//     ($logger:expr; $($arg:expr),*) => {
//       $logger.log(crate::log::LogLevel::Debug, &format!($($arg),*));
//     };
//     ($($arg:expr),*) => {
//       let singleton = crate::utils::get_engine_singleton();
//       match singleton {
//         Some(singleton) => singleton.bind().logger.log(crate::log::LogLevel::Debug, &format!($($arg),*)),
//         None => godot_print!("Failed to print debug message: {}", &format!($($arg),*)),
//       }
//     }
// }
