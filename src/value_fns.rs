use std::io::Read;

use crate::game_state::*;
pub trait ValueFunction {
    fn get_value(&mut self, state: &State) -> [f32; 3] {
        // Use the final score if the game is finished.
        if state.is_finished {
            return state.player_scores;
        }
        self.get_in_progress_value(&state)
    }
    fn get_in_progress_value(&mut self, state: &State) -> [f32; 3];
}

fn get_session_data(
    filename: &str,
    num_dims: u64,
) -> (
    tensorflow::Session,
    tensorflow::Tensor<f32>,
    tensorflow::Graph,
) {
    let states = tensorflow::Tensor::new(&[1, 3, num_dims]);
    let mut graph = tensorflow::Graph::new();
    let mut proto = Vec::new();
    std::fs::File::open(filename)
        .unwrap()
        .read_to_end(&mut proto)
        .unwrap();
    graph
        .import_graph_def(&proto, &tensorflow::ImportGraphDefOptions::new())
        .unwrap();
    let session = tensorflow::Session::new(&tensorflow::SessionOptions::new(), &graph).unwrap();
    (session, states, graph)
}
fn run_graph(
    session: &mut tensorflow::Session,
    states: &mut tensorflow::Tensor<f32>,
    graph: &mut tensorflow::Graph,
) -> [f32; 3] {
    let mut args = tensorflow::SessionRunArgs::new();
    args.add_feed(
        &graph.operation_by_name_required("states").unwrap(),
        0,
        states,
    );
    let values_idx = args.request_fetch(&graph.operation_by_name_required("values").unwrap(), 0);
    session.run(&mut args).unwrap();
    let values: tensorflow::Tensor<f32> = args.fetch(values_idx).unwrap();
    let mut result = [0.0, 0.0, 0.0];
    for i in 0..3 {
        result[i] = values[i];
    }
    result
}
pub struct ValueFunctionTFV2 {
    session: tensorflow::Session,
    states: tensorflow::Tensor<f32>,
    graph: tensorflow::Graph,
}
impl ValueFunctionTFV2 {
    pub fn new() -> ValueFunctionTFV2 {
        let (session, states, graph) = get_session_data("val_v2.pb", 54);
        ValueFunctionTFV2 {
            states,
            session,
            graph,
        }
    }
    fn get_value_raw(&mut self, state_arr: [[f32; 54]; 3]) -> [f32; 3] {
        let mut pos = 0;
        for player_row in state_arr.iter() {
            for val in player_row.iter() {
                self.states[pos] = *val;
                pos += 1;
            }
        }
        run_graph(&mut self.session, &mut self.states, &mut self.graph)
    }
}

impl ValueFunction for ValueFunctionTFV2 {
    fn get_in_progress_value(&mut self, state: &State) -> [f32; 3] {
        let mut state_arr = [[0.0; 54], [0.0; 54], [0.0; 54]];
        let mut min_score = std::i32::MAX;
        for board_state in state.board_states.iter() {
            if board_state.score < min_score {
                min_score = board_state.score;
            }
        }
        for (player_num, board_state) in state.board_states.iter().enumerate() {
            let mut offset = 0;
            for (row_num, row) in board_state.rows.iter().enumerate() {
                if row_num == 0 {
                    continue;
                }
                if row.count > 0 {
                    state_arr[player_num][offset + row.count as usize - 1] = 1.;
                }
                offset += row_num;
            }
            for (row_num, row) in board_state.rows.iter().enumerate() {
                if row_num == 0 {
                    continue;
                }
                if row.count > 0 && row.color < 4 {
                    state_arr[player_num][offset + row.color as usize] = 1.;
                }
                offset += 4;
            }
            for row_idx in 0..5 {
                for col_idx in 0..5 {
                    if board_state.wall_state[row_idx][col_idx] {
                        state_arr[player_num][offset] = 1.;
                    }
                    offset += 1;
                }
            }
            state_arr[player_num][offset] = (board_state.score - min_score) as f32;

            // max wall count
            let mut counts = [0; 5];
            let mut colour_counts = [0; 5];
            for (row_id, row) in board_state.wall_state.iter().enumerate() {
                for (col_id, val) in row.iter().enumerate() {
                    if *val {
                        counts[col_id] += 1;
                        let colour = (col_id - row_id + 5) % 5;
                        colour_counts[colour] += 1;
                    }
                }
            }
            state_arr[player_num][offset] += *counts.iter().max().unwrap() as f32;
            offset += 1;
            state_arr[player_num][offset] += *colour_counts.iter().max().unwrap() as f32;
            offset += 1;

            assert!(offset == 53);
        }
        self.get_value_raw(state_arr)
    }
}
