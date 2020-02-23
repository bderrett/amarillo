use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::fmt;
use std::fmt::Display;

const NUM_FACTORY_DISPLAYS: u8 = 7;
// blue, yellow, red, green, cyan, first player token
pub const COLOR_NAMES: [char; 6] = ['B', 'Y', 'R', 'G', 'C', 'F'];
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Row {
    pub color: u8,
    pub count: u8,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct PlayerState {
    pub wall_state: [[bool; 5]; 5],
    pub rows: [Row; 5],
    pub score: i32,
    pub floor_tiles: [u8; 6],
}
#[cfg(unix)]
fn colorise_adv(string: &str, color_char: char) -> String {
    let rgb = match color_char {
        'B' => (0, 0, 255),
        'G' => (0, 255, 0),
        'Y' => (255, 255, 0),
        'C' => (0, 255, 255),
        'R' => (255, 0, 0),
        'F' => (128, 0, 128),
        _ => (255, 255, 255),
    };
    format!(
        "{}{}{}",
        termion::color::Fg(termion::color::Rgb(rgb.0, rgb.1, rgb.2)),
        string,
        termion::color::Fg(termion::color::Reset)
    )
}

#[cfg(not(unix))]
fn colorise_adv(string: &str, color_char: char) -> String {
    string.to_string()
}

fn colorise(color_char: char) -> String {
    colorise_adv(&color_char.to_string(), color_char)
}
fn color_arr<'a, I>(arr: I, compact: bool) -> String
where
    I: Iterator<Item = &'a u8>,
{
    let mut out_str = String::new();
    for (color, count) in arr.enumerate() {
        if *count == 0 {
            continue;
        }
        out_str.push_str(&colorise_adv(&format!("{}", count), COLOR_NAMES[color]));
        if !compact {
            out_str.push_str(&" ");
        }
    }
    out_str
}
fn stripped_len(string: &str) -> usize {
    let line_bytes: Vec<u8> = string.bytes().collect();
    let stripped_bytes = strip_ansi_escapes::strip(&line_bytes).expect("");
    stripped_bytes.len()
}
impl Display for PlayerState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();
        for (row_id, row) in self.rows.iter().enumerate() {
            let mut row_str = String::new();
            let wall_row = self.wall_state[row_id];
            for i in 0..5 {
                let colname = if row.count > 0 {
                    COLOR_NAMES[(row.color as usize)]
                } else {
                    'W'
                };
                if i + row_id < 4 {
                    row_str += " ";
                } else if i + (row.count as usize) > 4 {
                    row_str += &colorise(colname);
                } else {
                    row_str += &colorise_adv(&".".to_string(), colname);
                }
            }
            let mut wall_row_str = String::new();
            for (j, c) in wall_row.iter().enumerate() {
                let idx: i32 = ((j as i32) - (row_id as i32) + 5) % 5;
                let colname = COLOR_NAMES[idx as usize];
                if *c {
                    wall_row_str += &colorise(colname);
                } else {
                    wall_row_str += &colorise_adv(&".".to_string(), colname);
                }
            }
            let combined_str = format!("{} {}", row_str, wall_row_str);
            str.push_str(&combined_str);
            str.push_str("\n");
        }
        let mut floor_str = color_arr(self.floor_tiles.iter(), false);
        let score_str = format!("{}", self.score);
        let width = stripped_len(&floor_str) + score_str.len();
        for _ in 0..(11 - width) {
            floor_str.push(' ');
        }
        floor_str.push_str(&score_str);
        str.push_str(&floor_str);
        write!(f, "{}", str)
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct CentralState {
    pub central_state_arr: [[u8; 6]; 8],
}
fn arr_to_str(row: [u8; 6]) -> String {
    let mut output = String::new();
    for i in 0..6 {
        for _ in 0..row[i] {
            output.push_str(&colorise(COLOR_NAMES[i]));
        }
    }
    output
}

impl Display for CentralState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut num_tiles = [0; 1 + NUM_FACTORY_DISPLAYS as usize];
        for (display_number, central_state_row) in self.central_state_arr.iter().enumerate() {
            num_tiles[display_number] = central_state_row.iter().sum::<u8>();
        }
        let mut output = String::new();
        for i in 0..=NUM_FACTORY_DISPLAYS {
            if num_tiles[i as usize] > 0 {
                let dump_str = if i == NUM_FACTORY_DISPLAYS {
                    ": the centre"
                } else {
                    ""
                };
                output.push_str(&format!("{}{}    ", i, dump_str));
            }
        }
        output.push_str("\n");
        for factory_display in 0..=NUM_FACTORY_DISPLAYS {
            if num_tiles[factory_display as usize] > 0 {
                let central_state_row = self.central_state_arr[factory_display as usize];
                output.push_str(&arr_to_str(central_state_row));
                if factory_display < NUM_FACTORY_DISPLAYS {
                    for _ in 0..(5 - num_tiles[factory_display as usize]) {
                        output.push_str(" ");
                    }
                }
            }
        }
        write!(f, "{}", output)
    }
}
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct State {
    pub board_states: [PlayerState; 3],
    pub central_state: CentralState,
    pub player_to_play: u8,
    pub bag: [u8; 5],
    pub lid: [u8; 5],
    pub is_finished: bool,
    pub player_scores: [f32; 3],
}

