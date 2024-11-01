#![no_std]

use gstd::{debug, exec, msg, prelude::*};
use pebbles_game_io::*;

static mut GAME_STATE: Option<GameState> = None;

fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

fn optimal_move(pebbles_remaining: u32, max_pebbles_per_turn: u32) -> u32 {
    let mut best_move = 1;
    for i in 1..=max_pebbles_per_turn {
        if (pebbles_remaining - i) % (max_pebbles_per_turn + 1) == 0 {
            best_move = i;
            break;
        }
    }
    best_move
}

fn get_first_player() -> Player {
    let salt = [0u8; 32];
    let (hash, _num) = exec::random(salt).expect("get_first_player(): random call failed");
    let result = hash[0] % 2;
    match result {
        0 => Player::User,
        1 => Player::Program,
        _ => unreachable!(),
    }
}
fn check_winner(state: &GameState) -> Option<Player> {
    if state.pebbles_remaining == 0 {
        Some(match state.first_player {
            Player::User => Player::Program,
            Player::Program => Player::User,
        })
    } else {
        None
    }
}

#[no_mangle]
extern "C" fn init() {
    let pebbles_init: PebblesInit = msg::load().expect("Unable to decode PebbleInit");
    debug!("Received data: {:?}", pebbles_init);
    let first_player = get_first_player();
    let mut game_state = GameState {
        pebbles_count: pebbles_init.pebbles_count,
        max_pebbles_per_turn: pebbles_init.max_pebbles_per_turn,
        pebbles_remaining: pebbles_init.pebbles_count,
        difficulty: pebbles_init.difficulty,
        first_player: first_player.clone(),
        winner: None,
    };
    if first_player == Player::Program {
        let pebbles_to_take = match game_state.difficulty {
            DifficultyLevel::Easy => get_random_u32() % game_state.max_pebbles_per_turn + 1,
            DifficultyLevel::Hard => optimal_move(
                game_state.pebbles_remaining,
                game_state.max_pebbles_per_turn,
            ),
        };
        game_state.pebbles_remaining -= pebbles_to_take;
        game_state.winner = check_winner(&game_state);
    }
    unsafe { GAME_STATE = Some(game_state) };
}

#[no_mangle]
extern "C" fn handle() {
    let pebbles_action: PebblesAction = msg::load().expect("Unable to decode PebbleAction");
    debug!("Received data: {:?}", pebbles_action);
    unsafe {
        let mut game_state = GAME_STATE.take().expect("GameState isn't initialized");

        // The game is over, no further actions can be processed
        if game_state.winner.is_some() {
            GAME_STATE = Some(game_state);
            return;
        }

        match pebbles_action {
            PebblesAction::Turn(pebbles_to_take) => {
                if pebbles_to_take > game_state.max_pebbles_per_turn {
                    msg::reply("You can't take more than the maximum pebbles per turn", 1)
                        .expect("Failed to send reply");
                } else if pebbles_to_take > game_state.pebbles_remaining {
                    msg::reply("You can't take more than the remaining pebbles", 1)
                        .expect("Failed to send reply");
                } else {
                    game_state.pebbles_remaining -= pebbles_to_take;
                    if game_state.pebbles_remaining == 0 {
                        game_state.winner = Some(Player::User);
                        msg::reply(PebblesEvent::Won(Player::User), 0)
                            .expect("Failed to send reply");
                    } else {
                        let pebbles_to_take = match game_state.difficulty {
                            DifficultyLevel::Easy => {
                                get_random_u32() % game_state.max_pebbles_per_turn + 1
                            }
                            DifficultyLevel::Hard => optimal_move(
                                game_state.pebbles_remaining,
                                game_state.max_pebbles_per_turn,
                            ),
                        };
                        game_state.pebbles_remaining -= pebbles_to_take;
                        if game_state.pebbles_remaining == 0 {
                            game_state.winner = Some(Player::Program);
                            msg::reply(PebblesEvent::Won(Player::Program), 0)
                                .expect("Failed to send reply");
                        } else {
                            msg::reply(PebblesEvent::CounterTurn(pebbles_to_take), 0)
                                .expect("Failed to send reply");
                        }
                    }
                }
            }
            PebblesAction::GiveUp => {
                game_state.winner = Some(Player::Program);
                msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Failed to send reply");
            }
            PebblesAction::Restart {
                difficulty,
                pebbles_count,
                max_pebbles_per_turn,
            } => {
                let first_player = get_first_player();
                if first_player == Player::Program {
                    let pebbles_to_take = match difficulty {
                        DifficultyLevel::Easy => get_random_u32() % max_pebbles_per_turn + 1,
                        DifficultyLevel::Hard => optimal_move(pebbles_count, max_pebbles_per_turn),
                    };
                    msg::reply(pebbles_to_take, 0).expect("Failed to send reply");
                }
                game_state = GameState {
                    pebbles_count,
                    max_pebbles_per_turn,
                    pebbles_remaining: pebbles_count,
                    difficulty,
                    first_player,
                    winner: None,
                }
            }
        }

        GAME_STATE = Some(game_state);
    }
}

#[no_mangle]
extern "C" fn state() {
    unsafe {
        if let Some(ref game_state) = GAME_STATE {
            msg::reply(game_state.clone(), 0).expect("Failed to send reply");
        }
    }
}
