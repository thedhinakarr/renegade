use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use wasm_bindgen::JsValue;
use crate::game::GameState;
use crate::constants::*;

pub struct Renderer {
    ctx: CanvasRenderingContext2d,
}

impl Renderer {
    pub fn new(ctx: CanvasRenderingContext2d) -> Self {
        Renderer { ctx }
    }
    
    pub fn render(&self, state: &GameState) {
        self.clear();
        
        // Apply screen shake if active
        if state.screen_shake > 0.0 {
            let shake_x = (js_sys::Math::random() - 0.5) * state.screen_shake;
            let shake_y = (js_sys::Math::random() - 0.5) * state.screen_shake;
            self.ctx.save();
            self.ctx.translate(shake_x, shake_y).unwrap();
        }
        
        self.draw_background();
        self.draw_grid();
        self.draw_stars();
        self.draw_planet(state);
        self.draw_orbit();
        self.draw_particles(state);
        self.draw_player(state);
        self.draw_threats(state);
        self.draw_projectiles(state);
        
        if state.screen_shake > 0.0 {
            self.ctx.restore();
        }
        
        self.draw_ui(state);
        
        if state.game_over {
            self.draw_game_over(state);
        }
    }
    
    fn clear(&self) {
        self.ctx.set_fill_style(&"#000000".into());
        self.ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);
    }
    
    fn draw_background(&self) {
        // Already cleared to black
    }
    
    fn draw_grid(&self) {
        self.ctx.set_stroke_style(&"rgba(255, 0, 100, 0.1)".into());
        self.ctx.set_line_width(1.0);
        
        // Horizontal lines
        for i in 0..20 {
            let y = CANVAS_HEIGHT * 0.7 + (i as f64).powf(1.5) * 8.0;
            if y < CANVAS_HEIGHT {
                self.ctx.begin_path();
                self.ctx.move_to(0.0, y);
                self.ctx.line_to(CANVAS_WIDTH, y);
                self.ctx.stroke();
            }
        }
        
        // Vertical lines
        for i in 0..40 {
            let x = (i as f64 - 20.0) * 40.0 + CANVAS_WIDTH / 2.0;
            self.ctx.begin_path();
            self.ctx.move_to(x, CANVAS_HEIGHT * 0.7);
            self.ctx.line_to(x + (x - CANVAS_WIDTH / 2.0) * 0.3, CANVAS_HEIGHT);
            self.ctx.stroke();
        }
    }
    
    fn draw_stars(&self) {
        self.ctx.set_fill_style(&"rgba(255, 255, 255, 0.8)".into());
        for i in 0..15 {
            let x = (i * 137 % 800) as f64;
            let y = (i * 73 % 300) as f64;
            self.ctx.fill_rect(x, y, 1.0, 1.0);
        }
    }
    
    fn draw_planet(&self, state: &GameState) {
        // Red glow
        self.ctx.set_fill_style(&"rgba(200, 0, 0, 0.2)".into());
        self.ctx.begin_path();
        self.ctx.arc(
            CANVAS_WIDTH / 2.0,
            CANVAS_HEIGHT / 2.0,
            PLANET_RADIUS * 2.0,
            0.0,
            std::f64::consts::PI * 2.0,
        ).unwrap();
        self.ctx.fill();
        
        // Planet
        self.ctx.set_fill_style(&"#0a0a0a".into());
        self.ctx.begin_path();
        self.ctx.arc(
            CANVAS_WIDTH / 2.0,
            CANVAS_HEIGHT / 2.0,
            PLANET_RADIUS,
            0.0,
            std::f64::consts::PI * 2.0,
        ).unwrap();
        self.ctx.fill();
        
        // Rim light
        self.ctx.set_stroke_style(&"rgba(200, 0, 50, 0.5)".into());
        self.ctx.set_line_width(1.0);
        self.ctx.stroke();
    }
    
    fn draw_orbit(&self) {
        self.ctx.set_stroke_style(&"rgba(100, 0, 0, 0.2)".into());
        self.ctx.set_line_width(1.0);
        self.ctx.begin_path();
        self.ctx.arc(
            CANVAS_WIDTH / 2.0,
            CANVAS_HEIGHT / 2.0,
            ORBIT_RADIUS,
            0.0,
            std::f64::consts::PI * 2.0,
        ).unwrap();
        self.ctx.stroke();
    }
    
    fn draw_particles(&self, state: &GameState) {
        for particle in &state.particles {
            let alpha = particle.lifetime / particle.max_lifetime;
            self.ctx.set_fill_style(&JsValue::from_str(&format!(
                "rgba({}, {}, {}, {})",
                particle.color.0, particle.color.1, particle.color.2, alpha
            )));
            self.ctx.fill_rect(
                particle.x - particle.size / 2.0,
                particle.y - particle.size / 2.0,
                particle.size,
                particle.size,
            );
        }
    }
    
    fn draw_player(&self, state: &GameState) {
        let (player_x, player_y) = state.player.get_position();
        
        // Glow
        self.ctx.set_fill_style(&"rgba(255, 0, 0, 0.3)".into());
        self.ctx.begin_path();
        self.ctx.arc(player_x, player_y, PLAYER_SIZE, 0.0, std::f64::consts::PI * 2.0).unwrap();
        self.ctx.fill();
        
        // Ship
        self.ctx.set_fill_style(&"#cccccc".into());
        self.ctx.save();
        self.ctx.translate(player_x, player_y).unwrap();
        self.ctx.rotate(state.player.angle + std::f64::consts::PI / 2.0).unwrap();
        self.ctx.begin_path();
        self.ctx.move_to(0.0, -PLAYER_SIZE / 2.0);
        self.ctx.line_to(-PLAYER_SIZE / 3.0, PLAYER_SIZE / 2.0);
        self.ctx.line_to(PLAYER_SIZE / 3.0, PLAYER_SIZE / 2.0);
        self.ctx.close_path();
        self.ctx.fill();
        self.ctx.restore();
    }
    
    fn draw_threats(&self, state: &GameState) {
        for threat in &state.threats {
            self.ctx.set_fill_style(&"rgba(200, 0, 0, 0.4)".into());
            self.ctx.begin_path();
            self.ctx.arc(threat.x, threat.y, threat.radius * 1.5, 0.0, std::f64::consts::PI * 2.0).unwrap();
            self.ctx.fill();
            
            self.ctx.set_fill_style(&"#660000".into());
            self.ctx.begin_path();
            self.ctx.arc(threat.x, threat.y, threat.radius, 0.0, std::f64::consts::PI * 2.0).unwrap();
            self.ctx.fill();
        }
    }
    
    fn draw_projectiles(&self, state: &GameState) {
        for proj in &state.projectiles {
            self.ctx.set_fill_style(&"rgba(255, 100, 0, 0.6)".into());
            self.ctx.begin_path();
            self.ctx.arc(proj.x, proj.y, proj.radius * 2.0, 0.0, std::f64::consts::PI * 2.0).unwrap();
            self.ctx.fill();
            
            self.ctx.set_fill_style(&"#ff6600".into());
            self.ctx.begin_path();
            self.ctx.arc(proj.x, proj.y, proj.radius, 0.0, std::f64::consts::PI * 2.0).unwrap();
            self.ctx.fill();
        }
    }
    
    fn draw_ui(&self, state: &GameState) {
        self.ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        self.ctx.set_font("20px Arial");
        self.ctx.fill_text(&format!("Score: {}", state.score), 10.0, 30.0).unwrap();
        self.ctx.fill_text(&format!("Planet Health: {}", state.planet_health), 10.0, 60.0).unwrap();
        
        if state.combo > 1 {
            self.ctx.set_fill_style(&JsValue::from_str("#FFD700"));
            self.ctx.set_font("24px Arial");
            self.ctx.fill_text(&format!("{}x COMBO!", state.combo), 10.0, 90.0).unwrap();
        }
    }
    
    fn draw_game_over(&self, state: &GameState) {
        self.ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.7)"));
        self.ctx.fill_rect(0.0, 0.0, CANVAS_WIDTH, CANVAS_HEIGHT);
        
        self.ctx.set_fill_style(&JsValue::from_str("#FF0000"));
        self.ctx.set_font("48px Arial");
        self.ctx.set_text_align("center");
        self.ctx.fill_text("GAME OVER", CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0 - 50.0).unwrap();
        
        self.ctx.set_fill_style(&JsValue::from_str("#FFFFFF"));
        self.ctx.set_font("24px Arial");
        self.ctx.fill_text(&format!("Final Score: {}", state.score), CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0 + 10.0).unwrap();
        
        self.ctx.set_font("16px Arial");
        self.ctx.fill_text("Press F5 to restart", CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0 + 50.0).unwrap();
        
        self.ctx.set_text_align("left");
    }
}