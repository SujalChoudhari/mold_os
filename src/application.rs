use crate::vga_buffer::{clear_screen, draw_box, write_text_at};
use alloc::{
    format,
    string::{String, ToString},
};
use bootloader::BootInfo;
use mold_os::{clrscr, console::get_char, print, println, setcolor};
use x86_64::VirtAddr;

// Constants for the maze
const MAZE_WIDTH: usize = 79;
const MAZE_HEIGHT: usize = 24;

const PLAYER_CHAR: char = '@';
const WALL_CHAR: char = '#';
const EXPLORED_CHAR: char = ' ';
const UNEXPLORED_CHAR: char = '?';
const CHEST_CHAR: char = '$';
const MONSTER_CHAR: char = 'M';
const EXIT_CHAR: char = 'V';

const FOG_NEAR: char = '.';
const FOG_MID: char = ',';
const FOG_FAR: char = '*';

// Structure to hold game state
struct GameState {
    player: Player,
    maze: [[char; MAZE_WIDTH]; MAZE_HEIGHT],
    level: usize,
}

struct Player {
    x: usize,
    y: usize,
    health: i32,
    max_health: i32,
    xp: i32,
    sword_level: i32,
}

struct Monster {
    health: i32,
    xp_reward: i32,
}

impl GameState {
    fn new() -> Self {
        GameState {
            player: Player {
                x: 1,
                y: 1,
                health: 100,
                max_health: 100,
                xp: 0,
                sword_level: 1,
            },
            maze: initialize_maze(1),
            level: 1,
        }
    }
}

// Random number generator state
struct Rng {
    seed: u32,
}

impl Rng {
    fn new(seed: u32) -> Self {
        Rng { seed }
    }

    fn next(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345) & 0x7FFFFFFF;
        self.seed
    }

    fn next_range(&mut self, range: u32) -> usize {
        (self.next() % range) as usize
    }
}

fn initialize_maze(level: usize) -> [[char; MAZE_WIDTH]; MAZE_HEIGHT] {
    let mut maze: [[char; MAZE_WIDTH]; MAZE_HEIGHT] = [['#'; MAZE_WIDTH]; MAZE_HEIGHT];

    for row in 1..MAZE_HEIGHT - 1 {
        for col in 1..MAZE_WIDTH - 1 {
            maze[row][col] = UNEXPLORED_CHAR;
        }
    }

    let mut rng = Rng::new(level as u32);

    // Place chests
    for _ in 0..level {
        let chest_x = 1 + (rng.next_range((MAZE_WIDTH - 2) as u32) / 2) * 2;
        let chest_y = 1 + (rng.next_range((MAZE_HEIGHT - 2) as u32) / 2) * 2;
        maze[chest_y][chest_x] = CHEST_CHAR;
    }

    // Place monsters
    for _ in 0..level * 2 {
        let monster_x = 1 + (rng.next_range((MAZE_WIDTH - 2) as u32) / 2) * 2;
        let monster_y = 1 + (rng.next_range((MAZE_HEIGHT - 2) as u32) / 2) * 2;
        maze[monster_y][monster_x] = MONSTER_CHAR;
    }

    // Place the exit
    let exit_x = 1 + (rng.next_range((MAZE_WIDTH - 2) as u32) / 2) * 2;
    let exit_y = 1 + (rng.next_range((MAZE_HEIGHT - 2) as u32) / 2) * 2;
    maze[exit_y][exit_x] = EXIT_CHAR;

    maze[1][1] = PLAYER_CHAR;

    maze
}

pub fn run() {
    let mut game_state = GameState::new();

    loop {
        clear_and_draw_maze(&mut game_state);
        draw_player_stats(&game_state);
        handle_player_input(&mut game_state);
    }
}

fn clear_and_draw_maze(game_state: &mut GameState) {
    clrscr!();
    for row in 0..MAZE_HEIGHT {
        for col in 0..MAZE_WIDTH {
            let ch = if row == game_state.player.y && col == game_state.player.x {
                PLAYER_CHAR
            } else {
                get_fog_char(
                    row,
                    col,
                    game_state.player.x,
                    game_state.player.y,
                    &game_state.maze,
                )
            };
            write_text_at(row, col, &ch.to_string());
        }
    }
}

