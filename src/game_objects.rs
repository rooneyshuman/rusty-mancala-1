use crate::constants::*;
use serde::{Deserialize, Serialize};

/// Object holding all of a game's state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameState {
    pub player_one: u32,
    pub player_two: u32,
    pub game_name: String,
    pub game_id: u32,
    pub game_board: [u8; SLOTS * 2],
    player_one_goal_slot: usize,
    player_two_goal_slot: usize,
    pub player_one_turn: bool,
    pub active: bool,
    pub game_over: bool,
}

impl GameState {
    /// Initialize new gamestate.  One player required.
    /// Other values set to sensible defaults.
    pub fn new(p_one: u32, name: String, id: u32) -> GameState {
        let mut init_game_board = [STARTING_STONES; SLOTS * 2];
        init_game_board[SLOTS] = 0;
        init_game_board[0] = 0;
        GameState {
            player_one: p_one,
            player_two: 0,
            game_name: name,
            game_id: id,
            game_board: init_game_board,
            player_one_goal_slot: SLOTS,
            player_two_goal_slot: 0,
            player_one_turn: true,
            active: false,
            game_over: false,
        }
    }

    /// Creates an empty game as a workaround for rusts' insistence on
    /// not allowing uninitialized variables, and needing the game state
    /// to be sent with all messages.
    pub fn new_empty() -> GameState {
        GameState {
            player_one: 0,
            player_two: 0,
            game_name: "empty".to_string(),
            game_id: 0,
            game_board: [0; SLOTS * 2],
            player_one_goal_slot: 0,
            player_two_goal_slot: 0,
            player_one_turn: false,
            active: false,
            game_over: false,
        }
    }

    pub fn add_new_player(&mut self, player_id: u32) {
        if self.player_one == 0 {
            self.player_one = player_id;
        } else if self.player_two == 0 {
            self.player_two = player_id;
        }
        if self.player_one != 0 && self.player_two != 0 {
            self.active = true;
        }
    }

    pub fn remove_player(&mut self, player_id: u32) {
        if self.player_one == player_id {
            self.player_one = 0;
        } else if self.player_two == player_id {
            self.player_two = 0;
        }
        if self.player_one == 0 || self.player_two == 0 {
            let mut init_game_board = [STARTING_STONES; SLOTS * 2];
            init_game_board[SLOTS] = 0;
            init_game_board[0] = 0;
            self.game_board = init_game_board;
            self.active = false;
        }
        info!("Removed player {}, game is now: {:?}", player_id, self);
    }

    //noinspection ALL - don't have IDE warnings
    /// When game is over, add the remaining stones on each players
    /// side to their goal slot.
    fn collect_remaining_stones(&mut self) {
        self.game_board[self.player_one_goal_slot] +=
            &self.game_board[1..self.player_one_goal_slot].iter().sum();
        self.game_board[self.player_two_goal_slot] += &self.game_board
            [self.player_one_goal_slot + 1..]
            .iter()
            .sum();
    }

    fn is_game_over(&mut self) -> bool {
        if self.player_one_turn {
            return self.game_board[1..self.player_one_goal_slot]
                .iter()
                .all(|&x| x == 0);
        }
        self.game_board[self.player_one_goal_slot + 1..]
            .iter()
            .all(|&x| x == 0)
    }

    fn get_players_goal_slots(&mut self) -> (usize, usize) {
        if self.player_one_turn {
            (self.player_one_goal_slot, self.player_two_goal_slot)
        } else {
            (self.player_two_goal_slot, self.player_one_goal_slot)
        }
    }

    fn add_capture_points(&mut self, points_to_add: u8) {
        if self.player_one_turn {
            self.game_board[self.player_one_goal_slot] += points_to_add;
        } else {
            self.game_board[self.player_two_goal_slot] += points_to_add;
        }
    }

