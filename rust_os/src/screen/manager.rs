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

    pub fn switch_screen(&mut self, screen_id: usize) -> bool {
        if screen_id < MAX_SCREENS && self.screens[screen_id].is_some() {
            self.active_screen_id = screen_id;
            self.flush_to_physical();
            self.update_cursor();
            true
        } else {
            false
        }
    }
} 