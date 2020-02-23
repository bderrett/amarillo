use crate::game_state::*;
use crate::value_fns::*;
use rand::Rng;
use std::{thread, time};

/// Game tree starting from a particular action.
#[derive(Default, Debug)]
struct ActionTree {
    /// The number of playouts in the tree of this action (or the maximum int value if the end of the game tree is reached).
    num_plays: i32,
    /// Proportion of games won after playing this action.
    score: f32,
    /// Until a playout has been run from the action, this is None.
    post_state: Option<StateTree>,
}

/// Game tree starting from a particular state.
#[derive(Debug)]
struct StateTree {
    /// The state.
    state: State,
    /// The number of playouts that have been run from this state.
    num_plays: i32,
    /// Actions available.
    actions: std::collections::HashMap<Action, ActionTree>,
}

fn choose_random_action(state: &State) -> Option<Action> {
    let valid_actions = get_valid_actions(&state);
    if valid_actions.is_empty() {
        return None;
    }
    let mut rng = rand::thread_rng();
    let rand_idx = rng.gen_range(0, valid_actions.len());
    Some(valid_actions[rand_idx])
}

/// Gives each player's probability of winning from the current state.
/// The state is only provided at the end of each round,
/// when the centre is empty, floor rows are empty, and full rows
/// have been emptied.

fn highest_score_action(state_tree: &StateTree) -> Action {
    let actions = &state_tree.actions;
    debug_assert!(
        !actions.is_empty(),
        format!(
            "No actions on StateTree for State:\n{}",
            state_tree.state.to_string()
        )
    );
    let mut best_action = Action {
        display_number: 0,
        color: 0,
        row_id: 0,
    };
    let mut best_action_score = -std::f32::MAX;
    for (action, action_tree) in actions {
        if action_tree.score > best_action_score {
            best_action = *action;
            best_action_score = action_tree.score;
        }
    }
    best_action
}

/// Chooses the action with the maximum MCTS value for further exploration.
///
/// # Returns
///
/// The chosen action, if there is an action for which the game tree hasn't been fully
/// explored, else None.
fn choose_mcts_action(state_tree: &StateTree) -> Option<Action> {
    let log_n = ((1 + state_tree.num_plays) as f32).ln();

    let (best_action, _) = state_tree
        .actions
        .iter()
        .map(|(action, action_tree)| {
            (
                action,
                action_tree.score + 1.41 * (log_n / ((1 + action_tree.num_plays) as f32)).sqrt(),
            )
        })
        .max_by(|(_, value_a), (_, value_b)| value_a.partial_cmp(value_b).unwrap())?;
    Some(*best_action)
}

fn create_state_tree(state: State) -> StateTree {
    let valid_actions = get_valid_actions(&state);
    let action_trees = valid_actions
        .into_iter()
        .map(|action| (action, Default::default()))
        .collect();

    StateTree {
        state,
        num_plays: 0,
        actions: action_trees,
    }
}

fn mcts_backprop(mut tree: &mut StateTree, actions: &[Action], scores: [f32; 3]) {
    for action in actions {
        tree.num_plays += 1;
        let player_to_play = tree.state.player_to_play;
        let mut action_tree = tree.actions.get_mut(&action).unwrap();
        action_tree.score = (action_tree.score * (action_tree.num_plays as f32)
            + scores[player_to_play as usize])
            / ((action_tree.num_plays + 1) as f32);
        action_tree.num_plays += 1;
        tree = action_tree.post_state.as_mut().unwrap();
    }
}

/// Select, expand and simulate.
fn mcts_ses<T: ValueFunction>(
    mut stree: &mut StateTree,
    vf: &mut T,
) -> Option<(Vec<Action>, [f32; 3])> {
    let mut actions = Vec::new();

    loop {
        match choose_mcts_action(&stree) {
            Some(action) => {
                actions.push(action);
                let action_tree: &mut ActionTree = stree.actions.get_mut(&action).unwrap();
                if action_tree.post_state.is_some() {
                    stree = (&mut action_tree.post_state).as_mut().unwrap();
                } else {
                    let sstate = stree.state.clone();
                    let (next_state, _empty_centre) = step(sstate, action, true);
                    action_tree.post_state = Some(create_state_tree(next_state));
                    let post_state: &mut Option<StateTree> = &mut action_tree.post_state;
                    let mut current_state = post_state.as_mut().unwrap().state.clone();

                    // Run the playout.
                    while let Some(action) = choose_random_action(&current_state) {
                        let result = step(current_state, action, true);
                        current_state = result.0;
                        let empty_centre = result.1;
                        if empty_centre {
                            break;
                        }
                    }
                    let scores = vf.get_value(&current_state);
                    let rval = Some((actions, scores));
                    return rval;
                }
            }
            None => {
                return Some((actions, vf.get_value(&stree.state)));
            }
        }
    }
}

/// Returns whether the tree is complete.
fn update_tree<T: ValueFunction>(mut state_tree: &mut StateTree, vf: &mut T) -> bool {
    match mcts_ses(&mut state_tree, vf) {
        Some((actions, scores)) => {
            mcts_backprop(state_tree, &actions, scores);
            assert!(state_tree.num_plays > 0);
            false
        }
        None => {
            // The tree has already been fully updated.
            true
        }
    }
}
/// 1) Run playouts until:
///    a) the time limit expires; or
///    b) the full game tree has been explored.
/// 2) Return the action with the highest score.
pub fn make_move<T: ValueFunction>(
    state: &State,
    _time_limit: std::time::Duration,
    vf: &mut T,
) -> Action {
    let start = std::time::SystemTime::now();
    let mut state_tree = create_state_tree(state.clone());
    let mut is_complete = false;
    while std::time::SystemTime::now() < start + _time_limit {
        if update_tree(&mut state_tree, vf) {
            is_complete = true;
            break;
        }
    }
    let dur = time::Duration::from_millis(100);
    thread::sleep(dur);
    println!(
        "{} playouts{}.",
        state_tree.num_plays,
        if is_complete {
            " (tree fully explored)"
        } else {
            ""
        }
    );
    highest_score_action(&state_tree)
}
