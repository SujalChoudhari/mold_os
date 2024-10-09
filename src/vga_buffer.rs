use core::fmt;
use volatile::Volatile;

#[macro_export]
macro_rules! log {
    () => {{
        $crate::setcolor!($crate::vga_buffer::Color::Cyan, $crate::vga_buffer::Color::Black); // Set color to cyan
        $crate::print!("[INFO]\n");
        $crate::_reset_color(); // Reset color after logging
    }};
    ($($arg:tt)*) => {{
        $crate::setcolor!($crate::vga_buffer::Color::Cyan, $crate::vga_buffer::Color::Black); // Set color to cyan
        $crate::print!("[INFO] {}\n", format_args!($($arg)*));
        $crate::vga_buffer::_reset_color(); // Reset color after logging
    }};
}

#[macro_export]
macro_rules! warn {
    () => {{
        $crate::setcolor!($crate::vga_buffer::Color::Yellow, $crate::vga_buffer::Color::Black); // Set color to yellow
        $crate::print!("[WARN]\n");
        $crate::_reset_color(); // Reset color after warning
    }};
    ($($arg:tt)*) => {{
        $crate::setcolor!($crate::vga_buffer::Color::Yellow, $crate::vga_buffer::Color::Black); // Set color to yellow
        $crate::print!("[WARN] {}\n", format_args!($($arg)*));
        $crate::vga_buffer::_reset_color(); // Reset color after warning
    }};
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

#[macro_export]
macro_rules! clrscr {
    () => {
        $crate::vga_buffer::_clrscr(); // Call the _clrscr function to clear the screen
    };
}

#[macro_export]
macro_rules! setcolor {
    ($fg:expr, $bg:expr) => {
        $crate::vga_buffer::_setcolor($fg, $bg); // Call the _setcolor function
    };
}

#[macro_export]
macro_rules! resetcolor {
    () => {
        $crate::vga_buffer::_reset_color(); // Call the _setcolor function
    };
}

/// Prints the given formatted string to the VGA text buffer
/// through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _clrscr() {
    // Lock the writer and call the clear method
    let mut writer = WRITER.lock();
    writer.clear();
}

pub fn _setcolor(fg: Color, bg: Color) {
    let color_code: ColorCode = ColorCode::new(fg, bg);
    WRITER.lock().color_code = color_code; // Set the color code in the locked writer
}

pub fn _reset_color() {
    let color_code: ColorCode = ColorCode::new(Color::White, Color::Black);
    WRITER.lock().color_code = color_code; // Reset to white on black
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        let code = (background as u8) << 4 | (foreground as u8);
        ColorCode(code)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer, // valid for entire runtime of program
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(), // Handle Newline
            b'\x08' => {
                // Handle backspace
                if self.column_position > 0 {
                    // Move the cursor one position to the left
                    self.column_position -= 1;

                    // Clear the character at the new cursor position
                    let row = BUFFER_HEIGHT - 1;
                    let col = self.column_position;
                    let color_code = self.color_code;

                    // Replace the character with a space
                    self.buffer.chars[row][col].write(ScreenChar {
                        ascii_character: b' ', // Clear with space
                        color_code,
                    });
                }
            }
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row); // Clear each row
        }
        self.column_position = 0; // Reset column position to 0 after clearing
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e => self.write_byte(byte),
                b'\n' => self.write_byte(byte),
                b'\x08' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => {
                    self.write_byte(b'@');
                }
            }
        }
    }

    pub fn write_at(&mut self, row: usize, col: usize, character: u8) {
        if row < BUFFER_HEIGHT && col < BUFFER_WIDTH {
            let color_code = self.color_code;
            self.buffer.chars[row][col].write(ScreenChar {
                ascii_character: character,
                color_code,
            });
        }
    }

    /// Write a string at a specific position
    pub fn write_string_at(&mut self, row: usize, col: usize, s: &str) {
        let mut current_col = col;
        for byte in s.bytes() {
            if current_col >= BUFFER_WIDTH {
                break; // Avoid overflowing to the next line
            }
            self.write_at(row, current_col, byte);
            current_col += 1;
        }
    }

    /// Draw a horizontal line from (start_col, row) to (end_col, row)
    pub fn draw_horizontal_line(&mut self, row: usize, start_col: usize, end_col: usize) {
        if row >= BUFFER_HEIGHT {
            return;
        }
        let start = start_col.min(BUFFER_WIDTH);
        let end = end_col.min(BUFFER_WIDTH);

        for col in start..end {
            self.write_at(row, col, b'-'); // Use '-' for horizontal line
        }
    }

    /// Draw a vertical line from (col, start_row) to (col, end_row)
    pub fn draw_vertical_line(&mut self, col: usize, start_row: usize, end_row: usize) {
        if col >= BUFFER_WIDTH {
            return;
        }
        let start = start_row.min(BUFFER_HEIGHT);
        let end = end_row.min(BUFFER_HEIGHT);

        for row in start..end {
            self.write_at(row, col, b'|'); // Use '|' for vertical line
        }
    }

    /// Draw a rectangle (box) from (start_row, start_col) to (end_row, end_col)
    pub fn draw_box(&mut self, start_row: usize, start_col: usize, end_row: usize, end_col: usize) {
        self.draw_horizontal_line(start_row, start_col, end_col); // Top edge
        self.draw_horizontal_line(end_row, start_col, end_col); // Bottom edge
        self.draw_vertical_line(start_col, start_row, end_row); // Left edge
        self.draw_vertical_line(end_col, start_row, end_row); // Right edge

        // Draw corners
        self.write_at(start_row, start_col, b'+'); // Top-left corner
        self.write_at(start_row, end_col, b'+'); // Top-right corner
        self.write_at(end_row, start_col, b'+'); // Bottom-left corner
        self.write_at(end_row, end_col, b'+'); // Bottom-right corner
    }
}

use lazy_static::lazy_static;
use spin::Mutex;

use crate::serial_print;
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
