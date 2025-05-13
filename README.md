# Mold OS

Mold OS is a hobby operating system written in Rust. It's currently in its early stages of development and serves as a learning project to explore operating system development concepts.

## Features

- Basic VGA text mode output.
- PS/2 keyboard input.
- Serial port output for debugging.
- Global Descriptor Table (GDT) and Interrupt Descriptor Table (IDT) initialization.
- Double fault handling using an Interrupt Stack Table (IST).
- Heap allocation using a linked list allocator.
- Simple maze game application.

## Building and Running

Mold OS requires a nightly Rust toolchain and some additional tools like `bootimage` and `qemu`.

1. **Install the nightly Rust toolchain:**

   ```bash
   rustup toolchain install nightly
   ```

2. **Add the `nightly` toolchain to the project:**

   ```bash
   rustup override set nightly
   ```
   This command sets the nightly toolchain as the default one for the current project.

3. **Install `bootimage`:**

   ```bash
   cargo install bootimage
   ```

4. **Install `qemu`:**

   The installation method for `qemu` varies depending on your operating system.  Consult the QEMU documentation for specific instructions: [https://www.qemu.org/download/](https://www.qemu.org/download/)

   On Linux systems you can typically use your package manager (e.g., apt, pacman) to install the `qemu-system-x86_64` package. For example, on Debian-based systems:
   ```bash
   sudo apt install qemu-system-x86_64
   ```

5. **Build the bootimage:**
    To make the build process easier, you can use the provided Forge build commands, or alternatively, execute the full build command as follows:
   ```bash
   cargo bootimage
   ```
6. **Run in QEMU:**
    Similar to the build command, you can use the Forge run commands.  Alternatively, you can execute the full command as follows:
   ```bash
   qemu-system-x86_64 -drive format=raw,file=.\target\x86_64-mold_os\debug\bootimage-mold_os.bin
   ```

## Running Tests

Mold OS uses a custom test framework. Tests are located in the `tests` directory.

1. **Run the tests:**

   ```bash
   cargo test
   ```

## Maze Game

Mold OS includes a simple maze game. The player (`@`) navigates the maze using WASD keys, searching for chests (`$`), fighting monsters (`M`), and looking for the exit (`V`). The game features a fog of war mechanic, limiting the player's visibility.


## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.


