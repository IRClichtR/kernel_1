use crate::arch::x86::port::outb;
use super::screen::{ Buffer, Screen, BUFFER_HEIGHT, BUFFER_WIDTH };

pub struct ScreenManager {
    pub screen: Screen,
    pub physical_buffer: &'static mut Buffer,
}

impl ScreenManager {
    pub fn new() -> Self {
        ScreenManager {
            screen: Screen::new(1), // Always use screen ID 1 as the main screen
            physical_buffer: unsafe {
                &mut *(0xb8000 as *mut Buffer)
            },
        }
    }

    pub fn get_screen(&self) -> &Screen {
        &self.screen
    }

    pub fn get_screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }

    pub fn clear_screen(&mut self) {
        self.screen.clear();
    }

    pub fn write_to_screen(&mut self, data: &str) {
        use super::screen::Writer;
        let mut writer = Writer::new(&mut self.screen);
        
        for byte in data.bytes() {
            writer.write_byte(byte);
        }
        
        self.flush_to_physical();
        self.update_cursor();
    }

    pub fn flush_to_physical(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.physical_buffer.chars[row][col] = self.screen.buffer.chars[row][col];
            }
        }
    }

    pub fn update_cursor(&self) {
        let row = self.screen.row_position.min(BUFFER_HEIGHT - 1);
        let col = self.screen.column_position.min(BUFFER_WIDTH - 1);

        let pos: u16 = (row * BUFFER_WIDTH + col) as u16;
        unsafe {
            outb(0x3D4, 0x0F);
            outb(0x3D5, (pos & 0xFF) as u8);

            outb(0x3D4, 0x0E);
            outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
        }
    }

    pub fn get_cursor_position(&self) -> (usize, usize) {
        (self.screen.row_position, self.screen.column_position)
    }

    pub fn set_cursor_position(&mut self, row: usize, col: usize) {
        self.screen.set_cursor_position(row, col);
        self.update_cursor();
    }
} 