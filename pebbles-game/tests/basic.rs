#[cfg(test)]
mod tests {
    use super::*;
    use gstd::prelude::*;
    use gtest::{Program, System};
    use pebbles_game_io::*;

    fn create_system_and_user() -> (System, u64) {
        let sys = System::new();
        sys.init_logger();
        let user_id = 1; // User ID
        sys.mint_to(user_id, 10000000000000);
        (sys, user_id)
    }

    #[test]
    fn test_init_success() {
        let (sys, user_id) = create_system_and_user();
        let program = Program::current(&sys);

        let init_msg = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 10,
            max_pebbles_per_turn: 3,
        };

        program.send_bytes(user_id, init_msg.encode());

        // Check the initial state
        let state: GameState = program.read_state(()).expect("Failed to read state");
        println!("{:?}", state);
        assert_eq!(state.pebbles_count, 10);
        assert_eq!(state.max_pebbles_per_turn, 3);
        assert!(state.pebbles_remaining <= 10); // Adjust based on initial player
        assert!(state.first_player == Player::User || state.first_player == Player::Program);
    }

    #[test]
    fn test_who_turn() {
        let (sys, user_id) = create_system_and_user();
        let program = Program::current(&sys);

        let init_msg = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 10,
            max_pebbles_per_turn: 3,
        };

        program.send_bytes(user_id, init_msg.encode());

        let turn_action = PebblesAction::Turn(3);
        program.send_bytes(user_id, turn_action.encode());

        // Check the state after the turn
        let state: GameState = program.read_state(()).expect("Failed to read state");
        println!("State: {:?}", state);
        assert!(state.pebbles_remaining <= 7); // Depending on who plays first
        assert!(state.first_player == Player::Program || state.first_player == Player::User);
    }

    #[test]
    fn test_who_wins() {
        let (sys, user_id) = create_system_and_user();
        let program = Program::current(&sys);

        let init_msg = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 1,
            max_pebbles_per_turn: 1,
        };

        program.send_bytes(user_id, init_msg.encode());

        let turn_action = PebblesAction::Turn(1);
        program.send_bytes(user_id, turn_action.encode());

        // Check the state after the turn to determine the winner
        let state: GameState = program.read_state(()).expect("Failed to read state");
        println!("State: {:?}", state);
        assert_eq!(state.winner, Some(Player::User)); // Adjust based on your game logic
    }

    #[test]
    fn test_restart_game() {
        let (sys, user_id) = create_system_and_user();
        let program = Program::current(&sys);

        let init_msg = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 10,
            max_pebbles_per_turn: 3,
        };

        program.send_bytes(user_id, init_msg.encode());

        let restart_action = PebblesAction::Restart {
            difficulty: DifficultyLevel::Hard,
            pebbles_count: 20,
            max_pebbles_per_turn: 5,
        };

        program.send_bytes(user_id, restart_action.encode());

        // Check the state after restarting the game
        let state: GameState = program.read_state(()).expect("Failed to read state");
        println!("{:?}", state);
        assert_eq!(state.pebbles_count, 20);
        assert_eq!(state.max_pebbles_per_turn, 5);
        assert_eq!(state.pebbles_remaining, 20);
    }

    #[test]
    fn test_give_up() {
        let (sys, user_id) = create_system_and_user();
        let program = Program::current(&sys);

        let init_msg = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 10,
            max_pebbles_per_turn: 3,
        };

        program.send_bytes(user_id, init_msg.encode());

        let give_up_action = PebblesAction::GiveUp;
        program.send_bytes(user_id, give_up_action.encode());

        // Check the state after giving up
        let state: GameState = program.read_state(()).expect("Failed to read state");
        println!("{:?}", state);
        assert_eq!(state.winner, Some(Player::Program));
    }
}