    /// "capturing" function
    /// if your last manacala piece ends up on your side, in an empty slot,
    /// you get to capture your opponents' pieces in the opposite slot and
    /// add them to your goal
    fn capture(&mut self, cur_slot: usize) -> bool {
        if self.game_board[cur_slot] != 1 {
            return false;
        }
        let mut opposite_slot: usize = 0;
        if self.player_one_turn && cur_slot < SLOTS {
            opposite_slot = SLOTS + (SLOTS - cur_slot);
        } else if !self.player_one_turn && cur_slot > SLOTS {
            opposite_slot = SLOTS - (cur_slot - SLOTS);
        }
        if self.game_board[opposite_slot] == 0 || opposite_slot == 0 {
            return false;
        }
        self.add_capture_points(self.game_board[opposite_slot] + 1);
        self.game_board[cur_slot] = 0;
        self.game_board[opposite_slot] = 0;
        true
    }

    pub fn make_move(&mut self, slot_to_move: usize) {
        if self.game_over {
            return;
        }
        let mut num_of_stones: u8 = self.game_board[slot_to_move];
        let goal_slots: (usize, usize) = self.get_players_goal_slots();
        self.game_board[slot_to_move] = 0;
        let mut cur_slot: usize = (slot_to_move + 1) % BOARD_LENGTH;
        loop {
            if cur_slot == goal_slots.1 {
                // skip opponents goal
                cur_slot = (cur_slot + 1) % BOARD_LENGTH;
                continue;
            }
            self.game_board[cur_slot] += 1;
            num_of_stones -= 1;
            if num_of_stones == 0 {
                break;
            }
            cur_slot = (cur_slot + 1) % BOARD_LENGTH;
        }
        // only change turns if current player didn't score
        if cur_slot != goal_slots.0 && !self.capture(cur_slot) {
            self.player_one_turn = !self.player_one_turn;
        }
        if self.is_game_over() {
            self.game_over = true;
            self.collect_remaining_stones();
            self.active = false;
        }
    }

    pub fn get_board(&self) -> [u8; SLOTS * 2] {
        self.game_board
    }

    pub fn set_game_over(&mut self) {
        self.game_over = true;
    }

    pub fn get_player_one_score(&self) -> u8 {
        self.game_board[self.player_one_goal_slot]
    }

    pub fn get_player_two_score(&self) -> u8 {
        self.game_board[self.player_two_goal_slot]
    }
}

#[test]
fn test_game_state_can_be_initialized() {
    let gs: GameState = GameState::new(1, "name".to_string(), 0);
    assert!(!gs.active);
}

#[test]
fn test_game_state_init_values_are_correct() {
    let gs: GameState = GameState::new(1, "name".to_string(), 0);
    let mut init_game_board = [STARTING_STONES; SLOTS * 2];
    init_game_board[SLOTS] = 0;
    init_game_board[0] = 0;
    assert_eq!(gs.player_one, 1);
    assert_eq!(gs.player_two, 0);
    assert_eq!(gs.game_name, "name");
    assert_eq!(gs.game_id, 0);
    assert_eq!(gs.game_board, init_game_board);
    assert!(gs.player_one_turn);
    assert!(!gs.active);
}

#[test]
fn test_game_becomes_active_after_adding_both_players() {
    let mut gs: GameState = GameState::new_empty();
    gs.add_new_player(1);
    assert!(!gs.active);
    gs.add_new_player(2);
    assert!(gs.active);
}

#[test]
fn test_game_properly_removes_player_one() {
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    assert_eq!(gs.player_one, 1);
    gs.remove_player(1);
    assert_eq!(gs.player_one, 0);
}

#[test]
fn test_game_properly_removes_player_two() {
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    gs.add_new_player(2);
    assert_eq!(gs.player_two, 2);
    gs.remove_player(2);
    assert_eq!(gs.player_two, 0);
}

#[test]
fn test_game_becomes_active_after_adding_player_two() {
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    assert!(!gs.active);
    gs.add_new_player(1);
    assert!(gs.active);
}

