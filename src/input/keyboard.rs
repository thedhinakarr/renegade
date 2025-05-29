// src/input/keyboard.rs
use web_sys::KeyboardEvent;
use std::cell::RefCell;
use std::rc::Rc;
use crate::game::GameState;
// If you use constants like PLAYER_SPEED_INCREMENT, ensure they are defined or imported.
// For now, using the hardcoded values from your original code.

pub struct InputHandler {
    game_state: Rc<RefCell<GameState>>, 
}

impl InputHandler {
    pub fn new(game_state: Rc<RefCell<GameState>>) -> Self {
        InputHandler { game_state }
    }
    
    pub fn handle_keydown(&self, event: KeyboardEvent) {
        // Optional: Keep this log for now to see that this method is reached
        // web_sys::console::log_1(&format!("InputHandler::handle_keydown - Key: {} - Applying game logic.", event.key()).into());

        // --- RESTORED GAME LOGIC ---
        let mut state = self.game_state.borrow_mut(); 
        match event.key().as_str() {
            "ArrowLeft" | "a" | "A" => {
                state.player.speed = -0.03; // Or your defined constant for player speed
            }
            "ArrowRight" | "d" | "D" => {
                state.player.speed = 0.03; // Or your defined constant for player speed
            }
            " " => { // Space bar
                // The state.shoot() method itself should still have its internal
                // audio_manager.play_sfx("shoot") call commented out for this specific test.
                state.shoot(); 
            }
            _ => {}
        }
        // --- END OF RESTORED GAME LOGIC ---
    }
    
    pub fn handle_keyup(&self, event: KeyboardEvent) {
        // Optional: Keep this log
        // web_sys::console::log_1(&format!("InputHandler::handle_keyup - Key: {}", event.key()).into());

        // --- RESTORED GAME LOGIC ---
        let mut state = self.game_state.borrow_mut();
        match event.key().as_str() {
            "ArrowLeft" | "a" | "A" | "ArrowRight" | "d" | "D" => {
                state.player.speed = 0.0; // Stop player movement, or to a default idle/drift speed
            }
            _ => {}
        }
        // --- END OF RESTORED GAME LOGIC ---
    }
}