fn get_fog_char(
    row: usize,
    col: usize,
    player_x: usize,
    player_y: usize,
    maze: &[[char; MAZE_WIDTH]; MAZE_HEIGHT],
) -> char {
    let dx = (col as isize - player_x as isize).abs();
    let dy = (row as isize - player_y as isize).abs();
    let distance = (dx * dx / 2 + dy * dy) as f32;

    if distance < 4.0 {
        maze[row][col]
    } else if distance < 49.0 {
        FOG_NEAR
    } else if distance < 81.0 {
        FOG_MID
    } else if distance < 100.0 {
        FOG_FAR
    } else {
        UNEXPLORED_CHAR
    }
}

fn draw_player_stats(game_state: &GameState) {
    let stats = format!(
        "Health: {}/{} | XP: {} | Sword Level: {} | Level: {}",
        game_state.player.health,
        game_state.player.max_health,
        game_state.player.xp,
        game_state.player.sword_level,
        game_state.level
    );
    setcolor!(
        mold_os::vga_buffer::Color::Cyan,
        mold_os::vga_buffer::Color::Black
    );
    write_text_at(MAZE_HEIGHT, 0, &stats);
    setcolor!(
        mold_os::vga_buffer::Color::White,
        mold_os::vga_buffer::Color::Black
    );
}

fn handle_player_input(game_state: &mut GameState) {
    let input = get_char();

    match input {
        'w' | 's' | 'a' | 'd' => move_player(game_state, input),
        'i' => inspect_surroundings(game_state),
        _ => {}
    }
}

fn move_player(game_state: &mut GameState, direction: char) {
    let (new_x, new_y) = match direction {
        'w' => (game_state.player.x, game_state.player.y.saturating_sub(1)),
        's' => (game_state.player.x, game_state.player.y + 1),
        'a' => (game_state.player.x.saturating_sub(1), game_state.player.y),
        'd' => (game_state.player.x + 1, game_state.player.y),
        _ => (game_state.player.x, game_state.player.y),
    };

    if game_state.maze[new_y][new_x] != WALL_CHAR {
        game_state.player.x = new_x;
        game_state.player.y = new_y;
        reveal_area(game_state, new_x, new_y);

        if let Some(entity) = get_entity_at(game_state, new_x, new_y) {
            handle_interaction(game_state, entity);
        }
    }
}

fn inspect_surroundings(game_state: &GameState) {
    let mut info = String::new();

    for dy in -1..=1 {
        for dx in -1..=1 {
            let x = game_state.player.x as isize + dx;
            let y = game_state.player.y as isize + dy;

            if x >= 0 && x < MAZE_WIDTH as isize && y >= 0 && y < MAZE_HEIGHT as isize {
                let entity = game_state.maze[y as usize][x as usize];
                match entity {
                    CHEST_CHAR => info.push_str("You see a chest nearby. "),
                    MONSTER_CHAR => info.push_str("A monster lurks in the shadows. "),
                    EXIT_CHAR => info.push_str("You spot the exit! "),
                    WALL_CHAR => info.push_str("There's a wall. "),
                    _ => {}
                }
            }
        }
    }

    if info.is_empty() {
        info = "There's nothing interesting nearby.".to_string();
    }

    display_info_box(&info);
}

fn display_info_box(info: &str) {
    clrscr!();
    write_text_at(11, 6, info);
    write_text_at(18, 30, "Press any key to continue...");
    get_char();
}

fn exit_menu(game_state: &mut GameState) {
    loop {
        display_info_box("You found the exit! 1. Proceed to next level 2. Stay on current level");
        match get_char() {
            '1' => {
                next_level(game_state);
                break;
            }
            '2' => break,
            _ => {}
        }
    }
}

fn handle_interaction(game_state: &mut GameState, entity: char) {
    match entity {
        CHEST_CHAR => chest_menu(game_state),
        MONSTER_CHAR => monster_menu(game_state),
        EXIT_CHAR => exit_menu(game_state),
        _ => {}
    }
}

fn chest_menu(game_state: &mut GameState) {
    loop {
        display_info_box("You found a chest! 1. Open 2. Leave");
        match get_char() {
            '1' => {
                open_chest(game_state);
                break;
            }
            '2' => break,
            _ => {}
        }
    }
}

fn monster_menu(game_state: &mut GameState) {
    loop {
        display_info_box("You encountered a monster! What do you want to do?\n1. Fight\n2. Flee");
        match get_char() {
            '1' => {
                fight_monster(game_state);
                break;
            }
            '2' => {
                flee_from_monster(game_state);
                break;
            }
            _ => {}
        }
    }
}

