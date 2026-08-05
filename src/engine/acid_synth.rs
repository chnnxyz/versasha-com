use crate::acid_bass::acid_sequencer::AcidSequencer;
use crate::acid_bass::acid_step::AcidStep;
use crate::acid_bass::acid_voice::AcidVoice;
use crate::dsp::oscillators::waveform::Waveform;
use crate::dsp::types::{Frequency, Sample, SampleRate, Time};
use crate::sequencing::transport::SequencerStatus;

// The top-level acid engine: wires one AcidSequencer (the pattern/clock)
// to one AcidVoice (the actual synthesis chain), the same role
// DrumMachine plays for SampleTrack/Sequencer -- reuses
// sequencing::transport::SequencerStatus rather than redefining an
// identical Play/Pause/Stop enum here.
//
// TODO(session-refactor): once engine::session::Session exists and owns a
// shared sequencing::transport::Transport, sequencer_status below plus
// play()/pause()/stop()/bpm()/set_bpm()/current_step() and the
// advance-and-trigger block at the top of next_sample() all become
// redundant -- see the identical note in drum_machine.rs.
pub struct AcidSynth {
    sequencer_status: SequencerStatus,
    sequencer: AcidSequencer,
    voice: AcidVoice,
    master_volume: f32,
}

impl AcidSynth {
    pub fn new(
        rate: SampleRate,
        num_steps: usize,
        bpm: f32,
        steps_per_beat: Option<usize>,
    ) -> Self {
        Self {
            sequencer_status: SequencerStatus::Stop,
            sequencer: AcidSequencer::new(rate, num_steps, bpm, steps_per_beat),
            voice: AcidVoice::new(rate),
            master_volume: 0.5,
        }
    }

    // getters

    pub fn bpm(&self) -> f32 {
        self.sequencer.bpm()
    }

    pub fn current_step(&self) -> usize {
        self.sequencer.current_step()
    }

    pub fn step_count(&self) -> usize {
        self.sequencer.step_count()
    }

    pub fn sequencer_status(&self) -> SequencerStatus {
        self.sequencer_status
    }

    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    // setters / playback controls

    pub fn set_bpm(&mut self, bpm: f32) {
        self.sequencer.set_bpm(bpm);
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    pub fn play(&mut self) {
        self.sequencer_status = SequencerStatus::Play;
    }

    pub fn pause(&mut self) {
        self.sequencer_status = SequencerStatus::Pause
    }

    pub fn stop(&mut self) {
        self.sequencer_status = SequencerStatus::Stop;
        self.sequencer.stop();
    }

    // pattern editing

    pub fn set_step(&mut self, index: usize, step: AcidStep) {
        if index < self.sequencer.step_count() {
            self.sequencer.set_step(index, step);
        }
    }

    pub fn clear_all_steps(&mut self) {
        self.sequencer.clear_all_steps();
    }

    // voice controls -- one passthrough per AcidVoice knob; no track
    // index needed anywhere here, unlike DrumMachine's per-track setters,
    // since there's only ever the one monophonic voice

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.voice.set_waveform(waveform);
    }

    pub fn set_tuning(&mut self, semitones: f32) {
        self.voice.set_tuning(semitones);
    }

    pub fn set_cutoff(&mut self, cutoff: Frequency) {
        self.voice.set_cutoff(cutoff);
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.voice.set_resonance(resonance);
    }

    pub fn set_env_mod(&mut self, amount: f32) {
        self.voice.set_env_mod(amount);
    }

    pub fn set_decay(&mut self, time: Time) {
        self.voice.set_decay(time);
    }

    pub fn set_accent_amount(&mut self, amount: f32) {
        self.voice.set_accent_amount(amount);
    }

    pub fn set_glide_time(&mut self, time: Time) {
        self.voice.set_glide_time(time);
    }

    // audio generation

