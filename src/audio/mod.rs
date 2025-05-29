//! Audio system that tries assets first, falls back to Web Audio beeps

use wasm_bindgen::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

static AUDIO_CONTEXT_STARTED: AtomicBool = AtomicBool::new(false);
static BACKGROUND_MUSIC_PLAYING: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub enum Sound {
    Shoot,
    Explosion,
    PlanetHit,
    Background,
}

/// Initialize audio context on first user interaction
pub fn init_audio_context() {
    if !AUDIO_CONTEXT_STARTED.load(Ordering::Relaxed) {
        if let Ok(context) = web_sys::AudioContext::new() {
            if let Err(_) = context.resume() {
                web_sys::console::log_1(&"AudioContext resume failed".into());
            } else {
                AUDIO_CONTEXT_STARTED.store(true, Ordering::Relaxed);
                web_sys::console::log_1(&"🎵 AudioContext started successfully!".into());
            }
        }
    }
}

/// Start synthwave background music
pub fn start_background_music() {
    if BACKGROUND_MUSIC_PLAYING.load(Ordering::Relaxed) {
        return; // Already playing
    }
    
    init_audio_context();
    
    if let Ok(context) = web_sys::AudioContext::new() {
        let _ = context.resume();
        
        // Create bass line - dark, moody low frequencies
        if let Ok(bass_osc) = context.create_oscillator() {
            if let Ok(bass_gain) = context.create_gain() {
                if let Ok(bass_filter) = context.create_biquad_filter() {
                    bass_osc.set_type(web_sys::OscillatorType::Sawtooth);
                    bass_osc.frequency().set_value(55.0); // A1 - deep bass
                    
                    // Dark filter for bass
                    bass_filter.set_type(web_sys::BiquadFilterType::Lowpass);
                    bass_filter.frequency().set_value(200.0);
                    bass_filter.q().set_value(0.7);
                    
                    bass_gain.gain().set_value(0.15); // Subtle bass
                    
                    let _ = bass_osc.connect_with_audio_node(&bass_filter);
                    let _ = bass_filter.connect_with_audio_node(&bass_gain);
                    let _ = bass_gain.connect_with_audio_node(&context.destination());
                    let _ = bass_osc.start();
                }
            }
        }
        
        // Create lead synth - Kavinsky-style arpeggiated melody
        if let Ok(lead_osc) = context.create_oscillator() {
            if let Ok(lead_gain) = context.create_gain() {
                lead_osc.set_type(web_sys::OscillatorType::Square);
                
                // Slow, atmospheric melody 
                let now = context.current_time();
                
                // Am chord progression (A-C-E-A pattern)
                lead_osc.frequency().set_value_at_time(220.0, now).unwrap();           // A3
                lead_osc.frequency().set_value_at_time(261.6, now + 2.0).unwrap();    // C4  
                lead_osc.frequency().set_value_at_time(329.6, now + 4.0).unwrap();    // E4
                lead_osc.frequency().set_value_at_time(440.0, now + 6.0).unwrap();    // A4
                lead_osc.frequency().set_value_at_time(220.0, now + 8.0).unwrap();    // Back to A3
                
                // Atmospheric envelope - slow fade in/out
                lead_gain.gain().set_value_at_time(0.0, now).unwrap();
                lead_gain.gain().linear_ramp_to_value_at_time(0.08, now + 1.0).unwrap();
                lead_gain.gain().set_value_at_time(0.08, now + 7.0).unwrap();
                lead_gain.gain().linear_ramp_to_value_at_time(0.0, now + 8.0).unwrap();
                
                let _ = lead_osc.connect_with_audio_node(&lead_gain);
                let _ = lead_gain.connect_with_audio_node(&context.destination());
                let _ = lead_osc.start();
                let _ = lead_osc.stop_with_when(now + 8.0);
            }
        }
        
        // Create pad/atmosphere - wide, dreamy sound
        if let Ok(pad_osc) = context.create_oscillator() {
            if let Ok(pad_gain) = context.create_gain() {
                if let Ok(pad_filter) = context.create_biquad_filter() {
                    pad_osc.set_type(web_sys::OscillatorType::Triangle);
                    pad_osc.frequency().set_value(110.0); // A2 - middle register
                    
                    // Soft filter for dreamy effect
                    pad_filter.set_type(web_sys::BiquadFilterType::Lowpass);
                    pad_filter.frequency().set_value(800.0);
                    pad_filter.q().set_value(0.5);
                    
                    let now = context.current_time();
                    
                    // Very slow fade in for atmosphere
                    pad_gain.gain().set_value_at_time(0.0, now).unwrap();
                    pad_gain.gain().linear_ramp_to_value_at_time(0.05, now + 4.0).unwrap();
                    
                    let _ = pad_osc.connect_with_audio_node(&pad_filter);
                    let _ = pad_filter.connect_with_audio_node(&pad_gain);
                    let _ = pad_gain.connect_with_audio_node(&context.destination());
                    let _ = pad_osc.start();
                }
            }
        }
        
        BACKGROUND_MUSIC_PLAYING.store(true, Ordering::Relaxed);
        web_sys::console::log_1(&"🎵 Synthwave background music started!".into());
        
        // Schedule restart after 8 seconds for looping
        let restart_callback = Closure::wrap(Box::new(move || {
            BACKGROUND_MUSIC_PLAYING.store(false, Ordering::Relaxed);
            // Recursive restart with a slight delay
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    &js_sys::Function::new_no_args("window.restartBackgroundMusic()"),
                    500, // 0.5 second gap
                )
                .unwrap();
        }) as Box<dyn FnMut()>);
        
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                restart_callback.as_ref().unchecked_ref(),
                8000, // 8 seconds
            )
            .unwrap();
        
        restart_callback.forget();
    }
}

