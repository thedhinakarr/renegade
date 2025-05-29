use super::{Backend, Sound};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::HtmlAudioElement;

const PATHS: &[(&str, Sound)] = &[
    ("assets/audio/shoot.wav"     , Sound::Shoot     ),
    ("assets/audio/explosion.wav" , Sound::Explosion ),
    ("assets/audio/planet_hit.wav", Sound::PlanetHit ),
    ("assets/audio/background.mp3", Sound::Background),
];

pub struct WebAudio {
    clips: HashMap<Sound, Vec<HtmlAudioElement>>,
    master: f32,
    pool_size: usize,
}

impl Backend for WebAudio {
    fn preload() -> Self {
        let mut clips = HashMap::new();
        for (path, id) in PATHS {
            let mut vec = Vec::new();
            for _ in 0..4 {                      // 4-voice pool per clip
                let el = HtmlAudioElement::new_with_src(path).unwrap();
                el.set_preload("auto");
                vec.push(el);
            }
            clips.insert(*id, vec);
        }
        Self { clips, master: 1.0, pool_size: 4 }
    }

    fn play(&self, sound: Sound, looped: bool) {
        if let Some(pool) = self.clips.get(&sound) {
            let idx = (js_sys::Math::random() * self.pool_size as f64) as usize;
            let el  = &pool[idx];
            el.set_current_time(0.0);
            el.set_loop(looped);
            el.set_volume(self.master as f64);
            let _ = el.play();                   // ignore the JS promise
        }
    }

    fn set_volume(&self, v: f32) {
        for pool in self.clips.values() {
            for el in pool { el.set_volume(v as f64); }
        }
    }
}
