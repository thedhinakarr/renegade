//! Cross-platform audio façade.
//! Call `audio::play(Sound::Shoot)` or `audio::looped(Sound::Background)`
//! anywhere in the game; the correct backend is chosen at compile time.

use once_cell::sync::Lazy;

/// Logical IDs for every sound used in the game.
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum Sound {
    Shoot,
    Explosion,
    PlanetHit,
    Background,
}

/// A tiny trait the two back-ends must fulfil.
pub trait Backend: 'static + Send + Sync {
    fn preload() -> Self
    where
        Self: Sized;
    fn play(&self, sound: Sound, looped: bool);
    fn set_volume(&self, volume: f32); // master 0.0-1.0
}

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub(crate) type Impl = web::WebAudio;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type Impl = native::RodioBackend;

/// Lazily created singleton so game code never worries about handles.
pub static AUDIO: Lazy<Impl> = Lazy::new(Impl::preload);

// -------------------------------------------------------------------------
// Convenience wrappers used by game code
// -------------------------------------------------------------------------
pub fn play(sound: Sound)           { web_sys::console::log_1(&format!("▶ {:?}", sound).into());AUDIO.play(sound, false); }
pub fn looped(sound: Sound)         { AUDIO.play(sound, true ); }
pub fn set_master(volume: f32)      { AUDIO.set_volume(volume); }
