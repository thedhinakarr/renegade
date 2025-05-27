use crate::game::entities::*;
use crate::constants::*;

pub struct GameState {
    pub player: Player,
    pub threats: Vec<Threat>,
    pub projectiles: Vec<Projectile>,
    pub particles: Vec<Particle>,
    pub score: u32,
    pub planet_health: i32,
    pub time: f64,
    pub game_over: bool,
    pub combo: u32,
    pub combo_timer: f64,
    pub screen_shake: f64,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            player: Player::new(),
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
    
    pub fn create_explosion(&mut self, x: f64, y: f64, color: (u8, u8, u8), count: u32) {
        for _ in 0..count {
            let angle = js_sys::Math::random() * std::f64::consts::PI * 2.0;
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
    
    pub fn add_screen_shake(&mut self, intensity: f64) {
        self.screen_shake = (self.screen_shake + intensity).min(10.0);
    }
    
    pub fn shoot(&mut self) {
        if self.game_over {
            return;
        }
        
        let (player_x, player_y) = self.player.get_position();
        let vx = self.player.angle.cos() * 8.0;
        let vy = self.player.angle.sin() * 8.0;
        
        self.projectiles.push(Projectile {
            x: player_x,
            y: player_y,
            vx,
            vy,
            radius: 5.0,
        });
    }
}