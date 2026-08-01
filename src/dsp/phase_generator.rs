use super::types::{
    DEFAULT_FREQ, DEFAULT_PHASE, DEFAULT_SAMPLE_RATE, Frequency, Phase, Sample, SampleRate,
};
use super::waveform::Waveform;

pub struct PhaseGenerator {
    waveform: Waveform,
    phase: Phase,
    frequency: Frequency,
    sample_rate: SampleRate,
}

impl PhaseGenerator {
    pub fn new(rate: SampleRate) -> Self {
        Self {
            waveform: Waveform::Sine,
            phase: DEFAULT_PHASE,
            frequency: DEFAULT_FREQ,
            sample_rate: rate,
        }
    }

    // Setters
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn set_phase(&mut self, phase: Phase) {
        self.phase = phase;
    }

    pub fn set_freq(&mut self, freq: Frequency) {
        self.frequency = freq;
    }

    pub fn set_sample_rate(&mut self, rate: SampleRate) {
        self.sample_rate = rate;
    }

    // Getters
    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn freq(&self) -> Frequency {
        self.frequency
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    //helpers

    fn advance_phase(&mut self) {
        self.phase += self.frequency / self.sample_rate;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }

    fn sine(&self) -> Sample {
        (self.phase * std::f32::consts::TAU).sin()
    }

    fn square(&self) -> Sample {
        if self.phase < 0.5 { 1.0 } else { -1.0 }
    }

    fn saw(&self) -> Sample {
        (self.phase * 2.0) - 1.0
    }

    fn triangle(&self) -> Sample {
        if self.phase < 0.5 {
            (self.phase * 4.0) - 1.0
        } else {
            3.0 - (self.phase * 4.0)
        }
    }

    // Move
    pub fn next_sample(&mut self) -> Sample {
        let sample = match self.waveform {
            Waveform::Sine => self.sine(),
            Waveform::Square => self.square(),
            Waveform::Saw => self.saw(),
            Waveform::Triangle => self.triangle(),
        };

        self.advance_phase();

        sample
    }

    //reset
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }
}
