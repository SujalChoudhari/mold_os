use mold_os::{clrscr, console::get_word, print, println};

pub fn start() {
    clrscr!();
    println!("Welcome to Mold OS!");
    
}

pub fn run() {
    loop {
        print!(">> ");
        let command = get_word();
        println!("\n>> {}", command);

        if command.as_str() == "bye" {
            break;
        }
    }
}

pub fn end() {
    println!("Quitting OS!")
}
