use super::types::{Frequency, Sample, SampleRate};

pub struct LowPassFilter {
    sample_rate: SampleRate,
    cutoff: Frequency,
    resonance: f32,
    state: Sample,
}

impl LowPassFilter {
    pub fn new(rate: SampleRate) -> Self {
        Self {
            sample_rate: rate,
            cutoff: 2000.0,
            resonance: 0.0,
            state: 0.0,
        }
    }

    pub fn set_cutoff(&mut self, cutoff: Frequency) {
        self.cutoff = cutoff;
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance;
    }

    pub fn cutoff(&self) -> Frequency {
        self.cutoff
    }

    pub fn resonance(&self) -> f32 {
        self.resonance
    }

    pub fn reset(&mut self) {
        self.state = 0.0;
    }

    pub fn process(&mut self, input: Sample) -> Sample {
        let dt = 1.0 / self.sample_rate;
        let rc = 1.0 / (self.cutoff * 2.0 * std::f32::consts::PI);
        let alpha = dt / (rc + dt);
        self.state += alpha * (input - self.state);
        self.state
    }
}
