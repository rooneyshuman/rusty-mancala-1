#![feature(io)]

use std::io;
use std::io::prelude::*;
use std::fmt::Display;

use crate::proto::*;
use crate::game_objects::*;

use std::{thread, time};

// --------------- out of game --------------- //
pub fn initial_screen() -> String {
    let mut stdin = io::stdin();
    let mut host = String::new();
    let mut port = String::new();
    print!("Enter a host: ");
    io::stdout().flush();
    stdin.read_line(&mut host);
    print!("Enter a port: ");
    io::stdout().flush();
    stdin.read_line(&mut port);
    let port_int = port.trim().parse::<u32>().expect("could not make port into an int");
    let trimmed_host = host.trim().to_string();
    trimmed_host + &":".to_string() + &port_int.to_string()
}

pub fn initial_hello_msg() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::InitSetup,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty()
    }
}

pub fn handle_out_of_game(connection: &str, user_nick: &str) -> Msg {
    loop {
        let mut stdin = io::stdin();
        let mut selection = String::new();
        print!("
    Now connected to: {0}.
    Your current nickname is: {1}.
    Welcome to Mancala.  Please select one of the following options:
        (1) Change Nickname
        (2) List Active Games
        (3) List Active Users
        (4) Start New Game
        (5) Join Game
        (6) Disconnect

    Enter your choice: ", connection, user_nick);
        io::stdout().flush();
        stdin.read_line(&mut selection);
        let selection_int = selection.trim().parse::<u8>();
        match selection_int {
            Ok(choice) => {
                match choice {
                    1 => {
                        println!("\n");
                        return set_nickname();
                    }
                    2 => {
                        return list_active_games();
                    }
                    3 => {
                        return list_active_users();
                    }
                    4 => {
                        return start_new_game();
                    }
                    5 => {
                        return join_game();
                    }
                    6 => {
                        return client_disconnect(user_nick);
                    }
                    _ => {
                        println!("invalid selection");
                    }
                }
            }
            Err(e) => {
                error!("Could not read that input! More info: {}", e);
            }
        }
    }
}


// READ functions
fn list_active_games() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::ListGames,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty()
    }
}

fn list_active_users() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::ListUsers,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty()
    }
}

fn join_game() -> Msg {
    let mut stdin = io::stdin();
    let mut game_id = String::new();
    print!("Which Game Id do you want to join: ");
    io::stdout().flush().expect("Client input something nonsensical");
    io::stdin().read_line(&mut game_id).expect("I/O error");
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::JoinGame,
        game_status: GameStatus::NotInGame,
        data: game_id.trim().to_string(),
        game_state: GameState::new_empty()
    }
}


// WRITE functions
fn set_nickname() -> Msg {
    let mut stdin = io::stdin();
    let mut nickname = String::new();
    print!("Enter new nickname: ");
    io::stdout().flush().expect("Client input something nonsensical");
    io::stdin().read_line(&mut nickname).expect("I/O error");
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::SetNick,
        game_status: GameStatus::NotInGame,
        data: nickname.trim().to_string(),
        game_state: GameState::new_empty()
    }
}

pub fn start_new_game() -> Msg {
    let mut stdin = io::stdin();
    let mut game_name = String::new();
    print!("Enter a name of a new game: ");
    io::stdout().flush();
    stdin.read_line(&mut game_name);
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::MakeNewGame,
        game_status: GameStatus::NotInGame,
        data: game_name.trim().to_string(),
        game_state: GameState::new_empty()
    }
}
//
//pub fn join_game() -> Msg {}

fn client_disconnect(user_nick: &str) -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::KillMe,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty()
    }
}


// --------------- in game --------------- //
pub fn handle_in_game(server_msg: &Msg, connection: &str, my_id: u32) -> Msg {
    if server_msg.status != Status::Ok {
        return Msg {
            status: Status::Ok,
            headers: Headers::Read,
            command: Commands::KillMe,
            game_status: GameStatus::NotInGame,
            data: String::new(),
            game_state: GameState::new_empty()
        };
    }
    if server_msg.command == Commands::GameIsOver {
        println!("Game Over!");
        render_board(server_msg);
        return leave_game();
    }
    if !server_msg.game_state.active {
        println!("Waiting for another player...");
        return wait_for_my_turn();
    }
    let am_i_player_one: bool = my_id == server_msg.game_state.player_one;
    println!("Current game state: ");
    render_board(server_msg);
    if (am_i_player_one && server_msg.game_state.player_one_turn) || (!am_i_player_one && !server_msg.game_state.player_one_turn) {
        println!("Waiting for my turn...");
        return wait_for_my_turn();
    } else {
        return make_move();
    }
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::KillMe,
        game_status: GameStatus::NotInGame,
        data: String::new(),
        game_state: GameState::new_empty()
    }
}

// Response to Client
fn get_current_gamestate() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Read,
        command: Commands::GetCurrentGamestate,
        game_status: GameStatus::InGame,
        data: String::new(),
        game_state: GameState::new_empty()
    }
}

fn wait_for_my_turn() -> Msg {
    let two_sec = time::Duration::from_secs(2);
    let now = time::Instant::now();
    thread::sleep(two_sec);
    return get_current_gamestate();
}

fn send_game_state() {}

// Respond to Client's Actions
fn make_move() -> Msg {
    let mut stdin = io::stdin();
    let mut move_to_make = String::new();
    print!("Which slot do you want to move: ");
    io::stdout().flush();
    stdin.read_line(&mut move_to_make);
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::MakeMove,
        game_status: GameStatus::InGame,
        data: move_to_make.trim().to_string(),
        game_state: GameState::new_empty()
    }
}

fn render_board(msg: &Msg) {
    println!("{:?}", msg.game_state.get_board());
    println!("Player One score: {}", msg.game_state.get_player_one_score());
    println!("Player Two score: {}", msg.game_state.get_player_two_score());
}


fn leave_game() -> Msg {
    Msg {
        status: Status::Ok,
        headers: Headers::Write,
        command: Commands::LeaveGame,
        game_status: GameStatus::InGame,
        data: String::new(),
        game_state: GameState::new_empty()
    }
}

fn send_message() {} // TODO - implement?


// --------------- handle server response --------------- //
pub fn handle_server_response(
    server_msg: &Msg,
    connection: &str,
    nickname: &mut String,
    my_id: u32
) -> Msg {
    if !server_msg.data.is_empty() {
        println!("server response: {}", server_msg.data);
    }
    if server_msg.command == Commands::SetNick {
        if server_msg.status == Status::Ok {
            let new_nick: String = server_msg.data.clone();
            *nickname = new_nick.clone();
        }
    }
    return match server_msg.game_status {
        GameStatus::NotInGame => {
            handle_out_of_game(connection, &nickname)
        }
        _ => {
            handle_in_game(server_msg, connection, my_id)
        }
    }
}