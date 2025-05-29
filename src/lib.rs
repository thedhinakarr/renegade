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
    particles: Vec<Particle>,
    score: u32,
    planet_health: i32,
    time: f64,
    game_over: bool,
    combo: u32,
    combo_timer: f64,
    screen_shake: f64,
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

struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    size: f64,
    lifetime: f64,
    max_lifetime: f64,
    color: (u8, u8, u8),
}

impl GameState {
    fn new() -> Self {
        GameState {
            player_angle: 0.0,
            player_speed: 0.02,
            threats: Vec::new(),
            projectiles: Vec::new(),
            particles: Vec::new(),
            score: 0,
            planet_health: 100,
            time: 0.0,
            game_over: false,
            combo: 0,
            combo_timer: 0.0,
            screen_shake: 0.0,
        }
    }
    
    fn create_explosion(&mut self, x: f64, y: f64, color: (u8, u8, u8), count: u32) {
        for _ in 0..count {
            let angle = js_sys::Math::random() * f64::consts::PI * 2.0;
            let speed = js_sys::Math::random() * 4.0 + 2.0;
            
            self.particles.push(Particle {
                x,
                y,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                size: js_sys::Math::random() * 4.0 + 2.0,
                lifetime: 1.0,
                max_lifetime: 1.0,
                color,
            });
        }
    }
    
    fn add_screen_shake(&mut self, intensity: f64) {
        self.screen_shake = (self.screen_shake + intensity).min(10.0);
    }