fn random_choice<R: Rng>(int_weights: &[u8], rng: &mut R) -> usize {
    // Choose an element randomly according to the given weights.
    let sum: u8 = int_weights.iter().sum();
    let p: f64 = rng.gen();
    let mut weight_sum = 0;
    for (i, weight) in int_weights.iter().enumerate() {
        weight_sum += weight;
        if (weight_sum as f64) / (sum as f64) > p {
            return i;
        }
    }
    int_weights.len() - 1
}

fn is_first_move(state: &State) -> bool {
    for board in state.board_states.iter() {
        for row in board.rows.iter() {
            if row.count > 0 {
                return false;
            }
        }
        for i in 0..5 {
            if board.floor_tiles[i] > 0 {
                return false;
            }
        }
    }
    true
}

#[cfg(debug_assertions)]
pub fn check_counts(state: &State) {
    // Checks that no tiles have been gained or lost.
    let mut tiles: [u8; 6] = [0; 6];
    let initial_tiles = [NUM_TILES, NUM_TILES, NUM_TILES, NUM_TILES, NUM_TILES, 1];
    for color in 0..6 {
        // Add from factories and dump.
        for factory in 0..=NUM_FACTORY_DISPLAYS {
            tiles[color] += state.central_state.central_state_arr[factory as usize][color];
        }

        for board in state.board_states.iter() {
            tiles[color] += board.floor_tiles[color];
        }

        if color == 6 {
            continue;
        }
        for board in state.board_states.iter() {
            for row in board.rows.iter() {
                if row.color == (color as u8) {
                    tiles[color] += row.count;
                }
            }
            if color < 5 {
                for row_id in 0..5 {
                    let col_id = (row_id + color) % 5;
                    if board.wall_state[row_id][col_id] {
                        tiles[color] += 1;
                    }
                }
            }
        }
        if color < 5 {
            tiles[color] += state.bag[color];
            tiles[color] += state.lid[color];
        }
        if tiles[color] != initial_tiles[color] {
            panic!(
                "The following state is invalid:\n{}\nExpected {} {} tiles, state has {}.",
                state.to_string(),
                initial_tiles[color],
                COLOR_NAMES[color],
                tiles[color]
            );
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut board_strs: [String; 4] = Default::default();
        for (i, board_state) in self.board_states.iter().enumerate() {
            board_strs[i] = board_state.to_string();
        }
        let to_play_str = if self.is_finished {
            "".to_string()
        } else {
            format!("Player {} to play. ", self.player_to_play)
        };
        board_strs[3] = format!(
            "{}\n{}\nbag {}\nlid {}",
            to_play_str,
            self.central_state.to_string(),
            color_arr(self.bag.iter(), false),
            color_arr(self.lid.iter(), false)
        );
        let mut output_lines = Vec::new();
        for board_str in board_strs.iter() {
            let lines = board_str.split('\n');
            for (line_num, line) in lines.enumerate() {
                if line_num + 1 > output_lines.len() {
                    output_lines.push(String::new());
                }
                let pad_len = 13 - stripped_len(&line) as i32;
                output_lines[line_num] += &line.to_string();
                for _ in 0..pad_len {
                    output_lines[line_num].push(' ');
                }
            }
        }
        let mut output = String::new();
        for line in output_lines {
            output += line.trim_end();
            output += "\n";
        }
        write!(f, "{}", output)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Action {
    pub display_number: u8,
    pub color: u8,
    pub row_id: u8,
}

impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "Move {} tiles from display {} to row {}.",
                colorise(COLOR_NAMES[self.color as usize]),
                self.display_number,
                self.row_id
            )
        )
    }
}

