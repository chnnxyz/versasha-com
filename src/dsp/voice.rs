use crate::dsp::envelope::EnvelopeState;
use crate::dsp::filter::{Filter, FilterType};
use crate::dsp::fx::FxRoute;
use crate::dsp::fx::delay::Delay;
use crate::dsp::mixer::Mixer;
use crate::dsp::modulation::ModulationTarget;
use crate::dsp::modulation_matrix::ModulationMatrix;
use crate::params::voice_params::VoiceParams;

use super::envelope::Envelope;
use super::oscillator::Oscillator;
use super::types::{DryWet, Frequency, Sample, SampleRate, Time};

pub struct Voice {
    osc1: Oscillator,
    osc2: Oscillator,

    envelope: Envelope,

    frequency: Frequency,

    active: bool,

    mixer: Mixer,

    modulation_matrix: ModulationMatrix,

    params: VoiceParams,

    filter: Filter,

    delay: Delay,

    delay_route: FxRoute,
}

impl Voice {
    pub fn new(rate: SampleRate) -> Self {
        Self {
            osc1: Oscillator::new(rate),

            osc2: Oscillator::new(rate),

            envelope: Envelope::new(rate),

            frequency: 0.0,

            active: false,

            mixer: Mixer::new(),

            modulation_matrix: ModulationMatrix::new(),

            params: VoiceParams::default(),

            filter: Filter::new(rate),

            delay: Delay::new(rate),

            delay_route: FxRoute::Master,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn frequency(&self) -> Frequency {
        self.frequency
    }

    pub fn envelope_state(&self) -> EnvelopeState {
        self.envelope.state()
    }

    // =========================
    // Modulation
    // =========================
    pub fn modulation_matrix_mut(&mut self) -> &mut ModulationMatrix {
        &mut self.modulation_matrix
    }

    pub fn reset_modulation(&mut self) {
        self.modulation_matrix.reset();
    }

    pub fn modulation_value(&self, target: ModulationTarget) -> Sample {
        self.modulation_matrix.value(target)
    }

    fn current_frequency(&self) -> Frequency {
        self.frequency + self.pitch_offset() + self.vibrato_offset()
    }

    fn volume_multiplier(&self) -> Sample {
        let modulation = self.modulation_value(ModulationTarget::Volume);

        if modulation == 0.0 {
            1.0
        } else {
            modulation.clamp(0.0, 1.0)
        }
    }

    fn apply_volume_modulation(&self, sample: Sample) -> Sample {
        sample * self.volume_multiplier()
    }

    fn pitch_offset(&self) -> Frequency {
        let semitones = self.modulation_value(ModulationTarget::Pitch);

        self.frequency * (2.0_f32.powf(semitones / 12.0) - 1.0)
    }

    fn vibrato_offset(&self) -> Frequency {
        let amount = self.modulation_value(ModulationTarget::Vibrato);

        amount
    }
    // =========================
    // Oscillators
    // =========================

    fn oscillator_samples(&mut self) -> (Sample, Sample) {
        let frequency = self.current_frequency();

        self.osc1.set_freq(frequency);

        let osc2_frequency = frequency * 2.0_f32.powf(self.params.osc2_detune / 1200.0);

        self.osc2.set_freq(osc2_frequency);

        let mut osc1 = self.osc1.next_sample() * self.params.osc1.level;

        let mut osc2 = self.osc2.next_sample() * self.params.osc2.level;

        match self.delay_route {
            FxRoute::Osc1 => osc1 = self.delay.process(osc1),
            FxRoute::Osc2 => osc2 = self.delay.process(osc2),
            FxRoute::Master => {}
        }

        (osc1, osc2)
    }

    fn reset_oscillators(&mut self) {
        self.osc1.reset_phase();

        self.osc2.reset_phase();
    }

    // =========================
    // Filter
    // =========================

    fn filter_cutoff_with_envelope(&self, envelope: Sample) -> Frequency {
        let base = self.filter.cutoff();

        let envelope_amount = self.params.envelope.filter_amount;

        let modulation = self.modulation_value(ModulationTarget::FilterCutoff);

        base + (base * envelope_amount * envelope) + modulation
    }

    pub fn set_filter_cutoff(&mut self, cutoff: Frequency) {
        self.filter.set_cutoff(cutoff);
    }

    pub fn filter_cutoff(&self) -> Frequency {
        self.filter.cutoff()
    }

    pub fn set_filter_resonance(&mut self, resonance: f32) {
        self.filter.set_resonance(resonance);
    }

    pub fn filter_resonance(&self) -> f32 {
        self.filter.resonance()
    }

    pub fn set_filter_type(&mut self, filter_type: FilterType) {
        self.filter.set_filter_type(filter_type);
    }

    pub fn filter_type(&self) -> FilterType {
        self.filter.filter_type()
    }

    // =========================
    // Delay
    // =========================

    pub fn set_delay_route(&mut self, route: FxRoute) {
        self.delay_route = route;
    }

    pub fn delay_route(&self) -> FxRoute {
        self.delay_route
    }

    pub fn set_delay_time(&mut self, time: Time) {
        self.delay.set_time(time);
    }

    pub fn delay_time(&self) -> Time {
        self.delay.time()
    }

    pub fn set_delay_feedback(&mut self, feedback: Sample) {
        self.delay.set_feedback(feedback);
    }

    pub fn delay_feedback(&self) -> Sample {
        self.delay.feedback()
    }

    pub fn set_delay_mix(&mut self, mix: DryWet) {
        self.delay.set_mix(mix);
    }

    pub fn delay_mix(&self) -> DryWet {
        self.delay.mix()
    }

    pub fn reset_delay(&mut self) {
        self.delay.reset();
    }

    // =========================
    // Parameters
    // =========================

    pub fn set_params(&mut self, params: VoiceParams) {
        self.params = params;

        self.osc1.set_waveform(self.params.osc1.waveform);

        self.osc2.set_waveform(self.params.osc2.waveform);
    }

    pub fn params(&self) -> VoiceParams {
        self.params
    }

    // =========================
    // Notes
    // =========================

    pub fn note_on(&mut self, freq: Frequency) {
        self.active = true;

        self.frequency = freq;

        self.reset_oscillators();

        self.osc1.set_freq(freq);

        self.osc2.set_freq(freq);

        self.envelope.note_on();
    }

    pub fn note_off(&mut self) {
        self.envelope.note_off();
    }

    // =========================
    // Audio
    // =========================

    pub fn next_sample(&mut self) -> Sample {
        let (osc1, osc2) = self.oscillator_samples();

        self.mixer.reset();
        self.mixer.add(osc1);
        self.mixer.add(osc2);

        let mixed = self.mixer.output();
        let envelope = self.envelope.next_sample();

        if self.envelope.state() == EnvelopeState::Idle {
            self.active = false;
        }

        let cutoff = self.filter_cutoff_with_envelope(envelope);
        self.filter.set_cutoff(cutoff);

        let filtered = self.filter.process(mixed);

        self.apply_volume_modulation(filtered * envelope)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use crate::dsp::envelope::EnvelopeState;
    use crate::dsp::lfo::Lfo;
    use crate::dsp::modulation::{
        Modulation, ModulationGenerator, ModulationSource, ModulationTarget,
    };

    const EPSILON: f32 = 1e-3;

    fn assert_approx_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "expected {}, got {}",
            expected,
            actual
        );
    }

    fn push_modulation(voice: &mut Voice, target: ModulationTarget, value: Sample) {
        voice
            .modulation_matrix_mut()
            .push(Modulation::new(ModulationSource::Lfo, target, value));
    }

    #[test]
    fn new_voice_starts_silent() {
        let voice = Voice::new(48_000.0);

        assert_eq!(voice.frequency, 0.0);
        assert_eq!(voice.envelope.level(), 0.0);
        assert_eq!(voice.envelope.state(), EnvelopeState::Idle);
    }

    #[test]
    fn new_voice_is_inactive() {
        let voice = Voice::new(48_000.0);

        assert!(!voice.is_active());
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
    fn note_on_activates_voice() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        assert!(voice.is_active());
    }

    #[test]
    fn note_on_starts_envelope() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        assert_eq!(voice.envelope.state(), EnvelopeState::Attack);
    }

