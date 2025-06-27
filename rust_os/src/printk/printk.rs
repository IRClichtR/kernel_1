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
        let active_screen_id = manager.get_active_screen_id();
        
        // Write to the currently active screen
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let mut writer = Writer::new(active_screen);
            
            // Write the log level prefix
            for byte in self.level.as_str().bytes() {
                writer.write_byte(byte);
            }
            
            // Write the actual message
            for byte in s.bytes() {
                writer.write_byte(byte);
            }
            
            // Always flush and update cursor for active screen
            manager.flush_to_physical();
            manager.update_cursor();
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