/// Gets the actions that can be played in the given state.
pub fn get_valid_actions(state: &State) -> Vec<Action> {
    let mut valid_actions = Vec::new();
    let board = &state.board_states[state.player_to_play as usize];
    let central_state = &state.central_state;

    let mut dests = [[false; 6]; 5];
    for dest in dests.iter_mut() {
        dest[5] = true;
    }
    for row_id in 0..5 {
        let row = board.rows[row_id];
        if row.count > 0 {
            if row.count < row_id as u8 + 1 {
                // More space to add to the row.
                dests[row.color as usize][row_id] = true;
            }
        } else {
            // Row is empty.
            for (color, dest) in dests.iter_mut().enumerate() {
                let color_on_wall = board.wall_state[row_id][(row_id + color) % 5];
                if !color_on_wall {
                    dest[row_id] = true;
                }
            }
        }
    }
    for display_number in 0..=NUM_FACTORY_DISPLAYS {
        for (color, dest) in dests.iter().enumerate() {
            if central_state.central_state_arr[display_number as usize][color] == 0 {
                continue;
            }
            for (row_id, allowed) in dest.iter().enumerate() {
                if !*allowed {
                    continue;
                }
                valid_actions.push(Action {
                    display_number: display_number as u8,
                    color: color as u8,
                    row_id: row_id as u8,
                })
            }
        }
    }
    valid_actions
}

fn score_tile_placement(wall_state: &[[bool; 5]; 5], row_id: u8, col_id: u8) -> i32 {
    let mut pos = col_id as i8;
    while (pos > 0) && wall_state[row_id as usize][(pos - 1) as usize] {
        pos -= 1;
    }
    let num_left = col_id as i8 - pos;
    assert!(num_left >= 0);

    let mut pos = col_id as i8;
    while (pos < 4) && wall_state[row_id as usize][(pos + 1) as usize] {
        pos += 1;
    }
    let num_right = pos - col_id as i8;
    assert!(num_right >= 0);

    let mut pos = row_id as i8;
    while (pos > 0) && wall_state[(pos - 1) as usize][col_id as usize] {
        pos -= 1;
    }
    let num_above = row_id as i8 - pos;
    assert!(num_above >= 0);

    let mut pos = row_id as i8;
    while (pos < 4) && wall_state[(pos + 1) as usize][col_id as usize] {
        pos += 1;
    }
    let num_below = pos - row_id as i8;
    assert!(num_below >= 0);

    let horizontal_score = num_left + num_right + 1;
    let vertical_score = num_above + num_below + 1;
    if cmp::min(horizontal_score, vertical_score) == 1 {
        cmp::max(horizontal_score, vertical_score) as i32
    } else {
        (horizontal_score + vertical_score) as i32
    }
}

fn score_and_move_floor_tiles(board: &mut PlayerState, lid: &mut [u8; 5]) {
    let initial_sum = board.floor_tiles.iter().sum::<u8>() + lid.iter().sum::<u8>();
    let mut num_so_far = 0;
    let mut penalty = 0;
    for (color, value) in board.floor_tiles.iter_mut().enumerate() {
        for _ in 0..*value {
            if num_so_far <= 1 {
                penalty += 1;
            } else if num_so_far <= 4 {
                penalty += 2;
            } else if num_so_far <= 6 {
                penalty += 3;
            }
            num_so_far += 1;
        }
        if (color != 5) && (*value > 0) {
            lid[color] += *value;
            *value = 0;
        }
    }
    board.score -= penalty;
    board.score = std::cmp::max(board.score, 0);
    let final_sum = board.floor_tiles.iter().sum::<u8>() + lid.iter().sum::<u8>();
    if initial_sum != final_sum {
        panic!("{} {}", initial_sum, final_sum);
    }
}

