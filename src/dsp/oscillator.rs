use super::types::{
    DEFAULT_FREQ, DEFAULT_PHASE, DEFAULT_SAMPLE_RATE, Frequency, Phase, SampleRate,
};
use super::waveform::Waveform;

pub struct Oscillator {
    waveform: Waveform,
    phase: Phase,
    frequency: Frequency,
    sample_rate: SampleRate,
}

impl Oscillator {
    pub fn new(rate: f32) -> Self {
        Self {
            waveform: Waveform::Sine,
            phase: DEFAULT_PHASE,
            frequency: DEFAULT_FREQ,
            sample_rate: rate,
        }
    }

    // Setters
    pub fn set_freq(&mut self, freq: f32) {
        self.frequency = freq;
    }

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }

    pub fn reset(&mut self) {
        self.phase = DEFAULT_PHASE;
    }

    // Getters
    pub fn freq(&self) -> f32 {
        self.frequency
    }

    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    pub fn phase(&self) -> f32 {
        self.phase
    }

    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    //helpers
    fn advance_phase(&mut self) {
        self.phase += self.frequency / self.sample_rate;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }

    // Oscilltor types
    fn sine(&self) -> f32 {
        (self.phase * std::f32::consts::TAU).sin()
    }

    fn square(&self) -> f32 {
        if self.phase < 0.5 { 1.0 } else { -1.0 }
    }

    fn saw(&self) -> f32 {
        self.phase * 2.0 - 1.0
    }

    fn triangle(&self) -> f32 {
        1.0 - 4.0 * (self.phase - 0.5).abs()
    }

    // Actually oscillate
    pub fn next_sample(&mut self) -> f32 {
        let sample = match self.waveform {
            Waveform::Sine => self.sine(),
            Waveform::Square => self.square(),
            Waveform::Saw => self.saw(),
            Waveform::Triangle => self.triangle(),
        };

        self.advance_phase();

        sample
    }
}

// TESTS
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
        let osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        assert_eq!(osc.waveform(), Waveform::Sine);
        assert_eq!(osc.freq(), DEFAULT_FREQ);
        assert_eq!(osc.phase(), DEFAULT_PHASE);
        assert_eq!(osc.sample_rate(), DEFAULT_SAMPLE_RATE);
    }

    #[test]
    fn set_frequency_changes_frequency() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.set_freq(220.0);

        assert_eq!(osc.freq(), 220.0);
    }

    #[test]
    fn set_waveform_changes_waveform() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.set_waveform(Waveform::Saw);

        assert_eq!(osc.waveform(), Waveform::Saw);
    }

    #[test]
    fn set_sample_rate_changes_sample_rate() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.set_sample_rate(44_100.0);

        assert_eq!(osc.sample_rate(), 44_100.0);
    }

    #[test]
    fn reset_resets_phase_only() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.set_freq(1234.0);
        osc.set_waveform(Waveform::Triangle);

        osc.next_sample();
        osc.next_sample();

        osc.reset();

        assert_eq!(osc.phase(), DEFAULT_PHASE);
        assert_eq!(osc.freq(), 1234.0);
        assert_eq!(osc.waveform(), Waveform::Triangle);
    }

    #[test]
    fn phase_advances_correctly() {
        let mut osc = Oscillator::new(4.0);

        osc.set_freq(1.0);

        osc.next_sample();
        assert_approx_eq(osc.phase(), 0.25);

        osc.next_sample();
        assert_approx_eq(osc.phase(), 0.50);

        osc.next_sample();
        assert_approx_eq(osc.phase(), 0.75);
    }

    #[test]
    fn phase_wraps_after_one_cycle() {
        let mut osc = Oscillator::new(4.0);

        osc.set_freq(1.0);

        for _ in 0..4 {
            osc.next_sample();
        }

        assert_approx_eq(osc.phase(), 0.0);
    }

    #[test]
    fn sine_wave_known_points() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.phase = 0.0;
        assert_approx_eq(osc.sine(), 0.0);

        osc.phase = 0.25;
        assert_approx_eq(osc.sine(), 1.0);

        osc.phase = 0.5;
        assert_approx_eq(osc.sine(), 0.0);

        osc.phase = 0.75;
        assert_approx_eq(osc.sine(), -1.0);
    }

    #[test]
    fn square_wave_known_points() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.phase = 0.0;
        assert_eq!(osc.square(), 1.0);

        osc.phase = 0.49;
        assert_eq!(osc.square(), 1.0);

        osc.phase = 0.5;
        assert_eq!(osc.square(), -1.0);

        osc.phase = 0.99;
        assert_eq!(osc.square(), -1.0);
    }

    #[test]
    fn saw_wave_known_points() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.phase = 0.0;
        assert_approx_eq(osc.saw(), -1.0);

        osc.phase = 0.5;
        assert_approx_eq(osc.saw(), 0.0);

        osc.phase = 0.999;
        assert!(osc.saw() > 0.99);
    }

    #[test]
    fn triangle_wave_known_points() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        osc.phase = 0.0;
        assert_approx_eq(osc.triangle(), -1.0);

        osc.phase = 0.25;
        assert_approx_eq(osc.triangle(), 0.0);

        osc.phase = 0.5;
        assert_approx_eq(osc.triangle(), 1.0);

        osc.phase = 0.75;
        assert_approx_eq(osc.triangle(), 0.0);
    }

    #[test]
    fn samples_are_within_range() {
        let mut osc = Oscillator::new(DEFAULT_SAMPLE_RATE);

        let waveforms = [
            Waveform::Sine,
            Waveform::Square,
            Waveform::Saw,
            Waveform::Triangle,
        ];

        for waveform in waveforms {
            osc.reset();
            osc.set_waveform(waveform);

            for _ in 0..100_000 {
                let sample = osc.next_sample();

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
