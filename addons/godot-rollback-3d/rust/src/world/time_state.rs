use crate::types::*;
#[derive(Debug)]
pub struct TimeState {
    pub tick: Tick,
    pub secs: f32,
}

impl Default for TimeState {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeState {
    pub fn new() -> Self {
        Self { tick: 0, secs: 0.0 }
    }
}
