use crate::dsp::fx::delay::Delay;
use crate::dsp::fx::flanger::Flanger;
use crate::dsp::fx::reverb::Reverb;
use crate::dsp::types::{DryWet, Sample, SampleRate};
use crate::sequencing::note_division::NoteDivision;

const DEFAULT_DRY_WET: DryWet = 0.5;
const DEFAULT_BPM: f32 = 120.0;

// Pioneer DJM-style Beat FX: exactly one effect live at a time, its
// rate synced to the shared BPM, a single Dry/Wet knob, and an on/off
// toggle -- not three independent effects each with their own mix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    Delay,
    Reverb,
    Flanger,
}

// One effects send for the whole mixer. Delay/Reverb/Flanger are all
// constructed up front and never allocated mid-stream -- same reasoning
// as MixerEngine pre-building every InstrumentChannel in engine/
// mixer_engine.rs -- and only whichever one `active` points at actually
// reaches the output; `enabled` is the Pioneer-style on/off toggle on
// top of that. `channel` is which MixerEngine channel index (see
// engine/session.rs's DRUM_CHANNEL/ACID_CHANNEL/SYNTH_CHANNEL) this
// unit is currently inserted on.
pub struct EffectsUnit {
    delay: Delay,
    reverb: Reverb,
    flanger: Flanger,

    active: EffectType,
    enabled: bool,
    division: NoteDivision,
    dry_wet: DryWet,
    channel: usize,
    bpm: f32,
}

impl EffectsUnit {
    pub fn new(sample_rate: SampleRate) -> Self {
        let mut unit = Self {
            delay: Delay::new(sample_rate),
            reverb: Reverb::new(sample_rate),
            flanger: Flanger::new(sample_rate),

            active: EffectType::Delay,
            enabled: false,
            division: NoteDivision::Quarter,
            dry_wet: DEFAULT_DRY_WET,
            channel: 0,
            bpm: DEFAULT_BPM,
        };

        // push the real defaults through set_dry_wet/apply_division
        // rather than hand-setting delay/reverb/flanger's own mix and
        // rate fields separately, so there's exactly one place that
        // knows how (division, bpm, dry_wet) map onto the three effects
        unit.set_dry_wet(DEFAULT_DRY_WET);
        unit.apply_division();

        unit
    }

