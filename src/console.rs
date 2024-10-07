use crate::interrupts::BUFFER;
use crate::print;
use crate::string::String;

/// Clears the BUFFER by setting it to '\0'
pub fn clear_buffer() {
    *BUFFER.lock() = '\0';
}

/// Generalized function to retrieve content from the BUFFER until any of the specified delimiters is encountered
fn get_from_stdin_with_delimiters(delimiters: &[char]) -> String {
    let mut output = String::new();

    loop {
        let buffer_content;
        {
            while BUFFER.is_locked() {
                // wait
            }
            // Lock the buffer
            if *BUFFER.lock() != '\0' {
                let buffer = BUFFER.lock();
                print!("{}", *buffer);
                buffer_content = *buffer;
            } else {
                continue;
            }
            clear_buffer();
        } // Unlock buffer

        // Break the loop if the buffer content matches any delimiter
        if delimiters.contains(&buffer_content) {
            break;
        }

        output.push(buffer_content);

        // Small delay to allow keyboard handler to process the next character
        for _ in 0..10000 {
            core::hint::spin_loop();
        }
    }

    output
}

/// Retrieves a single word from the BUFFER until a space or newline is encountered
pub fn get_word() -> String {
    get_from_stdin_with_delimiters(&[' ', '\n'])
}

/// Retrieves an entire line from the BUFFER until a newline is encountered
pub fn get_line() -> String {
    get_from_stdin_with_delimiters(&['\n'])
}
