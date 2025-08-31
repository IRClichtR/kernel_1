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
    loglvl_write_flag: bool
}


#[allow(dead_code)]
impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self { 
            level,
            loglvl_write_flag: false
         }
    }
}

impl Write for Logger {
    fn write_str(&mut self, s: &str) -> Result {
        let mut manager = screen_manager().lock();
        let active_screen_id = manager.get_active_screen_id();
        
        // Write to the currently active screen
        if let Some(active_screen) = &mut manager.screens[active_screen_id] {
            let mut writer = Writer::new(active_screen);
            
            if self.loglvl_write_flag == false {
                for byte in self.level.as_str().bytes() {
                    writer.write_byte(byte);
                }
                self.loglvl_write_flag = true;
            }
            
            // Write the actual message
            for byte in s.bytes() {
                writer.write_byte(byte);

                // Handle newline characters
                if byte == b'\n' {
                    // Reset log level write flag for next line
                    self.loglvl_write_flag = false;
                }
            }

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
        use crate::printk::printk::{Logger, LogLevel};

        let mut logger = Logger::new($level);
        
        // Panic if write fails
        match write!(logger, $($arg)*) {
            Ok(_) => {},
            Err(_) => {
                let mut debug_logger = Logger::new(LogLevel::Error);
                let _ = debug_logger.write_str("PRINTK ERROR!\n");
                panic!("printk failed");
            }
        }
    }};
    
    // Case with no log level, use Default
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use crate::printk::printk::{Logger, LogLevel};

        let mut logger = Logger::new(LogLevel::Default);
        
        match write!(logger, $($arg)*) {
            Ok(_) => {},
            Err(_) => {
                let mut debug_logger = Logger::new(LogLevel::Error);
                let _ = debug_logger.write_str("PRINTK ERROR!\n");
            }
        }
    }};
}