/// Play audio using assets first, fallback to beeps
pub fn play(sound: Sound) {
    // Try to init audio context first
    init_audio_context();
    
    let path = match sound {
        Sound::Shoot => "audio/shoot.wav",           
        Sound::Explosion => "audio/explosion.wav",   
        Sound::PlanetHit => "audio/planet_hit.wav",  
        Sound::Background => "audio/background.mp3", 
    };
    
    web_sys::console::log_1(&format!("🔊 Playing: {:?} from {}", sound, path).into());
    
    // Try to create HTML Audio element
    match web_sys::HtmlAudioElement::new_with_src(path) {
        Ok(audio) => {
            audio.set_volume(0.3);
            
            if matches!(sound, Sound::Background) {
                audio.set_loop(true);
            }
            
            // Set up error handler to fallback to beeps
            let sound_for_error = sound;
            let error_callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
                web_sys::console::log_1(&format!("❌ Audio file failed, using beep for {:?}", sound_for_error).into());
                play_beep(sound_for_error);
            }) as Box<dyn FnMut(_)>);
            
            audio.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
            error_callback.forget();
            
            let _ = audio.play();
        }
        Err(_) => {
            web_sys::console::log_1(&format!("❌ Failed to create audio element, using beep for {:?}", sound).into());
            play_beep(sound);
        }
    }
}

/// Fallback beep system using Web Audio API
fn play_beep(sound: Sound) {
    if let Ok(context) = web_sys::AudioContext::new() {
        // Try to resume context if suspended
        let _ = context.resume();
        
        match sound {
            Sound::Shoot => {
                // Create a laser "pew" sound with frequency sweep
                if let Ok(osc) = context.create_oscillator() {
                    if let Ok(gain) = context.create_gain() {
                        osc.set_type(web_sys::OscillatorType::Square);
                        
                        let now = context.current_time();
                        
                        // Laser sweep: start high, drop quickly
                        osc.frequency().set_value_at_time(1200.0, now).unwrap();
                        osc.frequency().exponential_ramp_to_value_at_time(300.0, now + 0.15).unwrap();
                        
                        // Quick attack, fast decay for "pew" effect
                        gain.gain().set_value_at_time(0.0, now).unwrap();
                        gain.gain().linear_ramp_to_value_at_time(0.2, now + 0.01).unwrap();
                        gain.gain().exponential_ramp_to_value_at_time(0.001, now + 0.15).unwrap();
                        
                        let _ = osc.connect_with_audio_node(&gain);
                        let _ = gain.connect_with_audio_node(&context.destination());
                        let _ = osc.start_with_when(now);
                        let _ = osc.stop_with_when(now + 0.15);
                    }
                }
            }
            _ => {
                // Other sounds use simple beeps
                if let Ok(osc) = context.create_oscillator() {
                    if let Ok(gain) = context.create_gain() {
                        let freq = match sound {
                            Sound::Explosion => 150.0,
                            Sound::PlanetHit => 100.0,
                            Sound::Background => 440.0,
                            _ => 400.0,
                        };
                        
                        osc.set_type(web_sys::OscillatorType::Square);
                        osc.frequency().set_value(freq);
                        gain.gain().set_value(0.1);
                        
                        let _ = osc.connect_with_audio_node(&gain);
                        let _ = gain.connect_with_audio_node(&context.destination());
                        let _ = osc.start();
                        let _ = osc.stop_with_when(context.current_time() + 0.2);
                    }
                }
            }
        }
    }
}

pub fn looped(sound: Sound) {
    init_audio_context();
    
    match sound {
        Sound::Background => {
            start_background_music();
            return;
        }
        _ => {}
    }
    
    let path = match sound {
        Sound::Shoot => "audio/shoot.wav",           
        Sound::Explosion => "audio/explosion.wav",   
        Sound::PlanetHit => "audio/planet_hit.wav",  
        Sound::Background => "audio/background.mp3", 
    };
    
    web_sys::console::log_1(&format!("🔁 Looping: {:?} from {}", sound, path).into());
    
    match web_sys::HtmlAudioElement::new_with_src(path) {
        Ok(audio) => {
            audio.set_volume(0.2);
            audio.set_loop(true);
            
            // Fallback for background music
            let error_callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
                web_sys::console::log_1(&"❌ Background music failed, using generated music".into());
                start_background_music();
            }) as Box<dyn FnMut(_)>);
            
            audio.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
            error_callback.forget();
            
            let _ = audio.play();
        }
        Err(_) => {
            web_sys::console::log_1(&"❌ Failed to create background audio, using generated music".into());
            start_background_music();
        }
    }
}

pub fn set_master(volume: f32) {
    web_sys::console::log_1(&format!("🔉 Master volume: {}", volume).into());
}