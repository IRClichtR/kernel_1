use core::fmt::{Write, Result};
use crate::screen::global::screen_manager;
use crate::screen::screen::Writer;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum LogLevel {
    Emergency,
    Alert,
    Critical,
    Error,
    Warn,
    Notice,
    Info,
    Debug,
    Default
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Emergency => "<0> ",
            LogLevel::Alert => "<1> ",
            LogLevel::Critical => "<2> ",
            LogLevel::Error => "<3> ",
            LogLevel::Warn => "<4> ",
            LogLevel::Notice => "<5> ",
            LogLevel::Info => "<6> ",
            LogLevel::Debug => "<7> ",
            LogLevel::Default => "",
        }
    }
}

pub struct Logger {
    level: LogLevel,
}

#[allow(dead_code)]
impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }
}

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> Result {
        let mut manager = screen_manager().lock();
        
        if let Some(screen) = manager.get_screen_mut(1) {
            let mut writer = Writer::new(screen);
            
            for byte in self.level.as_str().bytes() {
                writer.write_byte(byte);
            }
            
            for byte in s.bytes() {
                writer.write_byte(byte);
            }
        }

        if manager.get_active_screen_id() == 1 {
            manager.flush_to_physical();
            manager.update_cursor();
        }
        
        Ok(())
    }
}

#[macro_export]
macro_rules! printk {
    ($level:expr, $($arg:tt)*) => {
        {
            use crate::printk::printk::{Logger, LogLevel};
            use core::fmt::Write;
            let mut logger = Logger::new($level);
            let _ = write!(logger, $($arg)*);
        }
    };
    ($($arg:tt)*) => {
        {
            use crate::printk::printk::{Logger, LogLevel};
            use core::fmt::Write;
            let mut logger = Logger::new(LogLevel::Default);
            let _ = write!(logger, $($arg)*);
        }
    };
}