    // getters
    pub fn active(&self) -> EffectType {
        self.active
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn division(&self) -> NoteDivision {
        self.division
    }

    pub fn dry_wet(&self) -> DryWet {
        self.dry_wet
    }

    pub fn channel(&self) -> usize {
        self.channel
    }

    // setters
    pub fn set_active(&mut self, effect: EffectType) {
        self.active = effect;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    // re-derives whichever active effect's own time/rate parameter from
    // (division, bpm) -- see NoteDivision::seconds in
    // sequencing/note_division.rs. Delay::set_time takes that result
    // directly; Flanger's own rate knob wants Hz, so this inverts the
    // note length into a sweep frequency (a note lasting half a second
    // sweeps once every half second, i.e. 2Hz). Reverb has no
    // time/rate concept in the same sense, so division doesn't touch it
    // at all.
    pub fn set_division(&mut self, division: NoteDivision) {
        self.division = division;
        self.apply_division();
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
        self.apply_division();
    }

    fn apply_division(&mut self) {
        let seconds = self.division.seconds(self.bpm);

        self.delay.set_time(seconds);
        self.flanger.set_rate(1.0 / seconds.max(0.001));
    }

    // one Dry/Wet knob for whichever effect is active, same Pioneer
    // Beat-FX convention as `enabled` -- pushed into all three effects'
    // own mix so switching `active` never leaves a stale, unmatched mix
    // value behind on whichever one you switch to
    pub fn set_dry_wet(&mut self, dry_wet: DryWet) {
        self.dry_wet = dry_wet.clamp(0.0, 1.0);

        self.delay.set_mix(self.dry_wet);
        self.reverb.set_mix(self.dry_wet);
        self.flanger.set_mix(self.dry_wet);
    }

    pub fn set_channel(&mut self, channel: usize) {
        self.channel = channel;
    }

    // Runs `input` through whichever effect is active (still calling
    // .process() on the other two so their own internal state -- a
    // reverb tail, a flanger's sweep phase -- doesn't go stale while
    // not selected; same "always advance, decide at output" rule
    // MixerEngine/InstrumentChannel already follow for mute/solo),
    // and passes `input` through unchanged if `enabled` is false.
    // dry/wet blending already happens inside each effect's own
    // process() (see set_dry_wet above), so nothing further to blend
    // here once the active one's been picked.
    pub fn process(&mut self, input: Sample) -> Sample {
        let delay_out = self.delay.process(input);
        let reverb_out = self.reverb.process(input);
        let flanger_out = self.flanger.process(input);

        if !self.enabled {
            return input;
        }

        match self.active {
            EffectType::Delay => delay_out,
            EffectType::Reverb => reverb_out,
            EffectType::Flanger => flanger_out,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_effects_unit_has_expected_defaults() {
        let unit = EffectsUnit::new(48_000.0);

        assert_eq!(unit.active(), EffectType::Delay);
        assert!(!unit.enabled());
        assert_eq!(unit.division(), NoteDivision::Quarter);
        assert_eq!(unit.dry_wet(), DEFAULT_DRY_WET);
        assert_eq!(unit.channel(), 0);
    }

    #[test]
    fn set_active_updates_the_getter() {
        let mut unit = EffectsUnit::new(48_000.0);

        unit.set_active(EffectType::Reverb);

        assert_eq!(unit.active(), EffectType::Reverb);
    }

    #[test]
    fn set_enabled_updates_the_getter() {
        let mut unit = EffectsUnit::new(48_000.0);

        unit.set_enabled(true);

        assert!(unit.enabled());
    }

    #[test]
    fn set_channel_updates_the_getter() {
        let mut unit = EffectsUnit::new(48_000.0);

        unit.set_channel(2);

        assert_eq!(unit.channel(), 2);
    }

    #[test]
    fn set_division_updates_the_getter() {
        let mut unit = EffectsUnit::new(48_000.0);

        unit.set_division(NoteDivision::Sixteenth);

        assert_eq!(unit.division(), NoteDivision::Sixteenth);
    }

    #[test]
    fn set_dry_wet_is_clamped() {
        let mut unit = EffectsUnit::new(48_000.0);

        unit.set_dry_wet(5.0);
        assert_eq!(unit.dry_wet(), 1.0);

        unit.set_dry_wet(-5.0);
        assert_eq!(unit.dry_wet(), 0.0);
    }

    #[test]
    fn disabled_unit_passes_input_through_unchanged() {
        let mut unit = EffectsUnit::new(48_000.0);
        unit.set_dry_wet(1.0);
        unit.set_active(EffectType::Reverb);

        for i in 0..1000 {
            let input = if i % 2 == 0 { 0.6 } else { -0.3 };
            assert_eq!(unit.process(input), input);
        }
    }

    #[test]
    fn switching_active_effect_changes_the_output() {
        // Delay defaults to a 0.3s time (14,400 samples @ 48kHz) and
        // Reverb's combs don't reach back into their own history until
        // several hundred samples in either -- a short window would
        // just compare two all-zero tails, so this needs to run long
        // enough for both to actually produce something
        let mut delay_unit = EffectsUnit::new(48_000.0);
        delay_unit.set_enabled(true);
        delay_unit.set_dry_wet(1.0);
        delay_unit.set_active(EffectType::Delay);
        delay_unit.process(1.0);
        let delay_tail: Vec<Sample> = (0..20_000).map(|_| delay_unit.process(0.0)).collect();

        let mut reverb_unit = EffectsUnit::new(48_000.0);
        reverb_unit.set_enabled(true);
        reverb_unit.set_dry_wet(1.0);
        reverb_unit.set_active(EffectType::Reverb);
        reverb_unit.process(1.0);
        let reverb_tail: Vec<Sample> = (0..20_000).map(|_| reverb_unit.process(0.0)).collect();

        assert_ne!(delay_tail, reverb_tail);
    }

    #[test]
    fn inactive_effects_still_advance_so_switching_is_seamless() {
        let mut unit = EffectsUnit::new(48_000.0);
        unit.set_enabled(true);
        unit.set_dry_wet(1.0);
        unit.set_active(EffectType::Delay);

        // feed Reverb (currently inactive) a loud, sustained input for a
        // while, then switch to it -- if process() only ever called
        // .process() on whichever effect is active, Reverb would still
        // be completely silent/fresh at this point
        for _ in 0..1000 {
            unit.process(1.0);
        }

        unit.set_active(EffectType::Reverb);

        let output = unit.process(0.0);

        assert_ne!(
            output, 0.0,
            "expected the reverb to have already accumulated tail energy from being fed input while inactive"
        );
    }

    #[test]
    fn set_bpm_and_division_do_not_panic_and_stay_queryable() {
        let mut unit = EffectsUnit::new(48_000.0);

        unit.set_bpm(90.0);
        unit.set_division(NoteDivision::ThirtySecond);

        assert_eq!(unit.division(), NoteDivision::ThirtySecond);

        // must not panic even at extreme bpm -- apply_division's
        // seconds()/1.0-over-seconds math both stay guarded
        unit.set_bpm(0.0);
        unit.set_bpm(-10.0);
    }

    #[test]
    fn process_output_stays_finite() {
        let mut unit = EffectsUnit::new(48_000.0);
        unit.set_enabled(true);
        unit.set_dry_wet(0.8);

        for effect in [EffectType::Delay, EffectType::Reverb, EffectType::Flanger] {
            unit.set_active(effect);

            for i in 0..20_000 {
                let input = if i % 200 < 100 { 1.0 } else { -1.0 };
                let sample = unit.process(input);

                assert!(sample.is_finite(), "non-finite output for {:?}: {}", effect, sample);
            }
        }
    }
}