/// Score and reset.
///
/// Returns bool, indicating whether the game is finished.
fn score_and_reset(state: &mut State) {
    #[cfg(debug_assertions)]
    check_counts(&state);

    // Update the player to play.
    for player_id in 0..3 {
        if state.board_states[player_id as usize].floor_tiles[5] > 0 {
            state.player_to_play = player_id;
        }
    }

    // Empty full rows and score. Subtract points for the floor tiles. Move the floor tiles to the lid.
    for (_player_id, board) in state.board_states.iter_mut().enumerate() {
        for row_id in 0..5 {
            let mut row = &mut board.rows[row_id];
            assert!(row.count <= row_id as u8 + 1);
            if row.count < row_id as u8 + 1 {
                continue;
            }
            let col_id: u8 = (row_id as u8 + row.color) % 5;

            // Move all but one tile from the row to the lid and the other tile to the wall.
            state.lid[row.color as usize] += row.count - 1;
            board.wall_state[row_id as usize][col_id as usize] = true;
            row.count = 0;
            board.score += score_tile_placement(&board.wall_state, row_id as u8, col_id);
        }

        score_and_move_floor_tiles(board, &mut state.lid);
    }
    #[cfg(debug_assertions)]
    check_counts(&state);
}

///  Determine whether the game is finished.
///
///  A game of Amarillo is over when:
///  There are no tiles to take from the centre; and either:
///  * a player has a horizontal
///  * there are no tiles in the bag or lid to refill with
fn is_finished(state: &State) -> bool {
    if !get_valid_actions(&state).is_empty() {
        return false;
    }
    for board in state.board_states.iter() {
        for row in 0..5 {
            let mut is_horizontal = true;
            for col in 0..5 {
                if !board.wall_state[row][col] {
                    is_horizontal = false;
                }
            }
            if is_horizontal {
                return true;
            }
        }
    }
    let bag_sum = state.bag.iter().sum::<u8>();
    let lid_sum = state.lid.iter().sum::<u8>();
    bag_sum + lid_sum == 0
}

/// Score vertical and horizontal rows and sets of colors.
fn score_bonuses(state: &mut State) {
    let mut max_score = -1000;
    for (_player_id, board) in state.board_states.iter_mut().enumerate() {
        for row_id in 0..5 {
            let mut is_full_row = true;
            for col_id in 0..5 {
                if !board.wall_state[row_id][col_id] {
                    is_full_row = false;
                }
            }
            if is_full_row {
                board.score += 2;
            }
        }

        for col_id in 0..5 {
            let mut is_full_col = true;
            for row_id in 0..5 {
                if !board.wall_state[row_id][col_id] {
                    is_full_col = false;
                }
            }
            if is_full_col {
                board.score += 7;
            }
        }
        for color in 0..5 {
            let mut color_score = 10;
            for i in 0..5 {
                if !board.wall_state[i][(i + color) % 5] {
                    color_score = 0;
                }
            }
            board.score += color_score;
            if color_score > 0 {}
        }
        max_score = cmp::max(max_score, board.score);
    }

    for (player_id, board) in state.board_states.iter().enumerate() {
        state.player_scores[player_id] = if board.score == max_score { 1. } else { 0. };
    }
    let sum_scores: f32 = state.player_scores.iter().sum();
    for player_id in 0..3 {
        state.player_scores[player_id] /= sum_scores;
    }
}

fn inital_player_state() -> PlayerState {
    PlayerState {
        wall_state: [[false; 5]; 5],
        rows: [Row { color: 0, count: 0 }; 5],
        score: 0,
        floor_tiles: [0; 6],
    }
}
pub fn get_random_initial_state<R: Rng>(rng: &mut R) -> State {
    let mut board_states = [
        inital_player_state(),
        inital_player_state(),
        inital_player_state(),
    ];
    // Give 0th player the start token.
    board_states[0].floor_tiles[5] = 1;
    let mut state = State {
        board_states,
        central_state: CentralState {
            central_state_arr: [[0; 6]; 8],
        },
        player_to_play: 0,
        bag: [20; 5],
        lid: [0; 5],
        is_finished: false,
        player_scores: [0.; 3],
    };
    fill_factory_displays(&mut state, rng);
    state
}

pub fn fill_factory_displays<R: Rng>(state: &mut State, rng: &mut R) {
    assert!(has_empty_centre(&state));
    #[cfg(debug_assertions)]
    check_counts(&state);
    let mut n: u8 = state.bag.iter().sum();
    for factory_id in 0..NUM_FACTORY_DISPLAYS {
        for _ in 0..4 {
            if n == 0 {
                for i in 0..5 {
                    state.bag[i] = state.lid[i];
                    state.lid[i] = 0;
                    n = state.bag.iter().sum();
                }
            }
            if n != 0 {
                let color = random_choice(&state.bag, rng);
                state.bag[color] -= 1;
                state.central_state.central_state_arr[factory_id as usize][color] += 1;
                n -= 1;
            }
        }
    }
}

