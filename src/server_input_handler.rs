use crate::proto::*;
use crate::game_objects::*;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};

// --------------- out of game --------------- //
pub fn handle_out_of_game(
    cmd: Commands,
    game_list_mutex: &Arc<Mutex<Vec<GameState>>>,
    id_game_map_mutex: &Arc<Mutex<HashMap<u32, u32>>>,
    active_nicks_mutex: &Arc<Mutex<HashSet<String>>>,
    id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>,
    client_msg: &Msg,
    client_id: u32
) -> Msg {
    if cmd == Commands::InitSetup {
        return initial_setup(id_nick_map_mutex, client_id);
    }
    else if cmd == Commands::ListGames {
        return list_active_games(game_list_mutex);
    }
    else if cmd == Commands::ListUsers {
        return list_active_users(active_nicks_mutex);
    }
    else if cmd == Commands::SetNick {
        return set_nickname(active_nicks_mutex, id_nick_map_mutex, client_msg, client_id);
    }
    else if cmd == Commands::KillMe {
        return client_disconnect(active_nicks_mutex, id_nick_map_mutex, client_id);
    }
    else if cmd == Commands::MakeNewGame {
        return start_new_game(game_list_mutex, id_game_map_mutex, client_id, client_msg.data.clone());
    }
    else if cmd == Commands::JoinGame {
        return join_game(game_list_mutex, id_game_map_mutex, client_id, client_msg);
    }
    Msg {
        status: Status::NotOk,
        headers: Headers::Response,
        command: Commands::Reply,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty()
    }
}


// READ functions
pub fn initial_setup(
    id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>,
    client_id: u32) -> Msg {
    let mut id_nick_map_unlocked = id_nick_map_mutex.lock().unwrap();
    let nickname = id_nick_map_unlocked.get(&client_id).unwrap();
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::Reply,
        game_status: GameStatus::NotInGame,
        data: format!("{}^{}", nickname, client_id),
        game_state: GameState::new_empty()
    }
}

pub fn list_active_games(game_list_mutex: &Arc<Mutex<Vec<GameState>>>) -> Msg {
    let game_list_unlocked = game_list_mutex.lock().unwrap();
    let game_list_string: String = game_list_unlocked
        .iter()
        .fold("Available Games: \n".to_string(), |acc, x| acc + &x.game_id.to_string() + ": " + &x.game_name + "\n");
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::Reply,
        game_status: GameStatus::NotInGame,
        data: game_list_string,
        game_state: GameState::new_empty()
    }
}

pub fn list_active_users(active_nicks_mutex: &Arc<Mutex<HashSet<String>>>) -> Msg {
    let active_nicks_unlocked = active_nicks_mutex.lock().unwrap();
    let active_nicks_string: String = active_nicks_unlocked
        .iter()
        .fold("Active Users: \n".to_string(), |acc, x| acc + x + "\n");
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::Reply,
        game_status: GameStatus::NotInGame,
        data: active_nicks_string,
        game_state: GameState::new_empty()
    }
}

// pub fn get_game_info() -> Msg {}


// WRITE functions
pub fn set_nickname(
    active_nicks_mutex: &Arc<Mutex<HashSet<String>>>,
    id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>,
    client_msg: &Msg,
    client_id: u32) -> Msg {
    let nickname = client_msg.data.clone();
    let mut active_nicks_unlocked = active_nicks_mutex.lock().unwrap();
    if active_nicks_unlocked.contains(&nickname) {
        return Msg {
            status: Status::NotOk,
            headers: Headers::Response,
            command: Commands::SetNick,
            game_status: GameStatus::NotInGame,
            data: "nickname already in use".to_string(),
            game_state: GameState::new_empty()
        };
    } else {
        let mut id_nick_map_unlocked = id_nick_map_mutex.lock().unwrap();
        let old_nick = id_nick_map_unlocked.remove(&client_id).unwrap();
        active_nicks_unlocked.remove(&old_nick);
        id_nick_map_unlocked.insert(client_id, nickname.clone());
        active_nicks_unlocked.insert(nickname.clone());
        Msg {
            status: Status::Ok,
            headers: Headers::Response,
            command: Commands::SetNick,
            game_status: GameStatus::NotInGame,
            data: format!("{}", nickname.clone()),
            game_state: GameState::new_empty()
        }
    }
}

