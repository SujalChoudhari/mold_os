use mold_os::{
    clrscr,
    console::{get_line, get_word},
    print, println, resetcolor, setcolor, warn,
};

pub fn start() {
    clrscr!();
    println!("Welcome to Mold OS!");


}

pub fn run() {
    loop {
        print!(">> ");
        let command = get_line();

        setcolor!(
            mold_os::vga_buffer::Color::DarkGray,
            mold_os::vga_buffer::Color::LightGreen
        );
        print!("{}", command);
        resetcolor!();
        println!();

        if command.as_str() == "bye" {
            break;
        }
    }
}

pub fn end() {
    println!("Quitting OS!")
}
