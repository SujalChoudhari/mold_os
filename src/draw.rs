use mold_os::vga_buffer::{Color, BUFFER_HEIGHT, BUFFER_WIDTH, WRITER};
use spin::Mutex;

use core::cmp::{max, min};
use core::fmt::Write;

/// A struct to handle drawing operations
///
pub static SCREEN_CHANGED: Mutex<bool> = Mutex::new(true);
pub struct Draw {
    fg_color: Color, // Foreground (text) color
    bg_color: Color, // Background color
}

impl Draw {
    /// Create a new `Draw` instance with default colors (White on Black)
    pub fn new() -> Self {
        Draw {
            fg_color: Color::White,
            bg_color: Color::Black,
        }
    }

    /// Change the foreground (text) color
    pub fn set_fg_color(&mut self, color: Color) {
        self.fg_color = color;
    }

    /// Change the background color
    pub fn set_bg_color(&mut self, color: Color) {
        self.bg_color = color;
    }

    /// Plot a single point at (x, y) using the current foreground and background colors
    pub fn plot(&self, x: usize, y: usize, character: u8) {
        *SCREEN_CHANGED.lock() = true;
        let mut writer = WRITER.lock();
        if x < BUFFER_WIDTH && y < BUFFER_HEIGHT {
            writer.buffer.chars[y][x].write(mold_os::vga_buffer::ScreenChar {
                ascii_character: character,
                color_code: mold_os::vga_buffer::ColorCode::new(self.fg_color, self.bg_color),
            });
        }
    }

    /// Draw a horizontal line using '─' characters
    pub fn draw_horizontal_line(&self, x0: usize, x1: usize, y: usize) {
        for x in x0..=x1 {
            self.plot(x, y, 0xC4); // ASCII value of '─'
        }
    }

    /// Draw a vertical line using '│' characters
    pub fn draw_vertical_line(&self, y0: usize, y1: usize, x: usize) {
        for y in y0..=y1 {
            self.plot(x, y, 0xB3); // ASCII value of '│'
        }
    }

    /// Bresenham's Line Algorithm for diagonal or arbitrary lines
    pub fn draw_diagonal_line(&self, x0: usize, y0: usize, x1: usize, y1: usize) {
        let mut x0 = x0 as isize;
        let mut y0 = y0 as isize;
        let x1 = x1 as isize;
        let y1 = y1 as isize;

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let mut err = dx + dy;

        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };

        loop {
            self.plot(x0 as usize, y0 as usize, b'*'); // Fallback for diagonal lines
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// Draw a line, choosing the appropriate method (horizontal, vertical, or diagonal)
    pub fn draw_line(&self, x0: usize, y0: usize, x1: usize, y1: usize) {
        if y0 == y1 {
            // Horizontal line
            self.draw_horizontal_line(min(x0, x1), max(x0, x1), y0);
        } else if x0 == x1 {
            // Vertical line
            self.draw_vertical_line(min(y0, y1), max(y0, y1), x0);
        } else {
            // Diagonal or arbitrary line
            self.draw_diagonal_line(x0, y0, x1, y1);
        }
    }

    /// Draw a box using line drawing characters
    pub fn draw_box(&self, x0: usize, y0: usize, x1: usize, y1: usize) {
        let x_min = min(x0, x1);
        let x_max = max(x0, x1);
        let y_min = min(y0, y1);
        let y_max = max(y0, y1);

        // Corners
        self.plot(x_min, y_min, 0xDA); // '┌'
        self.plot(x_max, y_min, 0xBF); // '┐'
        self.plot(x_min, y_max, 0xC0); // '└'
        self.plot(x_max, y_max, 0xD9); // '┘'

        // Horizontal lines
        self.draw_horizontal_line(x_min + 1, x_max - 1, y_min);
        self.draw_horizontal_line(x_min + 1, x_max - 1, y_max);

        // Vertical lines
        self.draw_vertical_line(y_min + 1, y_max - 1, x_min);
        self.draw_vertical_line(y_min + 1, y_max - 1, x_max);
    }

    /// Draw text at a specified (x, y) position
    pub fn draw_text(&self, x: usize, y: usize, text: &str) {
        *SCREEN_CHANGED.lock() = true;
        let mut writer = WRITER.lock();
        let mut col = x;

        for byte in text.bytes() {
            if col >= BUFFER_WIDTH {
                break; // Stop if the text goes beyond the screen width
            }
            writer.buffer.chars[y][col].write(mold_os::vga_buffer::ScreenChar {
                ascii_character: byte,
                color_code: mold_os::vga_buffer::ColorCode::new(self.fg_color, self.bg_color),
            });
            col += 1;
        }
    }

    /// Draw text box with wrapping within a box defined by (x0, y0) to (x1, y1)
    pub fn draw_text_box(&self, x0: usize, y0: usize, x1: usize, y1: usize, text: &str) {
        *SCREEN_CHANGED.lock() = true;
        let mut writer = WRITER.lock();
        let mut col = x0 + 1;
        let mut row = y0 + 1;

        for byte in text.bytes() {
            if row > y1 - 1 {
                break;
            }

            if col > x1 - 1 || byte == b'\n' {
                row += 1;
                col = x0;
                if row > y1 {
                    break;
                }
                if byte == b'\n' {
                    continue;
                }
            }

            writer.buffer.chars[row][col].write(mold_os::vga_buffer::ScreenChar {
                ascii_character: byte,
                color_code: mold_os::vga_buffer::ColorCode::new(self.fg_color, self.bg_color),
            });

            col += 1;
        }
    }
}