fn start_new_game(
    game_list_mutex: &Arc<Mutex<Vec<GameState>>>,
    id_game_map_mutex: &Arc<Mutex<HashMap<u32, u32>>>,
    client_id: u32, mut
    game_name: String
) -> Msg {
    let mut game_list_unlocked = game_list_mutex.lock().unwrap();
    let mut id_game_map_unlocked = id_game_map_mutex.lock().unwrap();
    let game_id = game_list_unlocked.len() as u32;
    if game_name.is_empty() {
        game_name = "New Game".to_string();
    }
    let mut new_game = GameState::new(
        client_id,
        game_name.clone(),
        game_id
    );
    game_list_unlocked.push(new_game.clone());
    id_game_map_unlocked.insert(client_id, game_id);
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::MakeNewGame,
        game_status: GameStatus::InGame,
        data: "New Game".to_string(),
        game_state: new_game
    }
}

fn join_game(
    game_list_mutex: &Arc<Mutex<Vec<GameState>>>,
    id_game_map_mutex: &Arc<Mutex<HashMap<u32, u32>>>,
    client_id: u32,
    client_msg: &Msg
) -> Msg {
    let mut game_list_unlocked = game_list_mutex.lock().unwrap();
    let mut id_game_map_unlocked = id_game_map_mutex.lock().unwrap();
    let game_id: usize = client_msg.data.parse().unwrap();
    let game: &mut GameState = &mut game_list_unlocked[game_id];
    game.add_player_two(client_id);
    id_game_map_unlocked.insert(client_id, game.game_id);
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::JoinGame,
        game_status: GameStatus::InGame,
        data: format!("Joined Game {}", &game.game_name),
        game_state: game.clone()
    }
}

fn client_disconnect(
    active_nicks_mutex: &Arc<Mutex<HashSet<String>>>,
    id_nick_map_mutex: &Arc<Mutex<HashMap<u32, String>>>,
    client_id: u32
) -> Msg {
    let mut active_nicks_unlocked = active_nicks_mutex.lock().unwrap();
    let mut id_nick_map_unlocked = id_nick_map_mutex.lock().unwrap();
    let nickname = id_nick_map_unlocked.remove(&client_id).unwrap();
    active_nicks_unlocked.remove(&nickname);
    Msg {
        status: Status::Ok,
        headers: Headers::Response,
        command: Commands::KillClient,
        game_status: GameStatus::NotInGame,
        data: format!("Nick {} successfully booted", nickname),
        game_state: GameState::new_empty()
    }
}


// --------------- in game --------------- //
pub fn handle_in_game(
    cmd: Commands,
    game_list_mutex: &Arc<Mutex<Vec<GameState>>>,
    id_game_map_mutex: &Arc<Mutex<HashMap<u32, u32>>>,
    client_msg: &Msg,
    client_id: u32
) -> Msg {
    let mut id_game_map_unlocked = id_game_map_mutex.lock().unwrap();
    let mut game_list_unlocked = game_list_mutex.lock().unwrap();
    let game_id = id_game_map_unlocked.get(&client_id).unwrap();
    let game: &mut GameState = &mut game_list_unlocked[*game_id as usize];
    if game.player_one != 0 && game.player_two != 0 && !game.active {
        return Msg {
            status: Status::Ok,
            headers: Headers::Write,
            command: Commands::GameIsOver,
            game_status: GameStatus::NotInGame,
            data: "Game Over".to_string(),
            game_state: game.clone()
        }
    }
    if cmd == Commands::GetCurrentGamestate {
        return current_state(game);
    }
    else if cmd == Commands::MakeMove {
        return make_move(client_msg, game);
    }
    Msg {
        status: Status::NotOk,
        headers: Headers::Read,
        command: Commands::Reply,
        game_status: GameStatus::NotInGame,
        data: "Somethings wrong".to_string(),
        game_state: game.clone()
    }
}

// Response to Client
fn current_state(game: &GameState) -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::Reply,
        game_status: GameStatus::InGame,
        data: "Current Game State".to_string(),
        game_state: game.clone()
    }
}

// Respond to Client's Actions
pub fn make_move(client_msg: &Msg, game: &mut GameState) -> Msg {
    let move_to_make: u32 = client_msg.data.parse().unwrap();
    game.make_move(move_to_make as usize);
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::Reply,
        game_status: GameStatus::InGame,
        data: "Current Game State".to_string(),
        game_state: game.clone()
    }
}

pub fn leave_game() {} // TODO - implement?

pub fn send_message() {} // TODO - implement?