use crate::acid_bass::glide::Glide;
use crate::dsp::envelopes::ad_envelope::ADEnvelope;
use crate::dsp::filters::filter_ladder::LadderFilter;
use crate::dsp::oscillators::Oscillator;
use crate::dsp::oscillators::waveform::Waveform;
use crate::dsp::tune::Tune;
use crate::dsp::types::{Frequency, Sample, SampleRate, Time};

const MAX_ENV_MOD_HZ: Frequency = 4000.0; // starting guess for full-depth env mod sweep, retune by ear

// The monophonic acid voice: one oscillator through one ladder filter into
// one output stage, with two decay-only envelopes (one modulating the
// filter's cutoff -- "Env Mod" -- one modulating output level) and a
// glide unit for Slide steps. AcidSequencer/AcidStep know nothing about
// audio; this is the thing that actually turns "note X, accent on, slide
// on" into a waveform.
pub struct AcidVoice {
    oscillator: Oscillator,
    filter: LadderFilter,
    filter_envelope: ADEnvelope,
    amp_envelope: ADEnvelope,
    glide: Glide,
    tune: Tune,

    // the filter's resting cutoff (the knob position) -- kept separate
    // from filter.cutoff() because next_sample() modulates the filter's
    // actual cutoff every sample (base + envelope contribution); without
    // a separate base value there'd be nothing to modulate *from*
    base_cutoff: Frequency,
    env_mod_amount: f32,
    accent_amount: f32,

    // whether the currently-playing note was triggered with accent on --
    // has to persist for the note's whole duration (not just at trigger
    // time), since it affects every sample of this note's envelope/level
    // output, not just the initial hit
    accented: bool,
}

impl AcidVoice {
    pub fn new(rate: SampleRate) -> Self {
        Self {
            oscillator: Oscillator::new(rate),
            filter: LadderFilter::new(rate),
            filter_envelope: ADEnvelope::new(rate),
            amp_envelope: ADEnvelope::new(rate),
            glide: Glide::new(rate),
            tune: Tune::new(),
            base_cutoff: 2000.0,
            env_mod_amount: 0.0,
            accent_amount: 0.0,
            accented: false,
        }
    }

