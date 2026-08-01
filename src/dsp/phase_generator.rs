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

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-6;

    fn assert_approx_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "expected {}, got {}",
            expected,
            actual
        );
    }

    #[test]
    fn new_initializes_defaults() {
        let generator = PhaseGenerator::new(DEFAULT_SAMPLE_RATE);

        assert_eq!(generator.waveform(), Waveform::Sine);
        assert_eq!(generator.freq(), DEFAULT_FREQ);
        assert_eq!(generator.phase(), DEFAULT_PHASE);
        assert_eq!(generator.sample_rate(), DEFAULT_SAMPLE_RATE);
    }

    #[test]
    fn set_frequency_changes_frequency() {
        let mut generator = PhaseGenerator::new(DEFAULT_SAMPLE_RATE);

        generator.set_freq(220.0);

        assert_eq!(generator.freq(), 220.0);
    }

    #[test]
    fn set_waveform_changes_waveform() {
        let mut generator = PhaseGenerator::new(DEFAULT_SAMPLE_RATE);

        generator.set_waveform(Waveform::Saw);

        assert_eq!(generator.waveform(), Waveform::Saw);
    }

    #[test]
    fn set_sample_rate_changes_sample_rate() {
        let mut generator = PhaseGenerator::new(DEFAULT_SAMPLE_RATE);

        generator.set_sample_rate(44_100.0);

        assert_eq!(generator.sample_rate(), 44_100.0);
    }

    #[test]
    fn reset_returns_phase_to_zero() {
        let mut generator = PhaseGenerator::new(DEFAULT_SAMPLE_RATE);

        generator.set_phase(0.5);

        generator.reset();

        assert_eq!(generator.phase(), 0.0);
    }

    #[test]
    fn phase_advances_correctly() {
        let mut generator = PhaseGenerator::new(4.0);

        generator.set_freq(1.0);

        generator.next_sample();

        assert_approx_eq(generator.phase(), 0.25);

        generator.next_sample();

        assert_approx_eq(generator.phase(), 0.5);

        generator.next_sample();

        assert_approx_eq(generator.phase(), 0.75);
    }

    #[test]
    fn phase_wraps_after_full_cycle() {
        let mut generator = PhaseGenerator::new(4.0);

        generator.set_freq(1.0);

        for _ in 0..4 {
            generator.next_sample();
        }

        assert_approx_eq(generator.phase(), 0.0);
    }

    #[test]
    fn sine_wave_known_points() {
        let mut generator = PhaseGenerator::new(DEFAULT_SAMPLE_RATE);

        generator.phase = 0.0;
        assert_approx_eq(generator.sine(), 0.0);

        generator.phase = 0.25;
        assert_approx_eq(generator.sine(), 1.0);

        generator.phase = 0.5;
        assert_approx_eq(generator.sine(), 0.0);

        generator.phase = 0.75;
        assert_approx_eq(generator.sine(), -1.0);
    }

    #[test]
    fn square_wave_known_points() {
        let mut generator = PhaseGenerator::new(DEFAULT_SAMPLE_RATE);

        generator.phase = 0.0;
        assert_eq!(generator.square(), 1.0);

        generator.phase = 0.49;
        assert_eq!(generator.square(), 1.0);

        generator.phase = 0.5;
        assert_eq!(generator.square(), -1.0);

        generator.phase = 0.99;
        assert_eq!(generator.square(), -1.0);
    }

    #[test]
    fn saw_wave_known_points() {
        let mut generator = PhaseGenerator::new(DEFAULT_SAMPLE_RATE);

        generator.phase = 0.0;
        assert_approx_eq(generator.saw(), -1.0);

        generator.phase = 0.5;
        assert_approx_eq(generator.saw(), 0.0);

        generator.phase = 0.999;
        assert!(generator.saw() > 0.99);
    }

    #[test]
    fn triangle_wave_known_points() {
        let mut generator = PhaseGenerator::new(DEFAULT_SAMPLE_RATE);

        generator.phase = 0.0;
        assert_approx_eq(generator.triangle(), -1.0);

        generator.phase = 0.25;
        assert_approx_eq(generator.triangle(), 0.0);

        generator.phase = 0.5;
        assert_approx_eq(generator.triangle(), 1.0);

        generator.phase = 0.75;
        assert_approx_eq(generator.triangle(), 0.0);
    }

    #[test]
    fn generated_samples_are_in_range() {
        let mut generator = PhaseGenerator::new(DEFAULT_SAMPLE_RATE);

        let waveforms = [
            Waveform::Sine,
            Waveform::Square,
            Waveform::Saw,
            Waveform::Triangle,
        ];

        for waveform in waveforms {
            generator.reset();
            generator.set_waveform(waveform);

            for _ in 0..100_000 {
                let sample = generator.next_sample();

                assert!(
                    (-1.0..=1.0).contains(&sample),
                    "{:?} produced sample {}",
                    waveform,
                    sample
                );
            }
        }
    }
}
