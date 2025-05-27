use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};
use std::cell::RefCell;
use std::rc::Rc;

// Assuming these modules are structured as in your project
mod constants;
mod game;
mod rendering;
mod input;

use constants::*;
use game::GameState;
use rendering::Renderer;
use input::InputHandler; // Make sure InputHandler is properly defined in input.rs

#[wasm_bindgen]
pub struct RenegadeGame {
    canvas: HtmlCanvasElement,
    renderer: Rc<Renderer>,
    game_state: Rc<RefCell<GameState>>,
    input_handler: Rc<InputHandler>, // MODIFIED: Changed from InputHandler to Rc<InputHandler>
}

#[wasm_bindgen]
impl RenegadeGame {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<RenegadeGame, JsValue> {
        // It's good practice to set this up for better panic messages in WASM
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
        
        let window = window().ok_or("should have a Window")?;
        let document = window.document().ok_or("should have a Document")?;
        let canvas = document
            .get_element_by_id("game-canvas")
            .ok_or("should have a canvas element with id 'game-canvas'")?
            .dyn_into::<HtmlCanvasElement>()?;
            
        canvas.set_width(CANVAS_WIDTH as u32);
        canvas.set_height(CANVAS_HEIGHT as u32);
        
        let ctx = canvas
            .get_context("2d")?
            .ok_or("should be able to get 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;
            
        let renderer = Rc::new(Renderer::new(ctx));
        let game_state = Rc::new(RefCell::new(GameState::new()));
        // MODIFIED: Wrap InputHandler in Rc and clone game_state for it
        let input_handler = Rc::new(InputHandler::new(game_state.clone())); 
        
        Ok(RenegadeGame {
            canvas,
            renderer,
            game_state,
            input_handler, // Now an Rc<InputHandler>
        })
    }
    
    pub fn init(&mut self) -> Result<(), JsValue> {
        let window = window().ok_or("should have a Window")?;
        let document = window.document().ok_or("should have a Document")?;
        
        // Hide loading message
        if let Some(loading) = document.get_element_by_id("loading") {
            loading.set_attribute("style", "display: none")?;
        }
        
        // Set up keyboard event handlers
        let keydown_handler = {
            // MODIFIED: Clone the Rc<InputHandler>
            let input_handler_clone = self.input_handler.clone(); 
            Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                // MODIFIED: Use the cloned Rc
                input_handler_clone.handle_keydown(event); 
            }) as Box<dyn FnMut(_)>)
        };
        
        let keyup_handler = {
            // MODIFIED: Clone the Rc<InputHandler>
            let input_handler_clone = self.input_handler.clone(); 
            Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                // MODIFIED: Use the cloned Rc
                input_handler_clone.handle_keyup(event); 
            }) as Box<dyn FnMut(_)>)
        };
        
        document.add_event_listener_with_callback("keydown", keydown_handler.as_ref().unchecked_ref())?;
        document.add_event_listener_with_callback("keyup", keyup_handler.as_ref().unchecked_ref())?;
        
        // The closures need to live as long as the event listeners are active.
        // .forget() means Rust will no longer manage the memory of these closures,
        // and they will live until the JavaScript garbage collector reclaims them (or the page closes).
        keydown_handler.forget();
        keyup_handler.forget();
        
        // Start game loop
        self.start_game_loop();
        
        Ok(())
    }
    
    // This method seems okay as renderer and game_state are already Rc
    fn start_game_loop(&self) {
        let renderer = self.renderer.clone();
        let game_state = self.game_state.clone();
        let last_time = Rc::new(RefCell::new(0.0)); // For calculating delta time
        
        // Create a persistent closure for the game loop
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
            let mut last_time_borrowed = last_time.borrow_mut();
            let delta = timestamp - *last_time_borrowed;
            *last_time_borrowed = timestamp;
            
            // Update game state (delta is typically in milliseconds)
            game_state.borrow_mut().update(delta / 1000.0); // Assuming update expects seconds
            
            // Render
            renderer.render(&game_state.borrow());
            
            // Continue loop
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut(f64)>));
        
        request_animation_frame(g.borrow().as_ref().unwrap());
    }
}

// Helper function to request animation frame
fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .unwrap() // Assuming window is always present in wasm context
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("Failed to register animation frame");
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Enable panic hook for better error messages (if not done in new())
    #[cfg(debug_assertions)]
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let mut game = RenegadeGame::new()?;
    game.init()?;
    Ok(())
}