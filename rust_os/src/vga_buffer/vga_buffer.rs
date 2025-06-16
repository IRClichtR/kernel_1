// use core::fmt;
// use spin::Mutex;
// use lazy_static::lazy_static;
// // port
// use crate::arch::x86::port::{outb};

// #[repr(C)]
// #[derive(Copy, Clone)]
// struct Char {
//     ascii_character: u8,
//     color_code: u8,
// }

// const BUFFER_HEIGHT: usize = 25;
// const BUFFER_WIDTH: usize = 80;

// #[repr(transparent)]
// struct Buffer {
//     chars: [[Char; BUFFER_WIDTH]; BUFFER_HEIGHT]
// }

// pub struct Writer {
//     column_position: usize,
//     row_position: usize,
//     buffer: &'static mut Buffer,
// }

// pub struct Writer<'a> {
//     screen: &'a mut Screen,
// }

// // Global kernel writer
// lazy_static! {
//     pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
// }

// impl Writer {
//     pub fn new() -> Self {
//         Writer {
//             column_position: 0,
//             row_position: 0,
//             buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
//         }
//     }

//     pub fn write_byte(&mut self, byte: u8) {
//         match byte {
//             b'\n' => self.new_line(),
//             byte => {
//                 if self.column_position >= BUFFER_WIDTH {
//                     self.new_line();
//                 }
                
//                 let row = self.row_position;
//                 let col = self.column_position;
    
//                 self.buffer.chars[row][col] = Char {
//                     ascii_character: byte,
//                     color_code: 0x0f,
//                 };
                
//                 self.column_position += 1;

//                 if self.column_position >= BUFFER_WIDTH {
//                     self.new_line();
//                 }
//                 self.set_cursor_position(self.row_position, self.column_position);
//             }
//         }
//     }

//     pub fn write_string(&mut self, s: &str) {
//         for byte in s.bytes() {
//             match byte {
//                 // printable ASCII byte or newline
//                 0x20..=0x7e | b'\n' => self.write_byte(byte),
//                 // not part of printable ASCII range
//                 _ => self.write_byte(0xfe),
//             }
//         }
//     }

//     fn new_line(&mut self) {
//         if self.row_position < BUFFER_HEIGHT - 1 {
//             self.row_position += 1;
//         } else {
//             for row in 1..BUFFER_HEIGHT {
//                 for col in 0..BUFFER_WIDTH {
//                     let character = self.buffer.chars[row][col];
//                     self.buffer.chars[row - 1][col] = character;
//                 }
//             }
//             self.clear_row(BUFFER_HEIGHT - 1);
//         }
//         self.column_position = 0;
//     }
        
//     fn clear_row(&mut self, row: usize) {
//         let blank = Char {
//             ascii_character: b' ',
//             color_code: 0x0f,
//         };
        
//         for col in 0..BUFFER_WIDTH {
//             self.buffer.chars[row][col] = blank;
//         }
//     }

//     pub fn set_cursor_position(&mut self, row: usize, col: usize) {
//         let row = row.min(BUFFER_HEIGHT - 1);
//         let col = col.min(BUFFER_WIDTH - 1);
    
//         let pos: u16 = (row * BUFFER_WIDTH + col) as u16;
//         unsafe {
//             outb(0x3D4, 0x0F);
//             outb(0x3D5, (pos & 0xFF) as u8);
    
//             outb(0x3D4, 0x0E);
//             outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
//         }
//     }    
// }

// impl fmt::Write for Writer {
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         self.write_string(s);
//         Ok(())
//     }
// }

// #[doc(hidden)]
// pub fn _print(args: fmt::Arguments) {
//     use core::fmt::Write;
//     WRITER.lock().write_fmt(args).unwrap();
// }

// #[macro_export]
// macro_rules! print {
//     ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
// }

// #[macro_export]
// macro_rules! println {
//     () => ($crate::print!("\n"));
//     ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
// }

use core::fmt;
use spin::Mutex;
use lazy_static::lazy_static;
// port
use crate::arch::x86::port::{outb};

#[repr(C)]
#[derive(Copy, Clone)]
struct ScreenChar {
    ascii_character: u8,
    color_code: u8,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

pub struct Writer {
    column_position: usize,
    row_position: usize,
    buffer: &'static mut Buffer,
}

// Global kernel writer
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
}

impl Writer {
    pub fn new() -> Self {
        Writer {
            column_position: 0,
            row_position: 0,
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                
                let row = self.row_position;
                let col = self.column_position;
    
                self.buffer.chars[row][col] = ScreenChar {
                    ascii_character: byte,
                    color_code: 0x0f,
                };
                
                self.column_position += 1;

                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                self.set_cursor_position(self.row_position, self.column_position);
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col];
                    self.buffer.chars[row - 1][col] = character;
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
        }
        self.column_position = 0;
    }
        
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: 0x0f,
        };
        
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col] = blank;
        }
    }

    pub fn set_cursor_position(&mut self, row: usize, col: usize) {
        let row = row.min(BUFFER_HEIGHT - 1);
        let col = col.min(BUFFER_WIDTH - 1);
    
        let pos: u16 = (row * BUFFER_WIDTH + col) as u16;
        unsafe {
            outb(0x3D4, 0x0F);
            outb(0x3D5, (pos & 0xFF) as u8);
    
            outb(0x3D4, 0x0E);
            outb(0x3D5, ((pos >> 8) & 0xFF) as u8);
        }
    }    
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}