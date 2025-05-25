use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};
use std::f64;
use std::cell::RefCell;
use std::rc::Rc;

// Game constants
const CANVAS_WIDTH: f64 = 800.0;
const CANVAS_HEIGHT: f64 = 600.0;
const PLANET_RADIUS: f64 = 50.0;
const ORBIT_RADIUS: f64 = 150.0;
const PLAYER_SIZE: f64 = 20.0;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Game state
struct GameState {
    player_angle: f64,
    player_speed: f64,
    threats: Vec<Threat>,
    projectiles: Vec<Projectile>,
    score: u32,
    planet_health: i32,
    time: f64,
    game_over: bool,
}

struct Threat {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    radius: f64,
}

struct Projectile {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    radius: f64,
}

impl GameState {
    fn new() -> Self {
        GameState {
            player_angle: 0.0,
            player_speed: 0.02,
            threats: Vec::new(),
            projectiles: Vec::new(),
            score: 0,
            planet_health: 100,
            time: 0.0,
            game_over: false,
        }
    }

    fn update(&mut self, delta: f64) {
        // Don't update if game is over
        if self.game_over {
            return;
        }

        self.time += delta;
        
        // Update player position (orbit)
        self.player_angle += self.player_speed;
        if self.player_angle > f64::consts::PI * 2.0 {
            self.player_angle -= f64::consts::PI * 2.0;
        }

        // Spawn threats periodically
        if self.time % 2000.0 < delta {
            self.spawn_threat();
        }

        // Update threats
        self.threats.retain_mut(|threat| {
            threat.x += threat.vx;
            threat.y += threat.vy;
            
            // Check if threat hit planet
            let dist_to_center = ((threat.x - CANVAS_WIDTH/2.0).powi(2) + 
                                 (threat.y - CANVAS_HEIGHT/2.0).powi(2)).sqrt();
            
            if dist_to_center < PLANET_RADIUS {
                self.planet_health -= 10;
                console_log!("Planet hit! Health: {}", self.planet_health);
                
                // Check for game over
                if self.planet_health <= 0 {
                    self.planet_health = 0;
                    self.game_over = true;
                    console_log!("GAME OVER! Final Score: {}", self.score);
                }
                
                return false;
            }
            
            // Keep if still on screen
            threat.x > -50.0 && threat.x < CANVAS_WIDTH + 50.0 && 
            threat.y > -50.0 && threat.y < CANVAS_HEIGHT + 50.0
        });

        // Update projectiles
        self.projectiles.retain_mut(|proj| {
            proj.x += proj.vx;
            proj.y += proj.vy;
            
            // Check collisions with threats
            for threat in &mut self.threats {
                let dx = proj.x - threat.x;
                let dy = proj.y - threat.y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist < proj.radius + threat.radius {
                    self.score += 10;
                    threat.radius = 0.0; // Mark for removal
                    return false;
                }
            }
            
            // Keep if still on screen
            proj.x > -10.0 && proj.x < CANVAS_WIDTH + 10.0 && 
            proj.y > -10.0 && proj.y < CANVAS_HEIGHT + 10.0
        });

        // Remove destroyed threats
        self.threats.retain(|t| t.radius > 0.0);
    }

    fn spawn_threat(&mut self) {
        let angle = js_sys::Math::random() * f64::consts::PI * 2.0;
        let spawn_dist = 400.0;
        
        let x = CANVAS_WIDTH / 2.0 + angle.cos() * spawn_dist;
        let y = CANVAS_HEIGHT / 2.0 + angle.sin() * spawn_dist;
        
        // Aim at planet with some randomness
        let target_x = CANVAS_WIDTH / 2.0 + (js_sys::Math::random() - 0.5) * 100.0;
        let target_y = CANVAS_HEIGHT / 2.0 + (js_sys::Math::random() - 0.5) * 100.0;
        
        let dx = target_x - x;
        let dy = target_y - y;
        let dist = (dx * dx + dy * dy).sqrt();
        
        self.threats.push(Threat {
            x,
            y,
            vx: (dx / dist) * 2.0,
            vy: (dy / dist) * 2.0,
            radius: 15.0,
        });
    }

    fn shoot(&mut self) {
        // Can't shoot if game is over
        if self.game_over {
            return;
        }
        
        let player_x = CANVAS_WIDTH / 2.0 + self.player_angle.cos() * ORBIT_RADIUS;
        let player_y = CANVAS_HEIGHT / 2.0 + self.player_angle.sin() * ORBIT_RADIUS;
        
        // Shoot outward from orbit
        let vx = self.player_angle.cos() * 8.0;
        let vy = self.player_angle.sin() * 8.0;
        
        self.projectiles.push(Projectile {
            x: player_x,
            y: player_y,
            vx,
            vy,
            radius: 5.0,
        });
    }
}