    // Controls: one method per real-303 panel knob

    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.oscillator.set_waveform(waveform);
    }

    pub fn set_tuning(&mut self, semitones: f32) {
        // the "Tuning" trim knob -- shared with drums::SampleTrack via
        // dsp::tune::Tune; note_on multiplies self.tune.ratio() into the
        // played frequency
        self.tune.set_semitones(semitones);
    }

    pub fn set_cutoff(&mut self, cutoff: Frequency) {
        // sets self.base_cutoff -- NOT self.filter's cutoff directly.
        // next_sample() is what pushes the actual modulated value into
        // the filter each sample; this only moves the resting point that
        // gets modulated from.
        self.base_cutoff = cutoff;
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.filter.set_resonance(resonance);
    }

    pub fn set_env_mod(&mut self, amount: f32) {
        self.env_mod_amount = amount.clamp(0.0, 1.0);
    }

    pub fn set_decay(&mut self, time: Time) {
        // 303s use a single decay knob for both envelopes
        self.filter_envelope.set_decay_time(time);
        self.amp_envelope.set_decay_time(time);
    }

    pub fn set_accent_amount(&mut self, amount: f32) {
        self.accent_amount = amount.clamp(0.0, 1.0);
    }

    pub fn set_glide_time(&mut self, time: Time) {
        self.glide.set_time(time);
    }

    // Playback -- called by whatever drives this voice (an AcidSynth
    // engine, not built yet) each time AcidSequencer crosses into a step
    // with gate = true. No note_off: like the drum machine's envelopes,
    // these are decay-only/one-shot and fade out on their own -- there's
    // nothing to release.
    pub fn note_on(&mut self, frequency: Frequency, accent: bool, slide: bool) {
        self.accented = accent;
        self.glide.set_target(frequency * self.tune.ratio(), slide);

        // always retrigger both envelopes, even on a slide -- a real 303
        // retriggers every step regardless; slide only ever affects pitch
        // continuity, never the envelopes
        self.filter_envelope.trigger();
        self.amp_envelope.trigger();
    }

    pub fn next_sample(&mut self) -> Sample {
        self.oscillator.set_freq(self.glide.next_frequency());

        // accent boosts both the filter sweep depth and the final output
        // level together -- a real 303's accent circuit drives both at
        // once, not just volume
        let accent_boost = if self.accented {
            1.0 + self.accent_amount
        } else {
            1.0
        };

        let filter_env_level = self.filter_envelope.next_sample();
        let cutoff_offset = filter_env_level * self.env_mod_amount * accent_boost * MAX_ENV_MOD_HZ;
        self.filter.set_cutoff(self.base_cutoff + cutoff_offset);

        let filtered = self.filter.process(self.oscillator.next_sample());
        let amp_env_level = self.amp_envelope.next_sample();

        filtered * amp_env_level * accent_boost
    }

    pub fn reset(&mut self) {
        self.oscillator.reset();
        self.filter.reset();
        self.filter_envelope.reset();
        self.amp_envelope.reset();
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

    #[test]
    fn next_sample_is_silent_before_any_trigger() {
        // amp_envelope starts Idle (silent) until note_on triggers it,
        // same as DrumEnvelope -- unlike SamplePlayer, nothing here is
        // "live" from construction
        let mut voice = AcidVoice::new(48_000.0);

        assert_eq!(voice.next_sample(), 0.0);
    }

    #[test]
    fn note_on_makes_the_voice_audible() {
        // tiny sample rate so ADEnvelope's default (near-instant) attack
        // completes within a single sample, same trick used throughout
        // drum_machine's tests. Square waveform sidesteps PhaseGenerator's
        // sine starting at phase 0.0 (== sample 0.0), which would make the
        // very first sample silent regardless of the envelope
        let mut voice = AcidVoice::new(16.0);
        voice.set_waveform(Waveform::Square);

        voice.note_on(440.0, false, false);

        assert_ne!(voice.next_sample(), 0.0);
    }

    #[test]
    fn reset_silences_the_voice_immediately() {
        let mut voice = AcidVoice::new(16.0);
        voice.set_waveform(Waveform::Square);

        voice.note_on(440.0, false, false);
        voice.next_sample();

        voice.reset();

        assert_eq!(voice.next_sample(), 0.0);
    }

    #[test]
    fn retrigger_restarts_the_envelope_even_mid_decay() {
        let mut voice = AcidVoice::new(16.0);
        voice.set_waveform(Waveform::Square);
        voice.set_decay(0.05);

        voice.note_on(440.0, false, false);
        let peak = voice.next_sample().abs();

        for _ in 0..1000 {
            voice.next_sample();
        }
        let decayed = voice.next_sample().abs();

        assert!(decayed < peak, "expected the note to have decayed by now");

        voice.note_on(440.0, false, false);
        let retriggered = voice.next_sample().abs();

        assert!(
            retriggered > decayed,
            "expected retriggering to jump back up: decayed={decayed}, retriggered={retriggered}"
        );
    }

    #[test]
    fn accented_note_is_louder_than_unaccented() {
        // env_mod stays at 0 (the default) so this isolates the accent's
        // effect on final output level from its effect on the filter sweep
        let mut normal = AcidVoice::new(16.0);
        normal.set_waveform(Waveform::Square);
        normal.set_accent_amount(1.0);
        normal.note_on(440.0, false, false);

        let mut accented = AcidVoice::new(16.0);
        accented.set_waveform(Waveform::Square);
        accented.set_accent_amount(1.0);
        accented.note_on(440.0, true, false);

        let normal_output = normal.next_sample();
        let accented_output = accented.next_sample();

        // accent_amount = 1.0 -> accent_boost = 2.0 for the accented voice,
        // 1.0 for the normal one, applied as a straight output multiplier
        assert_approx_eq(accented_output, normal_output * 2.0);
    }

    #[test]
    fn set_env_mod_clamps_above_one() {
        // indirect check, since env_mod_amount has no getter: a clamped
        // 5.0 and an already-in-range 1.0 must behave identically
        let mut clamped = AcidVoice::new(16.0);
        clamped.set_waveform(Waveform::Square);
        clamped.set_env_mod(5.0);
        clamped.note_on(440.0, false, false);

        let mut reference = AcidVoice::new(16.0);
        reference.set_waveform(Waveform::Square);
        reference.set_env_mod(1.0);
        reference.note_on(440.0, false, false);

        assert_approx_eq(clamped.next_sample(), reference.next_sample());
    }

    #[test]
    fn set_env_mod_does_not_panic() {
        // regression test: set_env_mod used to have a stray todo!() left
        // after its real assignment, so calling it always panicked even
        // though the actual clamp-and-store logic above it was correct
        let mut voice = AcidVoice::new(48_000.0);

        voice.set_env_mod(0.5);
        voice.note_on(440.0, false, false);
        voice.next_sample();
    }

    #[test]
    fn output_stays_finite_with_env_mod_and_accent_maxed_out() {
        let mut voice = AcidVoice::new(48_000.0);

        voice.set_waveform(Waveform::Square);
        voice.set_resonance(1.0);
        voice.set_env_mod(1.0);
        voice.set_accent_amount(1.0);
        voice.set_decay(0.01);

        for i in 0..20_000 {
            if i % 500 == 0 {
                voice.note_on(220.0 + i as f32, true, i % 1000 == 0);
            }

            let sample = voice.next_sample();

            assert!(sample.is_finite(), "non-finite output: {sample}");
        }
    }
}
