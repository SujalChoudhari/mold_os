use crate::vga_buffer::{clear_screen, draw_box, write_text_at};
use alloc::{format, string::ToString};
use bootloader::BootInfo;
use mold_os::{clrscr, console::get_char, print, println};
use x86_64::VirtAddr;

// Constants for the maze
const MAZE_WIDTH: usize = 79; // Updated width
const MAZE_HEIGHT: usize = 24; // Updated height

const PLAYER_CHAR: char = '@';
const WALL_CHAR: char = '#';
const EXPLORED_CHAR: char = ' '; // Visible explored area
const UNEXPLORED_CHAR: char = '?'; // Fog
const CHEST_CHAR: char = '$';
const MONSTER_CHAR: char = 'M';
const EXIT_CHAR: char = 'V';

// Structure to hold game state
struct GameState {
    player_x: usize,
    player_y: usize,
    maze: [[char; MAZE_WIDTH]; MAZE_HEIGHT],
}

impl GameState {
    fn new() -> Self {
        GameState {
            player_x: 1,
            player_y: 1,
            maze: initialize_maze(1), // Start level set to 1
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
    // Initialize the maze with walls around the edges
    let mut maze: [[char; MAZE_WIDTH]; MAZE_HEIGHT] = [['#'; MAZE_WIDTH]; MAZE_HEIGHT];

    // Fill the inner maze area with unexplored spaces
    for row in 1..MAZE_HEIGHT - 1 {
        for col in 1..MAZE_WIDTH - 1 {
            maze[row][col] = UNEXPLORED_CHAR;
        }
    }

    // Randomly place chests, monsters, and exit using modular arithmetic
    let mut rng = Rng::new(level as u32); // Use the level number as the seed

    // Place a chest at a random position
    let chest_x = 1 + (rng.next_range((MAZE_WIDTH - 2) as u32) / 2) * 2;
    let chest_y = 1 + (rng.next_range((MAZE_HEIGHT - 2) as u32) / 2) * 2;
    maze[chest_y][chest_x] = CHEST_CHAR;

    // Place a monster at a random position
    let monster_x = 1 + (rng.next_range((MAZE_WIDTH - 2) as u32) / 2) * 2;
    let monster_y = 1 + (rng.next_range((MAZE_HEIGHT - 2) as u32) / 2) * 2;
    maze[monster_y][monster_x] = MONSTER_CHAR;

    // Place the exit at a random position
    let exit_x = 1 + (rng.next_range((MAZE_WIDTH - 2) as u32) / 2) * 2;
    let exit_y = 1 + (rng.next_range((MAZE_HEIGHT - 2) as u32) / 2) * 2;
    maze[exit_y][exit_x] = EXIT_CHAR;

    // Set the player's starting position
    maze[1][1] = PLAYER_CHAR; // Set player at starting position

    maze
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
    println!("Welcome to Mold OS!");
}

pub fn run() {
    let mut game_state = GameState::new();

    loop {
        clear_and_draw_maze(&mut game_state);
        handle_player_input(&mut game_state);
    }
}

fn clear_and_draw_maze(game_state: &mut GameState) {
    clrscr!();
    for row in 0..MAZE_HEIGHT {
        for col in 0..MAZE_WIDTH {
            let ch = if row == game_state.player_y && col == game_state.player_x {
                PLAYER_CHAR // Draw player character
            } else if is_within_view(row, col, game_state.player_x, game_state.player_y) {
                game_state.maze[row][col] // Reveal adjacent cells
            } else {
                UNEXPLORED_CHAR // Fog of war
            };
            write_text_at(row, col, &ch.to_string());
        }
    }
}

// Function to check if a cell is within the view distance of the player
fn is_within_view(row: usize, col: usize, player_x: usize, player_y: usize) -> bool {
    let dx = (col as isize - player_x as isize).abs();
    let dy = (row as isize - player_y as isize).abs();
    dx <= 2 && dy <= 2 // Check if within two cells in both directions
}

fn handle_player_input(game_state: &mut GameState) {
    // Get the character input for movement
    let input = get_char();

    // Calculate new player position based on input
    let (new_x, new_y) = match input {
        'w' => (game_state.player_x, game_state.player_y.saturating_sub(1)), // Move up
        's' => (game_state.player_x, game_state.player_y + 1),               // Move down
        'a' => (game_state.player_x.saturating_sub(1), game_state.player_y), // Move left
        'd' => (game_state.player_x + 1, game_state.player_y),               // Move right
        ' ' => {
            // Check if player is on an entity and open the menu
            if let Some(entity) =
                get_entity_at(game_state, game_state.player_x, game_state.player_y)
            {
                open_interaction_menu(entity);
            }
            (game_state.player_x, game_state.player_y) // No movement
        }
        _ => (game_state.player_x, game_state.player_y), // No movement
    };

    // Check for walls and update player position if it's valid
    if game_state.maze[new_y][new_x] != WALL_CHAR {
        game_state.player_x = new_x;
        game_state.player_y = new_y;

        // Reveal the area around the player when they move
        reveal_area(game_state, new_x, new_y);
    }
}

// Function to get the entity at the given position
fn get_entity_at(game_state: &GameState, x: usize, y: usize) -> Option<char> {
    match game_state.maze[y][x] {
        CHEST_CHAR => Some(CHEST_CHAR),
        MONSTER_CHAR => Some(MONSTER_CHAR),
        EXIT_CHAR => Some(EXIT_CHAR),
        _ => None,
    }
}

fn open_interaction_menu(entity: char) {
    clrscr!();

    let actions: [&'static str; 3] = match entity {
        CHEST_CHAR => ["Open Chest", "Leave", "Inspect"],
        MONSTER_CHAR => ["Attack", "Flee", "Inspect"],
        EXIT_CHAR => ["Enter", "Leave", "Inspect"],
        _ => ["Leave", "Inspect", "Done"],
    };

    // Display menu
    let menu_height = actions.len() + 2; // Add 2 for title and border
    let start_row = (MAZE_HEIGHT - menu_height) / 2;
    let start_col = (MAZE_WIDTH - 20) / 2;

    draw_box(
        start_row,
        start_col,
        start_row + menu_height,
        start_col + 19,
    );
    write_text_at(start_row + 1, start_col + 1, "Select an action:");

    // Initial draw of the actions without selection indicator
    for (i, &action) in actions.iter().enumerate() {
        write_text_at(start_row + 2 + i, start_col + 1, action);
    }

    // Handle input to select an action
    let selection = get_menu_selection(&actions, actions.len(), start_row, start_col);
    handle_menu_selection(selection, entity);
}

fn get_menu_selection(
    actions: &[&'static str],
    num_actions: usize,
    start_row: usize,
    start_col: usize,
) -> usize {
    let mut selection = 0; // Start with the first action
    loop {
        // Highlight selected option and redraw all actions
        for i in 0..num_actions {
            let action = if i == selection {
                format!(">> {} <<", actions[i]) // Highlight selected option
            } else {
                format!("   {}   ", actions[i]) // Clear indicator for others
            };
            write_text_at(start_row + 2 + i, start_col + 1, &action);
        }

        // Get user input for menu navigation
        let input = get_char();
        match input {
            'w' => selection = selection.saturating_sub(1), // Move up
            's' => selection = (selection + 1).min(num_actions - 1), // Move down
            ' ' => break,                                   // Select action
            _ => {}
        }

        // Ensure selection wraps around
        if selection >= num_actions {
            selection = num_actions - 1;
        }
    }
    selection
}

fn handle_menu_selection(selection: usize, entity: char) {
    match entity {
        CHEST_CHAR => match selection {
            0 => println!("You opened the chest!"),
            1 => println!("You left the chest."),
            2 => println!("You inspected the chest."),
            _ => {}
        },
        MONSTER_CHAR => match selection {
            0 => println!("You attacked the monster!"),
            1 => println!("You fled from the monster."),
            2 => println!("You inspected the monster."),
            _ => {}
        },
        EXIT_CHAR => match selection {
            0 => println!("You entered the exit!"),
            1 => println!("You left the exit."),
            2 => println!("You inspected the exit."),
            _ => {}
        },
        _ => {}
    }
}

fn reveal_area(game_state: &mut GameState, new_x: usize, new_y: usize) {
    // Reveal cells in a 2-cell radius around the player
    for row in new_y.saturating_sub(2)..=(new_y + 2).min(MAZE_HEIGHT - 1) {
        for col in new_x.saturating_sub(2)..=(new_x + 2).min(MAZE_WIDTH - 1) {
            if game_state.maze[row][col] == UNEXPLORED_CHAR {
                game_state.maze[row][col] = EXPLORED_CHAR; // Set the cell to explored
            }
        }
    }
}

pub fn end() {
    println!("Quitting OS!");
}
