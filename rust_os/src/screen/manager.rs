use crate::arch::x86::port::outb;
use super::screen::{ Buffer, Screen, BUFFER_HEIGHT, BUFFER_WIDTH };

const MAX_SCREENS: usize = 2;

pub struct ScreenManager {
    pub screens: [Option<Screen>; MAX_SCREENS],
    pub active_screen_id: usize,
    pub physical_buffer: &'static mut Buffer,
}

impl ScreenManager {
    pub fn new() -> Self {
        ScreenManager {
            screens: core::array::from_fn(|i| Some(Screen::new(i + 1))),
            active_screen_id: 1,
            physical_buffer: unsafe {
                &mut *(0xb8000 as *mut Buffer)
            },
        }
    }

    pub fn get_screen(&self, screen_id: usize) -> Option<&Screen> {
        if screen_id >= 1 && screen_id <= MAX_SCREENS {
            self.screens[screen_id - 1].as_ref()
        } else {
            None
        }
    }

    pub fn get_screen_mut(&mut self, screen_id: usize) -> Option<&mut Screen> {
        if screen_id >= 1 && screen_id <= MAX_SCREENS {
            self.screens[screen_id - 1].as_mut()
        } else {
            None
        }
    }

    pub fn get_active_screen(&self) -> &Screen {
        self.screens[self.active_screen_id - 1].as_ref().unwrap()
    }

    pub fn get_active_screen_mut(&mut self) -> &mut Screen {
        self.screens[self.active_screen_id - 1].as_mut().unwrap()
    }

    pub fn get_active_screen_id(&self) -> usize {
        self.active_screen_id
    }

    pub fn clear_screen(&mut self, screen_id: usize) -> bool {
        if screen_id >= 1 && screen_id <= MAX_SCREENS {
            if let Some(screen) = &mut self.screens[screen_id - 1] {
                screen.clear();
                return true;
            }
        }
        false
    }

    pub fn clear_active_screen(&mut self) {
        if let Some(screen) = &mut self.screens[self.active_screen_id - 1] {
            screen.clear();
        }
    }

    pub fn write_to_screen(&mut self, screen_id: usize, data: &str) -> bool {
        if screen_id >= 1 && screen_id <= MAX_SCREENS {
            if let Some(screen) = &mut self.screens[screen_id - 1] {
                use super::screen::Writer;
                let mut writer = Writer::new(screen);
                
                for byte in data.bytes() {
                    writer.write_byte(byte);
                }
                
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

    pub fn write_to_active_screen(&mut self, data: &str) {
        if let Some(active_screen) = &mut self.screens[self.active_screen_id - 1] {
            use super::screen::Writer;
            let mut writer = Writer::new(active_screen);
            
            for byte in data.bytes() {
                writer.write_byte(byte);
            }
            
            self.flush_to_physical();
            self.update_cursor();
        }
    }

    pub fn flush_to_physical(&mut self) {
        let active_id = self.active_screen_id;
        
        if let Some(active_screen) = &self.screens[active_id - 1] {
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

    pub fn get_cursor_position(&self) -> (usize, usize) {
        let active = self.get_active_screen();
        (active.row_position, active.column_position)
    }

    pub fn set_cursor_position(&mut self, row: usize, col: usize) {
        if let Some(active_screen) = &mut self.screens[self.active_screen_id - 1] {
            active_screen.set_cursor_position(row, col);
            self.update_cursor();
        }
    }

    pub fn switch_screen(&mut self, screen_id: usize) -> bool {
        if screen_id >= 1 && screen_id <= MAX_SCREENS && self.screens[screen_id - 1].is_some() {
            self.active_screen_id = screen_id;
            self.flush_to_physical();
            self.update_cursor();
            true
        } else {
            false
        }
    }
} 