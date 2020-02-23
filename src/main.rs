use rand::SeedableRng;
use std::io::Write;

mod game_state;
mod mcts;
mod value_fns;
use game_state::*;
use mcts::*;
use value_fns::*;

fn read_char() -> Option<char> {
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).expect("Bad input.");
    s.chars().next()
}

fn options_to_str<T: ToString>(options: &std::collections::BTreeSet<T>) -> String {
    let mut output = String::new();
    // let output_vec = Vec::new();
    for (idx, option) in options.iter().enumerate() {
        // output_vec.push(&option.to_string());
        output.push_str(&option.to_string());
        if idx as isize == options.len() as isize - 2 {
            output.push_str(" or ");
        } else if idx < options.len() - 1 {
            output.push_str(", ");
        }
    }
    output
}

fn prompt<T: ToString>(thing: &str, options: &std::collections::BTreeSet<T>) -> Option<char> {
    if options.len() == 1 {
        let option = options.iter().next()?.to_string();
        println!("You are forced to choose {} {}.", thing, option);
        return option.chars().next();
    }
    print!("Select {} ({}): ", thing, options_to_str(&options));
    std::io::stdout().flush().unwrap();
    read_char()
}

fn input_move_opt(state: &State) -> Option<Action> {
    let mut valid_actions = get_valid_actions(&state);

    let mut display_numbers = std::collections::BTreeSet::new();
    for action in &valid_actions {
        display_numbers.insert(action.display_number);
    }
    let display_number = prompt("display", &display_numbers).and_then(|x| x.to_digit(10))? as u8;
    display_numbers.get(&display_number)?;
    valid_actions.retain(|action| action.display_number == display_number);

    let mut colors = std::collections::BTreeSet::new();
    for action in &valid_actions {
        colors.insert(action.color);
    }
    let mut color_chars = std::collections::BTreeSet::new();
    for color in &colors {
        color_chars.insert(COLOR_NAMES[*color as usize].to_ascii_lowercase());
    }
    let color_char = prompt("color", &color_chars)?;
    let mut color_opt = None;
    for (color_int, color_char_val) in COLOR_NAMES.iter().enumerate() {
        if color_char_val.to_ascii_lowercase() == color_char {
            color_opt = Some(color_int);
        }
    }
    let color_char = color_opt?;
    let color = color_char as u8;
    colors.get(&color)?;
    valid_actions.retain(|action| action.color == color);

    let mut row_ids = std::collections::BTreeSet::new();
    for action in &valid_actions {
        row_ids.insert(action.row_id);
    }
    let row_id = prompt("destination", &row_ids).and_then(|x| x.to_digit(10))? as u8;
    row_ids.get(&row_id)?;

    Some(Action {
        display_number,
        color,
        row_id,
    })
}

fn input_move(state: &State) -> Action {
    loop {
        if let Some(action) = input_move_opt(&state) {
            return action;
        }
    }
}

fn main() {
    let mut rng = rand::rngs::SmallRng::from_entropy();
    let mut vf = ValueFunctionTFV2::new();
    let time_limit = 0.4;
    let mut state = get_random_initial_state(&mut rng);
    loop {
        println!("{:}", state.to_string());
        if state.is_finished {
            break;
        }
        let action = if state.player_to_play == 0 {
            input_move(&state)
        } else {
            make_move(
                &state,
                std::time::Duration::from_nanos((time_limit * 1e9) as u64),
                &mut vf,
            )
        };
        println!("{:}", action.to_string());
        let (new_state, empty_centre) = step(state, action, true);
        state = new_state;
        if empty_centre && !state.is_finished {
            fill_factory_displays(&mut state, &mut rng);
        }
    }
    let mut scores = [0; 3];
    for (player_num, board) in state.board_states.iter().enumerate() {
        scores[player_num] = board.score;
    }
}
