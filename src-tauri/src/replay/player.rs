// Playback engine for Shadow Replay
// Future implementation: Advanced playback features

pub struct Player {
    pub speed: f32,
    pub loop_enabled: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            speed: 1.0,
            loop_enabled: false,
        }
    }
}