#[test]
fn test_game_state_updates_after_one_move() {
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    gs.add_new_player(1);
    let mut init_game_board = [STARTING_STONES; SLOTS * 2];
    init_game_board[SLOTS] = 0;
    init_game_board[0] = 0;
    gs.make_move(1);
    assert_ne!(gs.game_board, init_game_board);
}

#[test]
fn test_turn_changes_after_making_move() {
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    gs.add_new_player(1);
    let turn1: bool = gs.player_one_turn;
    gs.make_move(1);
    let turn2: bool = gs.player_one_turn;
    gs.make_move(2);
    let turn3: bool = gs.player_one_turn;
    gs.make_move(3);
    let turn4: bool = gs.player_one_turn;
    assert_eq!(turn1, turn3);
    assert_eq!(turn2, turn4);
    assert_ne!(turn1, turn2);
    assert_ne!(turn3, turn4);
}

#[test]
fn test_scoring_turns_dont_change_players() {
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    gs.add_new_player(1);
    let turn1: bool = gs.player_one_turn;
    gs.make_move(3);
    let turn2: bool = gs.player_one_turn;
    assert_eq!(turn1, turn2);
}

#[test]
fn test_player_one_captures() {
    // this test assumes SLOTS = 7 and starting_stones = 4
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    gs.add_new_player(1);
    gs.make_move(6);
    gs.make_move(11);
    let turn1: bool = gs.player_one_turn;
    gs.make_move(2);
    let turn2: bool = gs.player_one_turn;
    assert_eq!(turn1, turn2);
    assert_eq!(gs.game_board[gs.player_one_goal_slot], 7);
    assert_eq!(gs.game_board[gs.player_two_goal_slot], 1);
    assert_eq!(gs.game_board[8], 0);
    assert_eq!(gs.game_board[6], 0);
}

#[test]
fn test_player_two_captures() {
    // this test assumes SLOTS = 7 and starting_stones = 4
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    gs.add_new_player(2);
    gs.make_move(4);
    gs.make_move(13);
    gs.make_move(2);
    gs.make_move(4);
    gs.make_move(9);
    assert!(!gs.player_one_turn);
    assert_eq!(gs.game_board[gs.player_one_goal_slot], 2);
    assert_eq!(gs.game_board[gs.player_two_goal_slot], 7);
    assert_eq!(gs.game_board[1], 0);
    assert_eq!(gs.game_board[13], 0);
}

#[test]
fn test_collect_remaining() {
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    gs.add_new_player(1);
    let mut gs2: GameState = GameState::new(1, "name".to_string(), 0);
    gs.add_new_player(1);
    gs.collect_remaining_stones();
    gs2.make_move(6);
    gs2.collect_remaining_stones();
    assert_eq!(gs.game_board[gs.player_one_goal_slot], 24);
    assert_eq!(gs.game_board[gs.player_two_goal_slot], 24);
    assert_eq!(gs2.game_board[gs2.player_one_goal_slot], 21);
    assert_eq!(gs2.game_board[gs2.player_two_goal_slot], 27);
}

#[test]
fn test_game_over() {
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    gs.game_board = [24, 0, 0, 0, 0, 0, 0, 20, 0, 0, 4, 0, 0, 0];
    assert!(!gs.game_over);
    gs.make_move(10);
    assert!(gs.game_over);
}

#[test]
fn test_set_game_over() {
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    assert!(!gs.game_over);
    gs.set_game_over();
    assert!(gs.game_over);
}

#[test]
fn test_get_board() {
    let gs: GameState = GameState::new(1, "name".to_string(), 0);
    let gs_board = gs.get_board();
    let init_board = [0, 4, 4, 4, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4];
    assert_eq!(gs_board, init_board);
}

#[test]
fn test_get_scores() {
    let mut gs: GameState = GameState::new(1, "name".to_string(), 0);
    gs.game_board = [24, 0, 0, 0, 0, 0, 0, 20, 0, 0, 4, 0, 0, 0];
    let player_one_score = gs.get_player_one_score();
    let player_two_score = gs.get_player_two_score();
    assert_eq!(player_one_score, 20);
    assert_eq!(player_two_score, 24);
}
