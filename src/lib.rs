use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};
use std::f64;
use std::cell::RefCell;
use std::rc::Rc;

// Import your modular system
mod constants;
mod audio;
mod game;
mod input;
mod rendering;

use constants::*;
use audio::Sound;
use game::GameState;
use input::InputHandler;
use rendering::Renderer;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Main game loop - FIXED
fn game_loop(
    renderer: Rc<Renderer>,
    state: Rc<RefCell<GameState>>,
    last_time: Rc<RefCell<f64>>,
) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        let delta = timestamp - *last_time.borrow();
        *last_time.borrow_mut() = timestamp;

        // Convert milliseconds to seconds for game logic
        let dt = delta / 1000.0;

        // Update game state - CRITICAL FIX
        state.borrow_mut().update(dt);

        // Render
        renderer.render(&state.borrow());

        // Continue loop
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("Failed to register animation frame");
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_log!("RENEGADE starting with working audio...");

    // Test audio on startup
    audio::play(Sound::Shoot);

    // Get canvas and document
    let document = window().unwrap().document().unwrap();
    let canvas = document
        .get_element_by_id("game-canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()?;

    canvas.set_width(CANVAS_WIDTH as u32);
    canvas.set_height(CANVAS_HEIGHT as u32);

    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    // Hide loading message
    if let Some(loading) = document.get_element_by_id("loading") {
        loading.set_attribute("style", "display: none")?;
    }

    // Start background music on first user interaction
    let start_music = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        console_log!("Starting background music...");
        audio::looped(Sound::Background);
    }) as Box<dyn FnMut(_)>);
    
    document.add_event_listener_with_callback("click", start_music.as_ref().unchecked_ref())?;
    start_music.forget();
    
    // Add global restart function for background music looping
    let restart_music_func = Closure::wrap(Box::new(|| {
        audio::looped(Sound::Background);
    }) as Box<dyn Fn()>);
    
    js_sys::Reflect::set(
        &web_sys::window().unwrap(),
        &"restartBackgroundMusic".into(),
        restart_music_func.as_ref()
    )?;
    restart_music_func.forget();

    // Initialize game systems
    let renderer = Rc::new(Renderer::new(ctx));
    let state = Rc::new(RefCell::new(GameState::new()));

    // Set up controls - SIMPLIFIED
    let state_for_keydown = state.clone();
    let keydown = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        // Initialize audio on first key press
        audio::init_audio_context();
        
        let mut game_state = state_for_keydown.borrow_mut();
        match event.key().as_str() {
            "ArrowLeft" | "a" | "A" => {
                game_state.player.speed = -0.05; // Faster for visibility
            }
            "ArrowRight" | "d" | "D" => {
                game_state.player.speed = 0.05; // Faster for visibility
            }
            " " => {
                game_state.shoot();
            }
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);

    let state_for_keyup = state.clone();
    let keyup = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let mut game_state = state_for_keyup.borrow_mut();
        match event.key().as_str() {
            "ArrowLeft" | "a" | "A" | "ArrowRight" | "d" | "D" => {
                game_state.player.speed = 0.02; // Default orbital speed
            }
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);

    document.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())?;
    document.add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())?;
    
    keydown.forget();
    keyup.forget();

    // Start game loop
    let last_time = Rc::new(RefCell::new(0.0));
    game_loop(renderer, state, last_time);

    console_log!("RENEGADE initialization complete!");
    Ok(())
}