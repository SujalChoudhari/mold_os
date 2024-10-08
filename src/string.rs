use core::fmt;

/// A simple, fixed-size String-like struct for no_std environments.
/// Uses a fixed-size buffer and implements basic string operations.
pub struct String {
    buffer: [u8; 256], // Fixed-size buffer
    len: usize,        // Current length of the string
}

impl String {
    /// Create a new empty string
    pub fn new() -> Self {
        Self {
            buffer: [0; 256], // Initialize the buffer with zeros
            len: 0,           // Start with zero length
        }
    }

    /// Create a string from a character slice
    pub fn from_str(s: &str) -> Self {
        let mut string = Self::new();
        string.push_str(s);
        string
    }

    /// Push a character to the end of the string
    pub fn push(&mut self, ch: char) {
        let mut buf = [0; 4]; // Buffer to hold the UTF-8 bytes of the character
        let char_len = ch.encode_utf8(&mut buf).len(); // Get the UTF-8 bytes of the char

        if self.len + char_len <= self.buffer.len() {
            for &byte in &buf[0..char_len] {
                self.buffer[self.len] = byte;
                self.len += 1;
            }
        }
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.len == 0 {
            return None; // Return None if the string is empty
        }

        // Find the start of the last UTF-8 character
        let mut char_start = self.len;

        // UTF-8 uses continuation bytes (leading bits `10xxxxxx`), so we find the first byte
        // of the last character by looking for a byte that doesn't start with `10`.
        while char_start > 0 {
            char_start -= 1;
            if (self.buffer[char_start] & 0b1100_0000) != 0b1000_0000 {
                break;
            }
        }

        // Decode the last character from the found position
        let last_char = core::str::from_utf8(&self.buffer[char_start..self.len])
            .ok()?
            .chars()
            .next()?;

        // Update the length to exclude the last character
        self.len = char_start;

        // Return the removed character
        Some(last_char)
    }

    /// Append a string slice to the string
    pub fn push_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.push(ch);
        }
    }

    /// Get the length of the string
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clear the string
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Convert the string into a &str slice
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buffer[..self.len]).unwrap_or("")
    }
}

impl core::fmt::Write for String {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

// Implement Debug trait for the String
impl fmt::Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("String")
            .field("content", &self.as_str())
            .field("len", &self.len)
            .finish()
    }
}

// Implement Display trait for the String
impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Implement PartialEq trait for the String to enable string comparisons
impl PartialEq for String {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

// Implement Eq trait for String (since PartialEq is implemented, Eq can be derived)
impl Eq for String {}

// Implement Clone trait for the String
impl Clone for String {
    fn clone(&self) -> Self {
        let mut new_string = Self::new();
        new_string.push_str(self.as_str());
        new_string
    }
}

// Implement Default trait to allow creating an empty String using `String::default()`
impl Default for String {
    fn default() -> Self {
        Self::new()
    }
}
