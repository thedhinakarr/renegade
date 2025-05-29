//! src/game/state.rs
use crate::game::entities::{Player, Threat, Projectile, Particle};
use crate::constants::*;
use crate::audio::{self, Sound};
use wasm_bindgen::prelude::*;
use web_sys::console;

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
    pub threat_spawn_timer: f64,
}

impl GameState {
    pub fn new() -> Self {
        Self {
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
            threat_spawn_timer: 2.0,
        }
    }

    // ───────────────── helper that now triggers sound ───────────────── //

    pub fn shoot(&mut self) {
        if self.game_over { return; }

        let (x, y) = self.player.get_position();
        let speed  = 8.0;
        self.projectiles.push(Projectile {
            x,
            y,
            vx: self.player.angle.cos() * speed,
            vy: self.player.angle.sin() * speed,
            radius: 5.0,
        });

        console::log_1(&format!("Shot fired! Projectiles: {}", self.projectiles.len()).into());
        audio::play(Sound::Shoot);
    }

    pub fn create_explosion(&mut self, x: f64, y: f64,
                             color: (u8, u8, u8), count: u32) {
        for _ in 0..count {
            let ang   = js_sys::Math::random() * std::f64::consts::TAU;
            let speed = js_sys::Math::random() * 3.0 + 1.5;
            self.particles.push(Particle {
                x,
                y,
                vx: ang.cos() * speed,
                vy: ang.sin() * speed,
                size: js_sys::Math::random() * 3.0 + 1.0,
                lifetime: 1.0,
                max_lifetime: 1.0,
                color,
            });
        }
        audio::play(Sound::Explosion);
    }

    pub fn add_screen_shake(&mut self, intensity: f64) {
        self.screen_shake = (self.screen_shake + intensity).min(15.0);
        if intensity > 5.0 {
            audio::play(Sound::PlanetHit);
        }
    }

    // REMOVED the duplicate update_player function from here
    // It's now only in systems.rs
}