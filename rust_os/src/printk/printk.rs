use core::fmt::{Write, Result};
use crate::vga_buffer::vga_buffer::WRITER;

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
        // Acquire lock on the global writer
        let mut writer = WRITER.lock();
        
        // Write the log level prefix
        writer.write_string(self.level.as_str());
        
        // Write the actual message
        writer.write_string(s);
        
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