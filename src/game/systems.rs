// src/game/systems.rs
use crate::game::state::GameState;
use crate::game::entities::{Threat}; 
use crate::game::physics::*;
use crate::constants::*;
use crate::audio::{self, Sound};

impl GameState {
   pub fn update(&mut self, dt: f64) {
        if self.game_over { return; }
        self.time += dt;

        self.update_player();
        self.spawn_threats(dt);
        self.update_threats();
        self.update_projectiles();
        self.update_particles(dt);
        self.update_combo_timer(dt);
        self.update_screen_shake(dt);
    }
    
    fn update_player(&mut self) {
        self.player.angle += self.player.speed;
        if self.player.angle >= std::f64::consts::PI * 2.0 {
            self.player.angle -= std::f64::consts::PI * 2.0;
        } else if self.player.angle < 0.0 {
            self.player.angle += std::f64::consts::PI * 2.0;
        }
    }
    
    fn spawn_threats(&mut self, delta: f64) {
        self.threat_spawn_timer -= delta;
        if self.threat_spawn_timer <= 0.0 {
            self.spawn_threat(); 
            let initial_interval_secs = 2.0;
            let min_interval_secs = 0.5;
            let time_to_reach_min_rate_secs = 60.0;
            let progression = (self.time / time_to_reach_min_rate_secs).min(1.0);
            let next_spawn_interval = initial_interval_secs - (initial_interval_secs - min_interval_secs) * progression;
            self.threat_spawn_timer = next_spawn_interval.max(min_interval_secs);
        }
    }
    
    fn spawn_threat(&mut self) {
        let angle = js_sys::Math::random() * std::f64::consts::PI * 2.0;
        let spawn_dist = CANVAS_WIDTH * 0.6;
        let x = CANVAS_WIDTH / 2.0 + angle.cos() * spawn_dist;
        let y = CANVAS_HEIGHT / 2.0 + angle.sin() * spawn_dist;
        let target_x = CANVAS_WIDTH / 2.0 + (js_sys::Math::random() - 0.5) * PLANET_RADIUS * 1.5;
        let target_y = CANVAS_HEIGHT / 2.0 + (js_sys::Math::random() - 0.5) * PLANET_RADIUS * 1.5;
        let dx = target_x - x;
        let dy = target_y - y;
        let dist_to_target = (dx * dx + dy * dy).sqrt();
        let threat_speed = 1.5 + js_sys::Math::random() * 1.0;
        self.threats.push(Threat {
            x, y,
            vx: (dx / dist_to_target.max(0.1)) * threat_speed,
            vy: (dy / dist_to_target.max(0.1)) * threat_speed,
            radius: 12.0 + js_sys::Math::random() * 6.0,
        });
    }
    
 fn update_threats(&mut self) {
        let mut planet_hit = false;

        self.threats.retain_mut(|t| {
            t.x += t.vx;
            t.y += t.vy;
            if is_threat_hitting_planet(t) {
                self.planet_health = (self.planet_health - 10).max(0);
                planet_hit = true;
                return false;
            }
            is_on_screen(t.x, t.y, t.radius + CANVAS_WIDTH * 0.2)
        });

        if planet_hit {
            self.add_screen_shake(6.0);
            if self.planet_health == 0 { self.game_over = true; }
        }
    }

    
   fn update_projectiles(&mut self) {
        // FIXED: Collect explosions first, then apply after retain
        let mut explosions_to_create = Vec::new();
        let mut score_to_add = 0;
        
        self.projectiles.retain_mut(|p| {
            p.x += p.vx; 
            p.y += p.vy;
            
            for t in &mut self.threats {
                if check_collision(p.x, p.y, p.radius, t.x, t.y, t.radius) {
                    t.radius = 0.0; // Mark for removal
                    
                    // FIXED: Add scoring logic here
                    self.combo += 1;
                    self.combo_timer = 2.0; // Reset combo timer
                    let points = 10 * self.combo.min(10); // Max 10x multiplier
                    score_to_add += points;
                    
                    // Store explosion data for later
                    explosions_to_create.push((t.x, t.y, (255, 200, 100), 15));
                    
                    // Debug log
                    web_sys::console::log_1(&format!("Hit! Score: {}, Combo: {}x", points, self.combo).into());
                    
                    return false;
                }
            }
            is_on_screen(p.x, p.y, p.radius + 50.0)
        });
        
        // Apply score after borrows are resolved
        self.score += score_to_add;
        
        // Remove destroyed threats
        self.threats.retain(|t| t.radius > 0.0);
        
        // Create explosions after borrow issues are resolved
        for (x, y, color, count) in explosions_to_create {
            self.create_explosion(x, y, color, count);
        }
    }
    
    fn update_particles(&mut self, delta: f64) {
        self.particles.retain_mut(|particle| {
            particle.x += particle.vx; particle.y += particle.vy;
            particle.vx *= 0.98; particle.vy *= 0.98; 
            particle.lifetime -= delta * 2.0; 
            particle.lifetime > 0.0
        });
    }

    fn update_combo_timer(&mut self, delta: f64) {
        if self.combo_timer > 0.0 {
            self.combo_timer -= delta; 
            if self.combo_timer <= 0.0 { 
                web_sys::console::log_1(&format!("Combo reset from {}x", self.combo).into());
                self.combo = 0; 
                self.combo_timer = 0.0; 
            }
        }
    }

    fn update_screen_shake(&mut self, delta: f64) {
        if self.screen_shake > 0.0 {
            self.screen_shake -= delta * 5.0; 
            if self.screen_shake < 0.0 { self.screen_shake = 0.0; }
        }
    }
}