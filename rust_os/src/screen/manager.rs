use crate::arch::x86::port::outb;
use super::screen::{ Buffer, Screen, BUFFER_HEIGHT, BUFFER_WIDTH };

const MAX_SCREENS: usize = 3;

pub struct ScreenManager {
    pub screens: [Option<Screen>; MAX_SCREENS],
    pub active_screen_id: usize,
    pub physical_buffer: &'static mut Buffer,
}

impl ScreenManager {
    pub fn new() -> Self {
        ScreenManager {
            screens: core::array::from_fn(|i| if i == 0 { Some(Screen::new(i)) } else { None }),
            active_screen_id: 0,
            physical_buffer: unsafe {
                &mut *(0xb8000 as *mut Buffer)
            },
        }
    }

    pub fn get_active_screen(&self) -> &Screen {
        self.screens[self.active_screen_id].as_ref().unwrap()
    }

    pub fn get_active_screen_mut(&mut self) -> &mut Screen {
        self.screens[self.active_screen_id].as_mut().unwrap()
    }

    /// Get the current active screen ID in a thread-safe manner
    pub fn get_active_screen_id(&self) -> usize {
        self.active_screen_id
    }

    /// Check if a screen is currently active
    pub fn is_screen_active(&self, screen_id: usize) -> bool {
        screen_id < MAX_SCREENS && self.active_screen_id == screen_id
    }

    /// Check if a screen exists and is available
    pub fn is_screen_available(&self, screen_id: usize) -> bool {
        screen_id < MAX_SCREENS && self.screens[screen_id].is_some()
    }

    /// Clear a specific screen
    pub fn clear_screen(&mut self, screen_id: usize) -> bool {
        if screen_id < MAX_SCREENS {
            if let Some(screen) = &mut self.screens[screen_id] {
                screen.clear();
                return true;
            }
        }
        false
    }

    /// Clear the currently active screen
    pub fn clear_active_screen(&mut self) -> bool {
        self.clear_screen(self.active_screen_id)
    }

    /// Write data directly to the currently active screen
    pub fn write_to_active_screen(&mut self, data: &str) -> bool {
        if let Some(active_screen) = &mut self.screens[self.active_screen_id] {
            use super::screen::Writer;
            let mut writer = Writer::new(active_screen);
            
            for byte in data.bytes() {
                writer.write_byte(byte);
            }
            
            self.flush_to_physical();
            self.update_cursor();
            true
        } else {
            false
        }
    }

    /// Write data to a specific screen (for internal use)
    pub fn write_to_screen(&mut self, screen_id: usize, data: &str) -> bool {
        if screen_id < MAX_SCREENS {
            if let Some(screen) = &mut self.screens[screen_id] {
                use super::screen::Writer;
                let mut writer = Writer::new(screen);
                
                for byte in data.bytes() {
                    writer.write_byte(byte);
                }
                
                // Only flush and update cursor if this is the active screen
                if self.active_screen_id == screen_id {
                    self.flush_to_physical();
                    self.update_cursor();
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn create_screen(&mut self) -> Option<usize> {
        for i in 0..MAX_SCREENS {
            if self.screens[i].is_none() {
                self.screens[i] = Some(Screen::new(i));
                return Some(i);
            }
        }
        None
    }

    pub fn flush_to_physical(&mut self) {
        let active_id = self.active_screen_id;
        
        if let Some(active_screen) = &self.screens[active_id] {
            for row in 0..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    self.physical_buffer.chars[row][col] = active_screen.buffer.chars[row][col];
                }
            }
        }
    }

    pub fn update_cursor(&self) {
        let active = self.get_active_screen();
        let row = active.row_position.min(BUFFER_HEIGHT - 1);
        let col = active.column_position.min(BUFFER_WIDTH - 1);

        let pos: u16 = (row * BUFFER_WIDTH + col) as u16;
        unsafe {
            outb(0x3D4, 0x0F);
            outb(0x3D5, (pos & 0xFF) as u8);

            outb(0x3D4, 0x0E);
            outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
        }
    }

    /// Get the current cursor position of the active screen
    pub fn get_cursor_position(&self) -> (usize, usize) {
        let active = self.get_active_screen();
        (active.row_position, active.column_position)
    }

    /// Set the cursor position of the active screen
    pub fn set_cursor_position(&mut self, row: usize, col: usize) {
        if let Some(active_screen) = &mut self.screens[self.active_screen_id] {
            active_screen.set_cursor_position(row, col);
            self.update_cursor();
        }
    }

    /// Debug method to print current screen and cursor information
    pub fn debug_info(&self) {
        let active = self.get_active_screen();
        // This would need to be implemented with a print function
        // For now, we'll just ensure cursor is properly positioned
        self.update_cursor();
    }

    pub fn switch_screen(&mut self, screen_id: usize) -> bool {
        if screen_id < MAX_SCREENS && self.screens[screen_id].is_some() {
            // Switch to the new screen
            self.active_screen_id = screen_id;
            
            // Flush the new screen's content to physical buffer
            self.flush_to_physical();
            
            // Update cursor to the position stored in the new screen
            self.update_cursor();
            true
        } else {
            false
        }
    }
} 