    fn update(&mut self, delta: f64) {
        // Don't update if game is over
        if self.game_over {
            return;
        }

        self.time += delta;
        
        // Update screen shake
        if self.screen_shake > 0.0 {
            self.screen_shake -= delta * 0.01;
            if self.screen_shake < 0.0 {
                self.screen_shake = 0.0;
            }
        }
        
        // Update combo timer
        if self.combo_timer > 0.0 {
            self.combo_timer -= delta * 0.001;
            if self.combo_timer <= 0.0 {
                self.combo = 0;
            }
        }
        
        // Update player position (orbit)
        self.player_angle += self.player_speed;
        if self.player_angle > f64::consts::PI * 2.0 {
            self.player_angle -= f64::consts::PI * 2.0;
        }

        // Spawn threats periodically (increase with time)
        let spawn_rate = 2000.0 - (self.time * 0.02).min(1500.0);
        if self.time % spawn_rate < delta {
            self.spawn_threat();
        }

        // Update threats
        let mut explosions_to_create = Vec::new();
        let mut shake_to_add = 0.0;
        
        self.threats.retain_mut(|threat| {
            threat.x += threat.vx;
            threat.y += threat.vy;
            
            // Check if threat hit planet
            let dist_to_center = ((threat.x - CANVAS_WIDTH/2.0).powi(2) + 
                                 (threat.y - CANVAS_HEIGHT/2.0).powi(2)).sqrt();
            
            if dist_to_center < PLANET_RADIUS {
                self.planet_health -= 10;
                self.combo = 0;  // Reset combo on hit
                shake_to_add += 5.0;
                explosions_to_create.push((threat.x, threat.y, (255, 100, 100), 20));
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
        let mut hit_positions = Vec::new();
        self.projectiles.retain_mut(|proj| {
            proj.x += proj.vx;
            proj.y += proj.vy;
            
            // Check collisions with threats
            for threat in &mut self.threats {
                let dx = proj.x - threat.x;
                let dy = proj.y - threat.y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist < proj.radius + threat.radius {
                    // Combo system
                    self.combo += 1;
                    self.combo_timer = 2.0;
                    
                    // Score with combo multiplier
                    let points = 10 * self.combo.min(10);
                    self.score += points;
                    
                    // Store hit info for later
                    hit_positions.push((threat.x, threat.y));
                    shake_to_add += 2.0;
                    
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
        
        // Create explosions after the retain_mut
        for (x, y, color, count) in explosions_to_create {
            self.create_explosion(x, y, color, count);
        }
        
        for (x, y) in hit_positions {
            self.create_explosion(x, y, (255, 200, 100), 15);
        }
        
        self.add_screen_shake(shake_to_add);
        
        // Update particles
        self.particles.retain_mut(|particle| {
            particle.x += particle.vx;
            particle.y += particle.vy;
            particle.vx *= 0.98; // Friction
            particle.vy *= 0.98;
            particle.lifetime -= delta * 0.002;
            particle.lifetime > 0.0
        });
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

        // Clear canvas with very dark background
        ctx.set_fill_style(&"#000000".into());
        ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);
        
        // Add subtle grid overlay at bottom
        ctx.set_stroke_style(&"rgba(255, 0, 100, 0.1)".into());
        ctx.set_line_width(1.0);
        
        // Horizontal lines getting closer together toward bottom (perspective)
        for i in 0..20 {
            let y = CANVAS_HEIGHT * 0.7 + (i as f64).powf(1.5) * 8.0;
            if y < CANVAS_HEIGHT {
                ctx.begin_path();
                ctx.move_to(0.0, y);
                ctx.line_to(CANVAS_WIDTH, y);
                ctx.stroke();
            }
        }
        
        // Vertical lines for grid
        for i in 0..40 {
            let x = (i as f64 - 20.0) * 40.0 + CANVAS_WIDTH / 2.0;
            ctx.begin_path();
            ctx.move_to(x, CANVAS_HEIGHT * 0.7);
            ctx.line_to(x + (x - CANVAS_WIDTH / 2.0) * 0.3, CANVAS_HEIGHT);
            ctx.stroke();
        }

        let state_ref = state.borrow();
        
        // Apply screen shake
        if state_ref.screen_shake > 0.0 {
            let shake_x = (js_sys::Math::random() - 0.5) * state_ref.screen_shake;
            let shake_y = (js_sys::Math::random() - 0.5) * state_ref.screen_shake;
            ctx.save();
            ctx.translate(shake_x, shake_y).unwrap();
        }

        // Draw minimal stars - just a few bright ones
        ctx.set_fill_style(&"rgba(255, 255, 255, 0.8)".into());
        for i in 0..15 {
            let x = (i * 137 % 800) as f64;
            let y = (i * 73 % 300) as f64;
            ctx.fill_rect(x, y, 1.0, 1.0);
        }

        // Draw planet - dark with subtle red underglow
        // Simple red glow effect
        ctx.set_fill_style(&"rgba(200, 0, 0, 0.2)".into());
        ctx.begin_path();
        ctx.arc(
            CANVAS_WIDTH / 2.0,
            CANVAS_HEIGHT / 2.0,
            PLANET_RADIUS * 2.0,
            0.0,
            f64::consts::PI * 2.0,
        ).unwrap();
        ctx.fill();
        
        // Planet - very dark
        ctx.set_fill_style(&"#0a0a0a".into());
        ctx.begin_path();
        ctx.arc(
            CANVAS_WIDTH / 2.0,
            CANVAS_HEIGHT / 2.0,
            PLANET_RADIUS,
            0.0,
            f64::consts::PI * 2.0,
        ).unwrap();
        ctx.fill();
        
        // Subtle rim light
        ctx.set_stroke_style(&"rgba(200, 0, 50, 0.5)".into());
        ctx.set_line_width(1.0);
        ctx.stroke();
        
        // Planet - very dark
        ctx.set_fill_style(&"#0a0a0a".into());
        ctx.begin_path();
        ctx.arc(
            CANVAS_WIDTH / 2.0,
            CANVAS_HEIGHT / 2.0,
            PLANET_RADIUS,
            0.0,
            f64::consts::PI * 2.0,
        ).unwrap();
        ctx.fill();
        
        // Subtle rim light
        ctx.set_stroke_style(&"rgba(200, 0, 50, 0.5)".into());
        ctx.set_line_width(1.0);
        ctx.stroke();

        // Draw orbit path - very subtle
        ctx.set_stroke_style(&"rgba(100, 0, 0, 0.2)".into());
        ctx.set_line_width(1.0);
        ctx.begin_path();
        ctx.arc(
            CANVAS_WIDTH / 2.0,
            CANVAS_HEIGHT / 2.0,
            ORBIT_RADIUS,
            0.0,
            f64::consts::PI * 2.0,
        ).unwrap();
        ctx.stroke();

        // Draw particles (before other objects for background effect)
        for particle in &state_ref.particles {
            let alpha = particle.lifetime / particle.max_lifetime;
            ctx.set_fill_style(&JsValue::from_str(&format!(
                "rgba({}, {}, {}, {})",
                particle.color.0, particle.color.1, particle.color.2, alpha
            )));
            ctx.fill_rect(
                particle.x - particle.size / 2.0,
                particle.y - particle.size / 2.0,
                particle.size,
                particle.size,
            );
        }

        // Draw player - minimal with red accent
        let player_x = CANVAS_WIDTH / 2.0 + state_ref.player_angle.cos() * ORBIT_RADIUS;
        let player_y = CANVAS_HEIGHT / 2.0 + state_ref.player_angle.sin() * ORBIT_RADIUS;
        
        // Simple red glow behind player
        ctx.set_fill_style(&"rgba(255, 0, 0, 0.3)".into());
        ctx.begin_path();
        ctx.arc(player_x, player_y, PLAYER_SIZE, 0.0, f64::consts::PI * 2.0).unwrap();
        ctx.fill();
        
        // Player ship - white/gray
        ctx.set_fill_style(&"#cccccc".into());
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

        // Draw threats - dark with red glow
        for threat in &state_ref.threats {
            // Subtle red glow
            ctx.set_fill_style(&"rgba(200, 0, 0, 0.4)".into());
            ctx.begin_path();
            ctx.arc(threat.x, threat.y, threat.radius * 1.5, 0.0, f64::consts::PI * 2.0).unwrap();
            ctx.fill();
            
            // Threat core - dark red
            ctx.set_fill_style(&"#660000".into());
            ctx.begin_path();
            ctx.arc(threat.x, threat.y, threat.radius, 0.0, f64::consts::PI * 2.0).unwrap();
            ctx.fill();
        }

        // Draw projectiles - bright red/orange
        for proj in &state_ref.projectiles {
            // Projectile glow
            ctx.set_fill_style(&"rgba(255, 100, 0, 0.6)".into());
            ctx.begin_path();
            ctx.arc(proj.x, proj.y, proj.radius * 2.0, 0.0, f64::consts::PI * 2.0).unwrap();
            ctx.fill();
            
            // Projectile core
            ctx.set_fill_style(&"#ff6600".into());
            ctx.begin_path();
            ctx.arc(proj.x, proj.y, proj.radius, 0.0, f64::consts::PI * 2.0).unwrap();
            ctx.fill();
        }
        
        // Restore canvas if shaking
        if state_ref.screen_shake > 0.0 {
            ctx.restore();
        }

        // Draw UI
        ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        ctx.set_font("20px Arial");
        ctx.fill_text(&format!("Score: {}", state_ref.score), 10.0, 30.0).unwrap();
        ctx.fill_text(&format!("Planet Health: {}", state_ref.planet_health), 10.0, 60.0).unwrap();
        
        // Draw combo
        if state_ref.combo > 1 {
            ctx.set_fill_style(&JsValue::from_str("#FFD700"));
            ctx.set_font("24px Arial");
            ctx.fill_text(&format!("{}x COMBO!", state_ref.combo), 10.0, 90.0).unwrap();
        }

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