fn get_entity_at(game_state: &GameState, x: usize, y: usize) -> Option<char> {
    match game_state.maze[y][x] {
        CHEST_CHAR => Some(CHEST_CHAR),
        MONSTER_CHAR => Some(MONSTER_CHAR),
        EXIT_CHAR => Some(EXIT_CHAR),
        _ => None,
    }
}

fn flee_from_monster(game_state: &mut GameState) {
    let mut rng = Rng::new(game_state.level as u32);
    if rng.next_range(100) < 70 {
        display_info_box("You successfully fled from the monster!");
        // Move player to a random adjacent empty cell
        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        for _ in 0..4 {
            let (dx, dy) = directions[rng.next_range(4) as usize];
            let new_x = (game_state.player.x as isize + dx) as usize;
            let new_y = (game_state.player.y as isize + dy) as usize;
            if game_state.maze[new_y][new_x] == EXPLORED_CHAR {
                game_state.player.x = new_x;
                game_state.player.y = new_y;
                break;
            }
        }
    } else {
        display_info_box("You failed to flee! The monster attacks you.");
        game_state.player.health -= game_state.level as i32 * 5;
    }
}

fn open_chest(game_state: &mut GameState) {
    let mut rng = Rng::new(game_state.level as u32);
    let health_gain = rng.next_range(20) as i32 + 10;
    let xp_gain = rng.next_range(30) as i32 + 20;

    game_state.player.health =
        (game_state.player.health + health_gain).min(game_state.player.max_health);
    game_state.player.xp += xp_gain;

    println!(
        "You found a chest! Gained {} health and {} XP.",
        health_gain, xp_gain
    );
    game_state.maze[game_state.player.y][game_state.player.x] = EXPLORED_CHAR;
}

fn fight_monster(game_state: &mut GameState) {
    let mut monster = Monster {
        health: game_state.level as i32 * 20 + 10,
        xp_reward: game_state.level as i32 * 15 + 5,
    };

    println!("You encountered a monster! Fight begins!");

    loop {
        // Player's turn
        let player_damage = game_state.player.sword_level * 5 + 5;
        monster.health -= player_damage;
        println!("You dealt {} damage to the monster!", player_damage);

        if monster.health <= 0 {
            println!("You defeated the monster! Gained {} XP.", monster.xp_reward);
            game_state.player.xp += monster.xp_reward;
            game_state.maze[game_state.player.y][game_state.player.x] = EXPLORED_CHAR;
            break;
        }

        // Monster's turn
        let monster_damage = game_state.level * 2 + 5;
        game_state.player.health -= monster_damage as i32;
        println!("The monster dealt {} damage to you!", monster_damage);

        if game_state.player.health <= 0 {
            println!("Game Over! You were defeated by the monster.");
            // Handle game over logic here
            break;
        }
    }
}

fn next_level(game_state: &mut GameState) {
    game_state.level += 1;
    game_state.maze = initialize_maze(game_state.level);
    game_state.player.x = 1;
    game_state.player.y = 1;
    game_state.player.health = game_state.player.max_health;

    println!(
        "You reached the exit! Moving to level {}.",
        game_state.level
    );
    println!("Would you like to upgrade your sword? (y/n)");

    loop {
        let input = get_char();
        match input {
            'y' => {
                if game_state.player.xp >= 100 {
                    game_state.player.sword_level += 1;
                    game_state.player.xp -= 100;
                    println!("Sword upgraded to level {}!", game_state.player.sword_level);
                } else {
                    println!("Not enough XP to upgrade sword.");
                }
                break;
            }
            'n' => {
                println!("Sword not upgraded.");
                break;
            }
            _ => println!("Invalid input. Please enter 'y' or 'n'."),
        }
    }
}

fn reveal_area(game_state: &mut GameState, new_x: usize, new_y: usize) {
    for row in new_y.saturating_sub(2)..=(new_y + 2).min(MAZE_HEIGHT - 1) {
        for col in new_x.saturating_sub(2)..=(new_x + 2).min(MAZE_WIDTH - 1) {
            if game_state.maze[row][col] == UNEXPLORED_CHAR {
                game_state.maze[row][col] = EXPLORED_CHAR;
            }
        }
    }
}

pub fn start(boot_info: &'static BootInfo) {
    use mold_os::allocator;
    use mold_os::memory;
    use mold_os::memory::BootInfoFrameAllocator;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = memory::init(phys_mem_offset);
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    clrscr!();
    println!("Welcome to Mold OS Maze Game!");
}

pub fn end() {
    println!("Thanks for playing Mold OS Maze Game!");
}
