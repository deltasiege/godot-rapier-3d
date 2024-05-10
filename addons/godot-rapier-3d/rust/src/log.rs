use godot::builtin::GString;
use godot::log::{ godot_error, godot_print, godot_warn };
use godot::prelude::{ Export, GodotConvert, Var };

pub fn log(engine_level: LogLevel, msg_level: LogLevel, message: &str) {
    if msg_level <= engine_level {
        match msg_level {
            LogLevel::Error => godot_error!("{}", message),
            LogLevel::Warning => godot_warn!("{}", message),
            LogLevel::Info => godot_print!("{}", message),
            LogLevel::Debug => godot_print!("{}", message),
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug, GodotConvert, Var, Export, Copy, Clone)]
#[godot(via = GString)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
}

// TODO - I wanted to generate these macros using a super macro, but apparently repeating args are not able to be nested
// Is there a cleaner way to do this?

#[macro_export]
macro_rules! error {
    ($engine_bind:expr; $($arg:expr),*) => {
        crate::log::log($engine_bind.log_level, crate::log::LogLevel::Error, &format!($($arg),*));
    };
    ($($arg:expr),*) => {
        {
            let engine_result = crate::get_engine_checked!();
            match engine_result {
                Ok(engine) => {
                    let bind = engine.bind();
                    crate::log::log(bind.log_level, crate::log::LogLevel::Error, &format!($($arg),*));
                },
                Err(_) => {
                    godot_error!("Failed to print error message: {}", &format!($($arg),*));
                },
            };
        }
    };
}

#[macro_export]
macro_rules! warn {
    ($engine_bind:expr; $($arg:expr),*) => {
        crate::log::log($engine_bind.log_level, crate::log::LogLevel::Warn, &format!($($arg),*));
    };
    ($($arg:expr),*) => {
        {
            let engine_result = crate::get_engine_checked!();
            match engine_result {
                Ok(engine) => {
                    let bind = engine.bind();
                    crate::log::log(bind.log_level, crate::log::LogLevel::Warn, &format!($($arg),*));
                },
                Err(_) => {
                    godot_warn!("Failed to print warn message: {}", &format!($($arg),*));
                },
            };
        }
    };
}

#[macro_export]
macro_rules! info {
    ($engine_bind:expr; $($arg:expr),*) => {
        crate::log::log($engine_bind.log_level, crate::log::LogLevel::Info, &format!($($arg),*));
    };
    ($($arg:expr),*) => {
        {
            let engine_result = crate::get_engine_checked!();
            match engine_result {
                Ok(engine) => {
                    let bind = engine.bind();
                    crate::log::log(bind.log_level, crate::log::LogLevel::Info, &format!($($arg),*));
                },
                Err(_) => {
                    godot_print!("Failed to print info message: {}", &format!($($arg),*));
                },
            };
        }
    };
}

#[macro_export]
macro_rules! debug {
    ($engine_bind:expr; $($arg:expr),*) => {
        crate::log::log($engine_bind.log_level, crate::log::LogLevel::Debug, &format!($($arg),*));
    };
    ($($arg:expr),*) => {
        {
            let engine_result = crate::get_engine_checked!();
            match engine_result {
                Ok(engine) => {
                    let bind = engine.bind();
                    crate::log::log(bind.log_level, crate::log::LogLevel::Debug, &format!($($arg),*));
                },
                Err(_) => {
                    godot_print!("Failed to print debug message: {}", &format!($($arg),*));
                },
            };
        }
    };
}
