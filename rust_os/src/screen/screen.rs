use core::fmt::{Write, Result};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: u8,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    pub chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            chars: [[ScreenChar {
                ascii_character: b' ',
                color_code: 0x0f,
            }; BUFFER_WIDTH]; BUFFER_HEIGHT],
        }
    }
}

pub struct Screen {
    pub id: usize,
    pub column_position: usize,
    pub row_position: usize,
    pub buffer: Buffer,
}

impl Screen {
    pub fn new(id: usize) -> Self {
        Screen {
            id,
            column_position: 0,
            row_position: 0,
            buffer: Buffer::new()
        }
    }
    
    pub fn clear(&mut self) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: 0x0f
        };
        
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col] = blank;
            }
        }
        
        self.column_position = 0;
        self.row_position = 0;
    }

    pub fn column_position(&self) -> usize {
        self.column_position
    }
    
    pub fn row_position(&self) -> usize {
        self.row_position
    }
    
    pub fn set_column_position(&mut self, pos: usize) {
        self.column_position = pos;
    }
    
    pub fn set_row_position(&mut self, pos: usize) {
        self.row_position = pos;
    }
    
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub fn set_cursor_position(&mut self, row: usize, col: usize) {
        self.row_position = row;
        self.column_position = col;
    }
}

pub struct Writer<'a> {
    screen: &'a mut Screen,
}

impl<'a> Writer<'a> {
    pub fn new(screen: &'a mut Screen) -> Self {
        Self { screen }
    }

    pub fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' {
            self.screen.row_position += 1;
            self.screen.column_position = 0;
        } else {
            if self.screen.row_position >= BUFFER_HEIGHT {
                self.scroll_up();
                self.screen.row_position = BUFFER_HEIGHT - 1;
            }

            self.screen.buffer.chars[self.screen.row_position][self.screen.column_position] =
                ScreenChar {
                    ascii_character: byte,
                    color_code: 0x0f,
                };

            self.screen.column_position += 1;
            if self.screen.column_position >= BUFFER_WIDTH {
                self.screen.column_position = 0;
                self.screen.row_position += 1;
            }
        }
    }

    pub fn scroll_up(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            self.screen.buffer.chars[row - 1] = self.screen.buffer.chars[row];
        }

        self.screen.buffer.chars[BUFFER_HEIGHT - 1] =
            [ScreenChar {
                ascii_character: b' ',
                color_code: 0x0f,
            }; BUFFER_WIDTH];
    }
}

impl<'a> Write for Writer<'a> {
    fn write_str(&mut self, s: &str) -> Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
} 