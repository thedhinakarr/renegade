use crate::game::state::GameState;
use crate::game::entities::*;
use crate::game::physics::*;
use crate::constants::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

impl GameState {
    pub fn update(&mut self, delta: f64) {
        if self.game_over {
            return;
        }

        self.time += delta;
        
        // Update subsystems
        self.update_screen_shake(delta);
        self.update_combo_timer(delta);
        self.update_player();
        self.spawn_threats(delta);
        self.update_threats();
        self.update_projectiles();
        self.update_particles(delta);
    }
    
    fn update_screen_shake(&mut self, delta: f64) {
        if self.screen_shake > 0.0 {
            self.screen_shake -= delta * 0.01;
            if self.screen_shake < 0.0 {
                self.screen_shake = 0.0;
            }
        }
    }
    
    fn update_combo_timer(&mut self, delta: f64) {
        if self.combo_timer > 0.0 {
            self.combo_timer -= delta * 0.001;
            if self.combo_timer <= 0.0 {
                self.combo = 0;
            }
        }
    }
    
    fn update_player(&mut self) {
        self.player.angle += self.player.speed;
        if self.player.angle > std::f64::consts::PI * 2.0 {
            self.player.angle -= std::f64::consts::PI * 2.0;
        }
    }
    
    fn spawn_threats(&mut self, delta: f64) {
        let spawn_rate = 2000.0 - (self.time * 0.02).min(1500.0);
        if self.time % spawn_rate < delta {
            self.spawn_threat();
        }
    }
    
    fn spawn_threat(&mut self) {
        let angle = js_sys::Math::random() * std::f64::consts::PI * 2.0;
        let spawn_dist = 400.0;
        
        let x = CANVAS_WIDTH / 2.0 + angle.cos() * spawn_dist;
        let y = CANVAS_HEIGHT / 2.0 + angle.sin() * spawn_dist;
        
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
    
    fn update_threats(&mut self) {
        let mut explosions_to_create = Vec::new();
        let mut shake_to_add = 0.0;
        
        self.threats.retain_mut(|threat| {
            threat.x += threat.vx;
            threat.y += threat.vy;
            
            if is_threat_hitting_planet(threat) {
                self.planet_health -= 10;
                self.combo = 0;
                shake_to_add += 5.0;
                explosions_to_create.push((threat.x, threat.y, (255, 100, 100), 20));
                console_log!("Planet hit! Health: {}", self.planet_health);
                
                if self.planet_health <= 0 {
                    self.planet_health = 0;
                    self.game_over = true;
                    console_log!("GAME OVER! Final Score: {}", self.score);
                }
                
                return false;
            }
            
            is_on_screen(threat.x, threat.y, 50.0)
        });
        
        // Apply effects after iteration
        for (x, y, color, count) in explosions_to_create {
            self.create_explosion(x, y, color, count);
        }
        self.add_screen_shake(shake_to_add);
    }
    
    fn update_projectiles(&mut self) {
        let mut hit_positions = Vec::new();
        let mut shake_to_add = 0.0;
        
        self.projectiles.retain_mut(|proj| {
            proj.x += proj.vx;
            proj.y += proj.vy;
            
            for threat in &mut self.threats {
                if check_collision(proj.x, proj.y, proj.radius, threat.x, threat.y, threat.radius) {
                    self.combo += 1;
                    self.combo_timer = 2.0;
                    
                    let points = 10 * self.combo.min(10);
                    self.score += points;
                    
                    hit_positions.push((threat.x, threat.y));
                    shake_to_add += 2.0;
                    
                    threat.radius = 0.0;
                    return false;
                }
            }
            
            is_on_screen(proj.x, proj.y, 10.0)
        });
        
        self.threats.retain(|t| t.radius > 0.0);
        
        for (x, y) in hit_positions {
            self.create_explosion(x, y, (255, 200, 100), 15);
        }
        self.add_screen_shake(shake_to_add);
    }
    
    fn update_particles(&mut self, delta: f64) {
        self.particles.retain_mut(|particle| {
            particle.x += particle.vx;
            particle.y += particle.vy;
            particle.vx *= 0.98;
            particle.vy *= 0.98;
            particle.lifetime -= delta * 0.002;
            particle.lifetime > 0.0
        });
    }
}