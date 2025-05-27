use wasm_bindgen::prelude::*;
use web_sys::KeyboardEvent;
use std::cell::RefCell;
use std::rc::Rc;
use crate::game::GameState;

pub struct InputHandler {
    game_state: Rc<RefCell<GameState>>,
}

impl InputHandler {
    pub fn new(game_state: Rc<RefCell<GameState>>) -> Self {
        InputHandler { game_state }
    }
    
    pub fn handle_keydown(&self, event: KeyboardEvent) {
        let mut state = self.game_state.borrow_mut();
        match event.key().as_str() {
            "ArrowLeft" | "a" | "A" => state.player.speed = -0.03,
            "ArrowRight" | "d" | "D" => state.player.speed = 0.03,
            " " => state.shoot(),
            _ => {}
        }
    }
    
    pub fn handle_keyup(&self, event: KeyboardEvent) {
        let mut state = self.game_state.borrow_mut();
        match event.key().as_str() {
            "ArrowLeft" | "a" | "A" | "ArrowRight" | "d" | "D" => {
                state.player.speed = 0.02;
            }
            _ => {}
        }
    }
}