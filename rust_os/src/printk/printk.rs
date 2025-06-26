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
        // Get the screen manager and write to screen 0 only
        let mut manager = screen_manager().lock();
        
        // Write to screen 0 using the screen manager
        if let Some(screen_0) = &mut manager.screens[0] {
            let mut writer = Writer::new(screen_0);
            
            // Write the log level prefix
            for byte in self.level.as_str().bytes() {
                writer.write_byte(byte);
            }
            
            // Write the actual message
            for byte in s.bytes() {
                writer.write_byte(byte);
            }
            
            // Only update physical display if screen 0 is active
            if manager.active_screen_id == 0 {
                manager.flush_to_physical();
                manager.update_cursor();
            }
        }
        
        Ok(())
    }
}

#[macro_export]
macro_rules! printk {
    // Case with explicit log level
    ($level:expr, $($arg:tt)*) => {{
        use core::fmt::Write;
        use crate::printk::printk::Logger;
        use crate::printk::printk::LogLevel;
        let mut logger = Logger::new($level);
        let _ = write!(logger, $($arg)*);
    }};
    
    // Case with no log level, use Default
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use crate::printk::printk::Logger;
        use crate::printk::printk::LogLevel;
        let mut logger = Logger::new(LogLevel::Default);
        let _ = write!(logger, $($arg)*);
    }};
}