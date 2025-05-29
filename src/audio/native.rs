use super::{Backend, Sound};
use rodio::{OutputStream, OutputStreamHandle, Decoder, Sink};
use std::{collections::HashMap, fs::File, io::BufReader, sync::Arc};

const PATHS: &[(&str, Sound)] = &[
    ("assets/audio/shoot.wav"     , Sound::Shoot     ),
    ("assets/audio/explosion.wav" , Sound::Explosion ),
    ("assets/audio/planet_hit.wav", Sound::PlanetHit ),
    ("assets/audio/background.mp3", Sound::Background),
];

pub struct RodioBackend {
    _stream: OutputStream,                 // kept so the stream lives
    handle : OutputStreamHandle,
    sinks  : HashMap<Sound, Arc<Sink>>,
    master : f32,
}

impl Backend for RodioBackend {
    fn preload() -> Self {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let mut sinks = HashMap::new();

        for (path, id) in PATHS {
            let file   = BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap().buffered();
            let sink   = Sink::try_new(&handle).unwrap();
            sink.append(source);
            sink.pause();                           // start muted
            sinks.insert(*id, Arc::new(sink));
        }

        Self { _stream, handle, sinks, master: 1.0 }
    }

    fn play(&self, sound: Sound, looped: bool) {
        if let Some(sink) = self.sinks.get(&sound) {
            sink.stop();                            // rewind
            sink.set_loop(looped);
            sink.set_volume(self.master);
            sink.play();
        }
    }

    fn set_volume(&self, v: f32) {
        self.master = v;
        for sink in self.sinks.values() { sink.set_volume(v); }
    }
}
