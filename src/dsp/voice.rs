use super::envelope::{Envelope, EnvelopeState};
use super::oscillator::Oscillator;
use super::types::{Frequency, Sample, SampleRate};

pub struct Voice {
    osc1: Oscillator,
    osc2: Oscillator,
    envelope: Envelope,
    frequency: Frequency,
}

impl Voice {
    pub fn new(rate: SampleRate) -> Self {
        Self {
            osc1: Oscillator::new(rate),
            osc2: Oscillator::new(rate),
            envelope: Envelope::new(rate),
            frequency: 0.0,
        }
    }

    pub fn note_on(&mut self, freq: Frequency) {
        self.frequency = freq;
        self.osc1.set_freq(freq);
        self.osc2.set_freq(freq);
        self.envelope.note_on();
    }

    pub fn note_off(&mut self) {
        self.envelope.note_off();
    }

    pub fn next_sample(&mut self) -> Sample {
        let osc1 = self.osc1.next_sample();
        let osc2 = self.osc2.next_sample();

        let mixed = (osc1 + osc2) * 0.5;

        let amp = self.envelope.next_sample();

        mixed * amp
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
    fn new_voice_starts_silent() {
        let voice = Voice::new(48_000.0);

        assert_eq!(voice.frequency, 0.0);
        assert_eq!(voice.envelope.level(), 0.0);
        assert_eq!(voice.envelope.state(), EnvelopeState::Idle);
    }

    #[test]
    fn note_on_sets_frequency() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        assert_eq!(voice.frequency, 440.0);
        assert_eq!(voice.osc1.freq(), 440.0);
        assert_eq!(voice.osc2.freq(), 440.0);
    }

    #[test]
    fn note_on_starts_envelope() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        assert_eq!(voice.envelope.state(), EnvelopeState::Attack);
    }

    #[test]
    fn note_on_produces_output() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        let mut found_non_zero = false;

        for _ in 0..100 {
            let sample = voice.next_sample();

            if sample.abs() > 0.0 {
                found_non_zero = true;
                break;
            }
        }

        assert!(
            found_non_zero,
            "voice did not produce any output after note_on"
        );
    }

    #[test]
    fn note_off_releases_voice() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        // Let envelope enter sustain
        for _ in 0..20_000 {
            voice.next_sample();
        }

        voice.note_off();

        assert_eq!(voice.envelope.state(), EnvelopeState::Release);
    }

    #[test]
    fn note_off_eventually_silences_voice() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        // Reach sustain
        for _ in 0..20_000 {
            voice.next_sample();
        }

        voice.note_off();

        // Finish release
        for _ in 0..20_000 {
            voice.next_sample();
        }

        let sample = voice.next_sample();

        assert_approx_eq(sample, 0.0);
        assert_eq!(voice.envelope.state(), EnvelopeState::Idle);
    }

    #[test]
    fn output_stays_in_range() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        for _ in 0..100_000 {
            let sample = voice.next_sample();

            assert!(
                (-1.0..=1.0).contains(&sample),
                "voice produced out of range sample: {}",
                sample
            );
        }
    }
}