    #[test]
    fn note_on_resets_phases() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        voice.next_sample();
        voice.next_sample();

        assert_ne!(voice.osc1.phase(), 0.0);

        voice.note_on(220.0);

        assert_eq!(voice.osc1.phase(), 0.0);
        assert_eq!(voice.osc2.phase(), 0.0);
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
    fn modulation_value_can_be_pushed() {
        let mut voice = Voice::new(48_000.0);

        push_modulation(&mut voice, ModulationTarget::Pitch, 2.0);

        assert_eq!(voice.modulation_value(ModulationTarget::Pitch), 2.0);
    }

    #[test]
    fn modulation_values_stack() {
        let mut voice = Voice::new(48_000.0);

        push_modulation(&mut voice, ModulationTarget::Pitch, 5.0);

        push_modulation(&mut voice, ModulationTarget::Pitch, 7.0);

        assert_eq!(voice.modulation_value(ModulationTarget::Pitch), 12.0);
    }

    #[test]
    fn pitch_modulation_changes_frequency() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        push_modulation(&mut voice, ModulationTarget::Pitch, 10.0);

        assert_approx_eq(voice.current_frequency(), 783.99);
    }

    #[test]
    fn pitch_octave_doubles_frequency() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        push_modulation(&mut voice, ModulationTarget::Pitch, 12.0);

        assert_approx_eq(voice.current_frequency(), 880.0);
    }

    #[test]
    fn vibrato_modulation_changes_frequency() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        push_modulation(&mut voice, ModulationTarget::Vibrato, -5.0);

        assert_eq!(voice.current_frequency(), 435.0);
    }

    #[test]
    fn volume_without_modulation_returns_one() {
        let voice = Voice::new(48_000.0);

        assert_eq!(voice.volume_multiplier(), 1.0);
    }

    #[test]
    fn volume_modulation_is_multiplier() {
        let mut voice = Voice::new(48_000.0);

        push_modulation(&mut voice, ModulationTarget::Volume, 0.5);

        assert_eq!(voice.volume_multiplier(), 0.5);
    }

    #[test]
    fn volume_modulation_is_clamped() {
        let mut voice = Voice::new(48_000.0);

        push_modulation(&mut voice, ModulationTarget::Volume, 2.0);

        assert_eq!(voice.volume_multiplier(), 1.0);
    }

    #[test]
    fn reset_modulation_clears_values() {
        let mut voice = Voice::new(48_000.0);

        push_modulation(&mut voice, ModulationTarget::Pitch, 10.0);

        assert_eq!(voice.modulation_value(ModulationTarget::Pitch), 10.0);

        voice.reset_modulation();

        assert_eq!(voice.modulation_value(ModulationTarget::Pitch), 0.0);
    }

    #[test]
    fn filter_cutoff_can_be_changed() {
        let mut voice = Voice::new(48_000.0);

        voice.set_filter_cutoff(800.0);

        assert_eq!(voice.filter_cutoff(), 800.0);
    }

    #[test]
    fn filter_resonance_can_be_changed() {
        let mut voice = Voice::new(48_000.0);

        voice.set_filter_resonance(0.6);

        assert_eq!(voice.filter_resonance(), 0.6);
    }

    #[test]
    fn filter_type_can_be_changed() {
        let mut voice = Voice::new(48_000.0);

        voice.set_filter_type(FilterType::HighPass);

        assert_eq!(voice.filter_type(), FilterType::HighPass);
    }

    #[test]
    fn delay_defaults_to_master_route() {
        let voice = Voice::new(48_000.0);

        assert_eq!(voice.delay_route(), FxRoute::Master);
    }

    #[test]
    fn delay_route_can_be_changed() {
        let mut voice = Voice::new(48_000.0);

        voice.set_delay_route(FxRoute::Osc1);

        assert_eq!(voice.delay_route(), FxRoute::Osc1);
    }

    #[test]
    fn delay_params_round_trip() {
        let mut voice = Voice::new(48_000.0);

        voice.set_delay_time(0.4);
        voice.set_delay_feedback(0.6);
        voice.set_delay_mix(0.7);

        assert_eq!(voice.delay_time(), 0.4);
        assert_eq!(voice.delay_feedback(), 0.6);
        assert_eq!(voice.delay_mix(), 0.7);
    }

    #[test]
    fn master_route_does_not_apply_delay_pre_mix() {
        let mut osc1_voice = Voice::new(48_000.0);
        let mut master_voice = Voice::new(48_000.0);

        for voice in [&mut osc1_voice, &mut master_voice] {
            voice.set_delay_time(0.4);
            voice.set_delay_feedback(0.9);
            voice.set_delay_mix(1.0);
            voice.note_on(440.0);
        }

        osc1_voice.set_delay_route(FxRoute::Osc1);
        master_voice.set_delay_route(FxRoute::Master);

        // discard the first sample: phase 0 produces silence for a sine wave,
        // which would make both routes look identical before the delay has anything to echo
        osc1_voice.oscillator_samples();
        master_voice.oscillator_samples();

        assert_ne!(
            osc1_voice.oscillator_samples(),
            master_voice.oscillator_samples()
        );
    }

    #[test]
    fn filter_envelope_changes_cutoff() {
        let mut voice = Voice::new(48_000.0);

        voice.set_filter_cutoff(1000.0);

        let mut params = voice.params();

        params.envelope.filter_amount = 1.0;

        voice.set_params(params);

        assert_eq!(voice.filter_cutoff_with_envelope(1.0), 2000.0);
    }

    #[test]
    fn filter_modulation_changes_cutoff() {
        let mut voice = Voice::new(48_000.0);

        voice.set_filter_cutoff(1000.0);

        push_modulation(&mut voice, ModulationTarget::FilterCutoff, 250.0);

        assert_eq!(voice.filter_cutoff_with_envelope(0.0), 1250.0);
    }

    #[test]
    fn osc2_detune_changes_frequency() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        let mut params = voice.params();

        params.osc2_detune = 1200.0;

        voice.set_params(params);

        voice.next_sample();

        assert_eq!(voice.osc2.freq(), 880.0);
    }

    #[test]
    fn output_stays_in_range() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        for _ in 0..100_000 {
            let sample = voice.next_sample();

            assert!((-1.0..=1.0).contains(&sample), "out of range {}", sample);
        }
    }

    #[test]
    fn lfo_modulation_can_be_pushed_into_voice() {
        let mut voice = Voice::new(48_000.0);

        let mut lfo = Lfo::new(48_000.0);

        lfo.set_target(ModulationTarget::Pitch);

        let modulation = lfo.next_modulation();

        voice.modulation_matrix_mut().push(modulation);

        let value = voice.modulation_value(ModulationTarget::Pitch);

        assert!((-1.0..=1.0).contains(&value));
    }

    #[test]
    fn lfo_pitch_modulation_changes_frequency() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        let mut lfo = Lfo::new(48_000.0);

        lfo.set_target(ModulationTarget::Pitch);
        lfo.set_amount(12.0);

        for _ in 0..100 {
            voice.reset_modulation();

            let modulation = lfo.next_modulation();

            voice.modulation_matrix_mut().push(modulation);

            if voice.modulation_value(ModulationTarget::Pitch) != 0.0 {
                assert_ne!(voice.current_frequency(), 440.0);

                return;
            }
        }

        panic!("LFO never produced modulation");
    }

    #[test]
    fn lfo_pitch_modulation_moves_frequency_range() {
        let mut voice = Voice::new(48_000.0);

        voice.note_on(440.0);

        let mut lfo = Lfo::new(48_000.0);

        lfo.set_target(ModulationTarget::Pitch);
        lfo.set_amount(12.0);

        let mut min = f32::MAX;
        let mut max = f32::MIN;

        for _ in 0..20_000 {
            voice.reset_modulation();

            let modulation = lfo.next_modulation();

            voice.modulation_matrix_mut().push(modulation);

            let frequency = voice.current_frequency();

            min = min.min(frequency);
            max = max.max(frequency);
        }

        assert!(min < 440.0, "LFO never lowered pitch, min={}", min);

        assert!(max > 440.0, "LFO never raised pitch, max={}", max);
    }
}
