// src/rendering/renderer.rs
use web_sys::CanvasRenderingContext2d;
use wasm_bindgen::JsValue;
use crate::game::GameState; // Make sure GameState is imported if used by methods
use crate::constants::*;

// Define a simple struct for stars
struct Star {
    x: f64,
    y: f64,
    size: f64,
    // Future: maybe color/opacity
}

pub struct Renderer {
    ctx: CanvasRenderingContext2d,
    stars: Vec<Star>, // ADDED: Store star positions
}

impl Renderer {
    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        let mut stars = Vec::new();
        let num_stars = 50; // Or any number you like
        for _ in 0..num_stars {
            stars.push(Star {
                x: (js_sys::Math::random() * CANVAS_WIDTH).floor(),
                y: (js_sys::Math::random() * CANVAS_HEIGHT * 0.9).floor(), // Allow stars a bit lower
                size: js_sys::Math::random() * 1.5 + 0.5,
            });
        }
        Renderer { ctx, stars } // Initialize stars
    }
    
    pub fn render(&self, state: &GameState) {
        self.clear();
        
        let translation_applied = state.screen_shake > 0.0;
        if translation_applied {
             let shake_x = (js_sys::Math::random() - 0.5) * state.screen_shake;
             let shake_y = (js_sys::Math::random() - 0.5) * state.screen_shake;
             self.ctx.save();
             let _ = self.ctx.translate(shake_x, shake_y);
        }

        self.draw_background();
        self.draw_grid();
        self.draw_stars(); // Will now use stored stars
        self.draw_planet(state);
        self.draw_orbit();
        self.draw_particles(state);
        self.draw_player(state);
        self.draw_threats(state);
        self.draw_projectiles(state);
        
        if translation_applied {
            self.ctx.restore();
        }
        
        self.draw_ui(state);
        
        if state.game_over {
            self.draw_game_over(state);
        }
    }
    
    fn clear(&self) {
        self.ctx.set_fill_style(&JsValue::from_str("#000000"));
        self.ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);
    }
    
    fn draw_background(&self) { /* Cleared by self.clear() */ }
    
    fn draw_grid(&self) {
        self.ctx.set_stroke_style(&JsValue::from_str("rgba(255, 0, 100, 0.1)"));
        self.ctx.set_line_width(1.0);
        for i in 0..20 {
            let y = CANVAS_HEIGHT * 0.7 + (i as f64).powf(1.5) * 8.0;
            if y < CANVAS_HEIGHT {
                self.ctx.begin_path(); self.ctx.move_to(0.0, y); self.ctx.line_to(CANVAS_WIDTH, y); let _ = self.ctx.stroke();
            }
        }
        for i in 0..40 {
            let x = (i as f64 - 20.0) * 40.0 + CANVAS_WIDTH / 2.0;
            self.ctx.begin_path(); self.ctx.move_to(x, CANVAS_HEIGHT * 0.7); self.ctx.line_to(x + (x - CANVAS_WIDTH / 2.0) * 0.3, CANVAS_HEIGHT); let _ = self.ctx.stroke();
        }
    }
    
    // MODIFIED: draw_stars now iterates over the stored self.stars
    fn draw_stars(&self) {
        self.ctx.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.7)")); // Slightly less opaque
        for star in &self.stars {
            self.ctx.fill_rect(star.x, star.y, star.size, star.size);
        }
    }
    
    fn draw_planet(&self, _state: &GameState) { 
        self.ctx.set_fill_style(&JsValue::from_str("rgba(200, 0, 0, 0.2)"));
        self.ctx.begin_path(); let _ = self.ctx.arc(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0, PLANET_RADIUS * 2.0, 0.0, std::f64::consts::PI * 2.0); let _ = self.ctx.fill();
        self.ctx.set_fill_style(&JsValue::from_str("#0a0a0a"));
        self.ctx.begin_path(); let _ = self.ctx.arc(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0, PLANET_RADIUS, 0.0, std::f64::consts::PI * 2.0); let _ = self.ctx.fill();
        self.ctx.set_stroke_style(&JsValue::from_str("rgba(200, 0, 50, 0.5)"));
        self.ctx.set_line_width(1.0); let _ = self.ctx.stroke();
    }
    
    fn draw_orbit(&self) {
        self.ctx.set_stroke_style(&JsValue::from_str("rgba(100, 0, 0, 0.2)"));
        self.ctx.set_line_width(1.0); self.ctx.begin_path();
        let _ = self.ctx.arc(CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0, ORBIT_RADIUS, 0.0, std::f64::consts::PI * 2.0); let _ = self.ctx.stroke();
    }
    
    fn draw_particles(&self, state: &GameState) {
        for particle in &state.particles {
            let alpha = particle.lifetime / particle.max_lifetime;
            self.ctx.set_fill_style(&JsValue::from_str(&format!("rgba({}, {}, {}, {})", particle.color.0, particle.color.1, particle.color.2, alpha)));
            self.ctx.fill_rect(particle.x - particle.size / 2.0, particle.y - particle.size / 2.0, particle.size, particle.size);
        }
    }
    
    fn draw_player(&self, state: &GameState) {
        let (player_x, player_y) = state.player.get_position();
        self.ctx.set_fill_style(&JsValue::from_str("rgba(255, 0, 0, 0.3)"));
        self.ctx.begin_path(); let _ = self.ctx.arc(player_x, player_y, PLAYER_SIZE, 0.0, std::f64::consts::PI * 2.0); let _ = self.ctx.fill();
        self.ctx.set_fill_style(&JsValue::from_str("#cccccc"));
        self.ctx.save();
        let _ = self.ctx.translate(player_x, player_y);
        let _ = self.ctx.rotate(state.player.angle + std::f64::consts::PI / 2.0);
        self.ctx.begin_path(); self.ctx.move_to(0.0, -PLAYER_SIZE / 2.0);
        self.ctx.line_to(-PLAYER_SIZE / 3.0, PLAYER_SIZE / 2.0); self.ctx.line_to(PLAYER_SIZE / 3.0, PLAYER_SIZE / 2.0);
        self.ctx.close_path(); let _ = self.ctx.fill();
        self.ctx.restore();
    }
    
    fn draw_threats(&self, state: &GameState) {
        for threat in &state.threats {
            self.ctx.set_fill_style(&JsValue::from_str("rgba(200, 0, 0, 0.4)"));
            self.ctx.begin_path(); let _ = self.ctx.arc(threat.x, threat.y, threat.radius * 1.5, 0.0, std::f64::consts::PI * 2.0); let _ = self.ctx.fill();
            self.ctx.set_fill_style(&JsValue::from_str("#660000"));
            self.ctx.begin_path(); let _ = self.ctx.arc(threat.x, threat.y, threat.radius, 0.0, std::f64::consts::PI * 2.0); let _ = self.ctx.fill();
        }
    }
    
    fn draw_projectiles(&self, state: &GameState) {
        if state.projectiles.is_empty() && state.time < 5.0 { // Log only early if no projectiles
            // web_sys::console::log_1(&"draw_projectiles called, no projectiles to draw".into());
        }
        for proj in &state.projectiles {
            // web_sys::console::log_1(&format!("Drawing projectile at: ({}, {})", proj.x, proj.y).into());
            self.ctx.set_fill_style(&JsValue::from_str("rgba(255, 100, 0, 0.8)")); // Brighter for visibility
            self.ctx.begin_path(); 
            let _ = self.ctx.arc(proj.x, proj.y, proj.radius * 1.5, 0.0, std::f64::consts::PI * 2.0); // Slightly larger apparent radius
            let _ = self.ctx.fill();
            
            self.ctx.set_fill_style(&JsValue::from_str("#ffaa00")); // Brighter core
            self.ctx.begin_path(); 
            let _ = self.ctx.arc(proj.x, proj.y, proj.radius, 0.0, std::f64::consts::PI * 2.0); 
            let _ = self.ctx.fill();
        }
    }
    
    fn draw_ui(&self, state: &GameState) {
        self.ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        self.ctx.set_font("20px Arial");
        let _ = self.ctx.fill_text(&format!("Score: {}", state.score), 10.0, 30.0);
        let _ = self.ctx.fill_text(&format!("Planet Health: {}", state.planet_health), 10.0, 60.0);
        if state.combo > 1 {
            self.ctx.set_fill_style(&JsValue::from_str("#FFD700"));
            self.ctx.set_font("24px Arial");
            let _ = self.ctx.fill_text(&format!("{}x COMBO!", state.combo), 10.0, 90.0);
        }
    }
    
    fn draw_game_over(&self, state: &GameState) {
        self.ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.7)"));
        self.ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);
        self.ctx.set_fill_style(&JsValue::from_str("#FF0000"));
        self.ctx.set_font("48px Arial");
        self.ctx.set_text_align("center");
        let _ = self.ctx.fill_text("GAME OVER", CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0 - 50.0);
        self.ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        self.ctx.set_font("24px Arial");
        let _ = self.ctx.fill_text(&format!("Final Score: {}", state.score), CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0 + 10.0);
        self.ctx.set_font("16px Arial");
        let _ = self.ctx.fill_text("Press F5 to restart", CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0 + 50.0);
        self.ctx.set_text_align("left");
    }
}