// Main game loop
fn game_loop(
    ctx: Rc<CanvasRenderingContext2d>,
    state: Rc<RefCell<GameState>>,
    last_time: Rc<RefCell<f64>>,
) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        let delta = timestamp - *last_time.borrow();
        *last_time.borrow_mut() = timestamp;

        // Update game state
        state.borrow_mut().update(delta);

        // Clear canvas
        ctx.set_fill_style(&JsValue::from_str("#000011"));
        ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);

        // Draw stars
        ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        for i in 0..50 {
            let x = (i * 73 % 800) as f64;
            let y = (i * 37 % 600) as f64;
            ctx.fill_rect(x, y, 1.0, 1.0);
        }

        // Draw planet
        ctx.set_fill_style(&JsValue::from_str("#0066CC"));
        ctx.begin_path();
        ctx.arc(
            CANVAS_WIDTH / 2.0,
            CANVAS_HEIGHT / 2.0,
            PLANET_RADIUS,
            0.0,
            f64::consts::PI * 2.0,
        ).unwrap();
        ctx.fill();

        // Draw orbit path
        ctx.set_stroke_style(&JsValue::from_str("#333333"));
        ctx.begin_path();
        ctx.arc(
            CANVAS_WIDTH / 2.0,
            CANVAS_HEIGHT / 2.0,
            ORBIT_RADIUS,
            0.0,
            f64::consts::PI * 2.0,
        ).unwrap();
        ctx.stroke();

        // Draw player
        let state_ref = state.borrow();
        let player_x = CANVAS_WIDTH / 2.0 + state_ref.player_angle.cos() * ORBIT_RADIUS;
        let player_y = CANVAS_HEIGHT / 2.0 + state_ref.player_angle.sin() * ORBIT_RADIUS;
        
        ctx.set_fill_style(&JsValue::from_str("#00FF00"));
        ctx.save();
        ctx.translate(player_x, player_y).unwrap();
        ctx.rotate(state_ref.player_angle + f64::consts::PI / 2.0).unwrap();
        ctx.begin_path();
        ctx.move_to(0.0, -PLAYER_SIZE / 2.0);
        ctx.line_to(-PLAYER_SIZE / 3.0, PLAYER_SIZE / 2.0);
        ctx.line_to(PLAYER_SIZE / 3.0, PLAYER_SIZE / 2.0);
        ctx.close_path();
        ctx.fill();
        ctx.restore();

        // Draw threats
        ctx.set_fill_style(&JsValue::from_str("#FF3333"));
        for threat in &state_ref.threats {
            ctx.begin_path();
            ctx.arc(threat.x, threat.y, threat.radius, 0.0, f64::consts::PI * 2.0).unwrap();
            ctx.fill();
        }

        // Draw projectiles
        ctx.set_fill_style(&JsValue::from_str("#FFFF00"));
        for proj in &state_ref.projectiles {
            ctx.begin_path();
            ctx.arc(proj.x, proj.y, proj.radius, 0.0, f64::consts::PI * 2.0).unwrap();
            ctx.fill();
        }

        // Draw UI
        ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        ctx.set_font("20px Arial");
        ctx.fill_text(&format!("Score: {}", state_ref.score), 10.0, 30.0).unwrap();
        ctx.fill_text(&format!("Planet Health: {}", state_ref.planet_health), 10.0, 60.0).unwrap();

        // Draw game over screen
        if state_ref.game_over {
            // Dark overlay
            ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.7)"));
            ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);
            
            // Game over text
            ctx.set_fill_style(&JsValue::from_str("#FF0000"));
            ctx.set_font("48px Arial");
            ctx.set_text_align("center");
            ctx.fill_text("GAME OVER", CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0 - 50.0).unwrap();
            
            ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
            ctx.set_font("24px Arial");
            ctx.fill_text(&format!("Final Score: {}", state_ref.score), CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0 + 10.0).unwrap();
            
            ctx.set_font("16px Arial");
            ctx.fill_text("Press F5 to restart", CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0 + 50.0).unwrap();
            
            // Reset alignment
            ctx.set_text_align("left");
        }

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
    console_log!("Orbital Guardian starting...");

    // Get canvas
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

    let ctx = Rc::new(ctx);
    let state = Rc::new(RefCell::new(GameState::new()));
    let state_clone = state.clone();

    // Set up controls
    let keydown = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        match event.key().as_str() {
            "ArrowLeft" | "a" | "A" => state_clone.borrow_mut().player_speed = -0.03,
            "ArrowRight" | "d" | "D" => state_clone.borrow_mut().player_speed = 0.03,
            " " => state_clone.borrow_mut().shoot(),
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);

    let state_clone2 = state.clone();
    let keyup = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        match event.key().as_str() {
            "ArrowLeft" | "a" | "A" | "ArrowRight" | "d" | "D" => {
                state_clone2.borrow_mut().player_speed = 0.02;
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
    game_loop(ctx, state, last_time);

    Ok(())
}