    // TODO(session-refactor): the method a shared Transport will call
    // when it crosses a step boundary. Same trigger logic as the one
    // inlined at the top of next_sample() below, reading an
    // externally-supplied step instead of advancing its own clock.
    pub fn trigger_step(&mut self, step: usize) {
        if step < self.sequencer.step_count() {
            let curr_step: &AcidStep = self.sequencer.step(step);
            if curr_step.gate() {
                self.voice
                    .note_on(curr_step.note(), curr_step.accent(), curr_step.slide());
            }
        }
    }

    pub fn next_sample(&mut self) -> Sample {
        if self.sequencer_status == SequencerStatus::Play && self.sequencer.advance() {
            let step: &AcidStep = self.sequencer.active_step();
            if step.gate() {
                self.voice.note_on(step.note(), step.accent(), step.slide());
            }
        }
        let next: Sample = self.voice.next_sample();
        next * self.master_volume
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-3;

    fn assert_approx_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "expected {}, got {}",
            expected,
            actual
        );
    }

    // tiny sample rate + Square waveform, same trick used throughout
    // acid_voice's own tests: ADEnvelope's default attack completes
    // within a single sample here, and Square sidesteps PhaseGenerator's
    // sine starting at phase 0.0 (silent on the very first sample)
    fn test_synth() -> AcidSynth {
        let mut synth = AcidSynth::new(16.0, 4, 120.0, Some(4));
        synth.set_waveform(Waveform::Square);
        synth
    }

    fn gated_step(note: Frequency, accent: bool) -> AcidStep {
        let mut step = AcidStep::new();
        step.set_note(note);
        step.set_gate(true);
        step.set_accent(accent);
        step
    }

    #[test]
    fn new_sets_expected_defaults() {
        let synth = AcidSynth::new(16.0, 4, 120.0, Some(4));

        assert_eq!(synth.sequencer_status(), SequencerStatus::Stop);
        assert_approx_eq(synth.master_volume(), 0.5);
        assert_eq!(synth.step_count(), 4);
        assert_approx_eq(synth.bpm(), 120.0);
    }

    #[test]
    fn trigger_step_fires_a_gated_step() {
        let mut synth = test_synth();

        synth.set_step(0, gated_step(440.0, false));
        synth.trigger_step(0);

        assert_ne!(synth.next_sample(), 0.0);
    }

    #[test]
    fn trigger_step_ignores_a_step_with_gate_off() {
        let mut synth = test_synth();

        // note is set but gate is off -- default AcidStep::new()
        let mut step = AcidStep::new();
        step.set_note(440.0);
        synth.set_step(0, step);

        synth.trigger_step(0);

        assert_eq!(synth.next_sample(), 0.0);
    }

    #[test]
    fn trigger_step_ignores_out_of_range_step() {
        let mut synth = test_synth();

        // must not panic
        synth.trigger_step(999);
    }

    #[test]
    fn trigger_step_does_not_move_current_step() {
        let mut synth = test_synth();

        // reads an externally-supplied step -- it must not advance
        // self.sequencer's own clock as a side effect
        synth.trigger_step(2);

        assert_eq!(synth.current_step(), 0);
    }

    #[test]
    fn trigger_step_passes_accent_through_to_the_voice() {
        // env_mod stays at 0 (the default) so this isolates accent's
        // effect on final output level, same reasoning as
        // acid_voice::tests' own accent-boost-ratio test
        let mut normal = test_synth();
        normal.set_master_volume(1.0);
        normal.set_accent_amount(1.0);
        normal.set_step(0, gated_step(440.0, false));
        normal.trigger_step(0);

        let mut accented = test_synth();
        accented.set_master_volume(1.0);
        accented.set_accent_amount(1.0);
        accented.set_step(0, gated_step(440.0, true));
        accented.trigger_step(0);

        let normal_output = normal.next_sample();
        let accented_output = accented.next_sample();

        // accent_amount = 1.0 -> accent_boost = 2.0 for the accented
        // voice, 1.0 for the normal one
        assert_approx_eq(accented_output, normal_output * 2.0);
    }
}