fn has_empty_centre(state: &State) -> bool {
    for i in 0..=NUM_FACTORY_DISPLAYS {
        for j in 0..6 {
            if state.central_state.central_state_arr[i as usize][j] > 0 {
                return false;
            }
        }
    }
    true
}

/// Check that the action is valid
pub fn is_valid_action(state: &State, action: Action) -> bool {
    let valid_actions = get_valid_actions(&state);
    for valid_action in valid_actions {
        if action == valid_action {
            return true;
        }
    }
    false
}
/// Plays an action inplace.
/// Doesn't refill the factory displays.
pub fn step(mut state: State, action: Action, do_check_counts: bool) -> (State, bool) {
    debug_assert!(
        is_valid_action(&state, action),
        format!(
            "Tried to play invalid action {} in state:\n{}",
            action.to_string(),
            state.to_string()
        )
    );
    if do_check_counts {
        #[cfg(debug_assertions)]
        check_counts(&state);
    }
    state.player_scores = [0., 0., 0.];
    let display_number = action.display_number;
    let color = action.color;
    let num_tiles = state.central_state.central_state_arr[display_number as usize][color as usize];
    let row_id = action.row_id;
    {
        let central_state_arr = &mut state.central_state.central_state_arr;
        if display_number < NUM_FACTORY_DISPLAYS {
            // Move non-requested tiles from the factory to the dump.
            for color in 0..6 {
                if color as u8 == action.color {
                    continue;
                }
                central_state_arr[NUM_FACTORY_DISPLAYS as usize][color] +=
                    central_state_arr[display_number as usize][color];
                central_state_arr[display_number as usize][color] = 0;
            }
        }
    }

    if is_first_move(&state) {
        let floor_tiles = &mut state.board_states[state.player_to_play as usize].floor_tiles;
        // Check that the player to play has the first player token.
        debug_assert!(
            floor_tiles[5] == 1,
            format!("It's the first move, but the player to play doesn't have the first player token in state:\n{}", state.to_string())
        );
        // Move the first player token to the dump.
        floor_tiles[5] = 0;
        state.central_state.central_state_arr[NUM_FACTORY_DISPLAYS as usize][5] = 1;
    }
    if state.central_state.central_state_arr[display_number as usize][5] > 0 {
        // If the player token is in the dump, give it to the first player.
        let floor_tiles = &mut state.board_states[state.player_to_play as usize].floor_tiles;
        floor_tiles[5] = 1;
        state.central_state.central_state_arr[display_number as usize][5] = 0;
    }

    // Give the player the requested tiles.
    if row_id < 5 {
        let mut row = &mut state.board_states[state.player_to_play as usize].rows[row_id as usize];
        row.count += num_tiles;
        row.color = action.color;
    } else {
        state.board_states[state.player_to_play as usize].floor_tiles[action.color as usize] +=
            num_tiles;
    }
    state.central_state.central_state_arr[display_number as usize][action.color as usize] = 0;
    if do_check_counts {
        #[cfg(debug_assertions)]
        check_counts(&state);
    }

    // Move overflow tiles to the floor.
    if row_id < 5 {
        let board = &mut state.board_states[state.player_to_play as usize];
        let row_max = row_id + 1;
        let overflow_num = board.rows[row_id as usize].count as i8 - row_max as i8;
        if overflow_num > 0 {
            board.rows[row_id as usize].count = row_max;
            board.floor_tiles[color as usize] += overflow_num as u8;
        }
    }

    state.player_to_play = (state.player_to_play + 1) % 3;

    if do_check_counts {
        #[cfg(debug_assertions)]
        check_counts(&state);
    }
    let empty_centre = has_empty_centre(&state);
    if empty_centre {
        score_and_reset(&mut state);
        state.is_finished = is_finished(&state);
        if state.is_finished {
            score_bonuses(&mut state);
        }
    }
    if do_check_counts {
        #[cfg(debug_assertions)]
        check_counts(&state);
    }
    if empty_centre {
        // Check there are no floor tiles.
        for player_state in state.board_states.iter() {
            for (color, count) in player_state.floor_tiles.iter().enumerate() {
                if color == 5 {
                    continue;
                }
                debug_assert!(
                    *count == 0,
                    format!(
                        "Centre is empty but there are floor tiles for state:\n{}",
                        state.to_string()
                    )
                );
            }
        }
    }
    (state, empty_centre)
}
