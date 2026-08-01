use crate::dsp::mixer::Mixer;
use crate::dsp::modulation::{Modulation, ModulationTarget};
use crate::dsp::waveform::Waveform;
use crate::params::voice_params::VoiceParams;

use super::envelope::{Envelope, EnvelopeState};
use super::oscillator::Oscillator;
use super::types::{Frequency, Sample, SampleRate};

pub struct Voice {
    osc1: Oscillator,
    osc2: Oscillator,

    envelope: Envelope,

    frequency: Frequency,

    mixer: Mixer,

    modulation: Option<Modulation>,
    params: VoiceParams,
}

impl Voice {
    // Constructor
    pub fn new(rate: SampleRate) -> Self {
        Self {
            osc1: Oscillator::new(rate),
            osc2: Oscillator::new(rate),

            envelope: Envelope::new(rate),

            frequency: 0.0,

            mixer: Mixer::new(),

            modulation: None,
            params: VoiceParams::default(),
        }
    }

    // Modulation
    fn apply_pitch_modulation(&self, frequency: Frequency) -> Frequency {
        match self.modulation {
            Some(modulation) if modulation.target() == ModulationTarget::Pitch => {
                frequency + modulation.value() as f32
            }

            _ => frequency,
        }
    }

    fn apply_vibrato_modulation(&self, frequency: Frequency) -> Frequency {
        match self.modulation {
            Some(modulation) if modulation.target() == ModulationTarget::Vibrato => {
                frequency + modulation.value() as f32
            }

            _ => frequency,
        }
    }

    fn current_frequency(&self) -> Frequency {
        let frequency = self.apply_pitch_modulation(self.frequency);

        self.apply_vibrato_modulation(frequency)
    }

    fn apply_volume_modulation(&self, sample: Sample) -> Sample {
        match self.modulation {
            Some(modulation) if modulation.target() == ModulationTarget::Volume => {
                let amount = modulation.value() as f32 / 255.0;

                sample * amount
            }

            _ => sample,
        }
    }

    fn oscillator_samples(&mut self) -> (Sample, Sample) {
        let frequency = self.current_frequency();

        self.osc1.set_freq(frequency);

        let detuned = frequency * 2.0_f32.powf(self.params.osc2_detune / 1200.0);

        self.osc2.set_freq(detuned);

        let osc1 = self.osc1.next_sample() * self.params.osc1.level;

        let osc2 = self.osc2.next_sample() * self.params.osc2.level;

        (osc1, osc2)
    }

    // public endpoints
    pub fn set_params(&mut self, params: VoiceParams) {
        self.params = params;

        self.osc1.set_waveform(self.params.osc1.waveform);

        self.osc2.set_waveform(self.params.osc2.waveform);
    }

    pub fn params(&self) -> VoiceParams {
        self.params
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

    pub fn apply_modulation(&mut self, modulation: Modulation) {
        self.modulation = Some(modulation);
    }

    pub fn clear_modulation(&mut self) {
        self.modulation = None;
    }

    pub fn modulation(&self) -> Option<Modulation> {
        self.modulation
    }

    pub fn next_sample(&mut self) -> Sample {
        let (osc1, osc2) = self.oscillator_samples();

        self.mixer.reset();

        self.mixer.add(osc1);
        self.mixer.add(osc2);

        let mixed = self.mixer.output();

        let envelope = self.envelope.next_sample();

        self.apply_volume_modulation(mixed * envelope)
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

        let mut found = false;

        for _ in 0..100 {
            if voice.next_sample().abs() > 0.0 {
                found = true;
                break;
            }
        }

        assert!(found, "voice produced no output");
    }

    #[test]
    fn note_off_releases_voice() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

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

        for _ in 0..20_000 {
            voice.next_sample();
        }

        voice.note_off();

        for _ in 0..20_000 {
            voice.next_sample();
        }

        assert_approx_eq(voice.next_sample(), 0.0);

        assert_eq!(voice.envelope.state(), EnvelopeState::Idle);
    }

    #[test]
    fn modulation_is_stored() {
        let mut voice = Voice::new(48_000.0);

        let modulation = Modulation::new(ModulationTarget::Pitch, 128);

        voice.apply_modulation(modulation);

        assert_eq!(voice.modulation(), Some(modulation));
    }

    #[test]
    fn modulation_can_be_cleared() {
        let mut voice = Voice::new(48_000.0);

        voice.apply_modulation(Modulation::new(ModulationTarget::Volume, 100));

        voice.clear_modulation();

        assert_eq!(voice.modulation(), None);
    }

    #[test]
    fn pitch_modulation_changes_frequency() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        voice.apply_modulation(Modulation::new(ModulationTarget::Pitch, 10));

        assert_eq!(voice.current_frequency(), 450.0);
    }

    #[test]
    fn volume_modulation_reduces_output() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        voice.apply_modulation(Modulation::new(ModulationTarget::Volume, 128));

        for _ in 0..100 {
            voice.next_sample();
        }

        let sample = voice.next_sample();

        assert!(sample.abs() <= 1.0);
    }

    #[test]
    fn output_stays_in_range() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        for _ in 0..100_000 {
            let sample = voice.next_sample();

            assert!((-1.0..=1.0).contains(&sample), "out of range: {}", sample);
        }
    }

    #[test]
    fn default_voice_has_two_sine_oscillators() {
        let voice = Voice::new(48_000.0);

        assert_eq!(voice.params.osc1.waveform, Waveform::Sine);

        assert_eq!(voice.params.osc2.waveform, Waveform::Sine);
    }

    #[test]
    fn osc2_detune_changes_frequency() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        let mut params = voice.params();

        params.osc2_detune = 1200.0;

        voice.set_params(params);

        let (_, osc2_freq) = (voice.osc1.freq(), voice.osc2.freq());

        voice.next_sample();

        assert_eq!(voice.osc2.freq(), 880.0);
    }
}
