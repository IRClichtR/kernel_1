use core::fmt::{Write, Result};
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::screen::global::screen_manager;
use crate::screen::screen::Writer;

static PRINTK_TARGET_SCREEN: AtomicUsize = AtomicUsize::new(0);

pub fn set_printk_screen(screen_id: usize) {
    PRINTK_TARGET_SCREEN.store(screen_id, Ordering::Relaxed);
}

pub fn get_printk_screen() -> usize {
    PRINTK_TARGET_SCREEN.load(Ordering::Relaxed)
}

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
        // Get the screen manager and write to the target screen
        let mut manager = screen_manager().lock();
        let target_screen = get_printk_screen();
        
        // Write to the target screen using the screen manager
        if let Some(target_screen) = &mut manager.screens[target_screen] {
            let mut writer = Writer::new(target_screen);
            
            // Write the log level prefix
            for byte in self.level.as_str().bytes() {
                writer.write_byte(byte);
            }
            
            // Write the actual message
            for byte in s.bytes() {
                writer.write_byte(byte);
            }
            
            // Only update physical display if this is the active screen
            if target_screen == manager.active_screen_id {
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