use crate::dsp::types::{Sample, SampleRate};
use crate::mixer::effects_unit::EffectsUnit;
use crate::mixer::instrument_channel::{peak_decay_per_sample, InstrumentChannel, InstrumentChannelStatus};

// Owns one InstrumentChannel per instrument and sums them into the
// final mix. A Vec, not fixed fields, on purpose -- so instruments can
// be added or removed later (a second synth voice, a sampler, whatever)
// without this type's shape changing, only its length.
pub struct MixerEngine {
    channels: Vec<InstrumentChannel>,
    master_volume: f32,

    // one shared Beat FX send for the whole mixer (see mixer/
    // effects_unit.rs) -- inserted mono, pre-EQ/pan/volume, on whichever
    // channel effects_unit.channel() currently points at
    effects_unit: EffectsUnit,

    // the final summed-and-master-scaled output level, per channel, for
    // a stereo UI VU meter -- same instant-rise/exponential-decay
    // ballistics as each InstrumentChannel's own peak, just measured
    // after everything (including master_volume) has already been
    // applied. Kept as a left/right pair, unlike each InstrumentChannel's
    // single combined peak, since this is the one meter in the mixer
    // where seeing the two sides independently actually matters.
    master_peak_left: Sample,
    master_peak_right: Sample,
    master_peak_decay: f32,
}

impl MixerEngine {
    pub fn new(channel_count: usize, rate: SampleRate) -> Self {
        Self {
            channels: std::iter::repeat_with(|| InstrumentChannel::new(rate))
                .take(channel_count)
                .collect(),
            master_volume: 1.0,
            master_peak_left: 0.0,
            master_peak_right: 0.0,
            master_peak_decay: peak_decay_per_sample(rate),
            effects_unit: EffectsUnit::new(rate),
        }
    }

    // exposes EffectsUnit directly for its own set_active/set_enabled/
    // etc, same reasoning as channel_mut/eq_mut not re-exposing every
    // knob themselves
    pub fn effects_unit_mut(&mut self) -> &mut EffectsUnit {
        &mut self.effects_unit
    }

    pub fn effects_unit(&self) -> &EffectsUnit {
        &self.effects_unit
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn channel_mut(&mut self, index: usize) -> Option<&mut InstrumentChannel> {
        self.channels.get_mut(index)
    }

    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    pub fn master_peak_left(&self) -> Sample {
        self.master_peak_left
    }

    pub fn master_peak_right(&self) -> Sample {
        self.master_peak_right
    }

    fn any_solo(&self) -> bool {
        self.channels
            .iter()
            .any(|c| c.status() == InstrumentChannelStatus::Solo)
    }

    // Sums whichever channels qualify (never muted; solo-only if any
    // channel is soloed) after running each one's own EQ + volume --
    // channel.process() always runs regardless, so state never goes
    // stale just because a channel is excluded this sample. Scales the
    // sum by master_volume last, so the master meter and the final
    // output both reflect it.
    //
    // zip() over inputs, not indexing, so a mismatched length just
    // truncates instead of panicking mid-callback.
    //
    // effects_unit is inserted mono, pre-EQ/pan/volume, on whichever
    // one channel effects_unit.channel() points at -- computed once up
    // front (not inside the channels loop below) both to avoid
    // borrowing self.effects_unit and self.channels mutably at the same
    // time, and to keep this a single call per sample regardless of how
    // many channels there are.
    pub fn process(&mut self, inputs: &[Sample]) -> (Sample, Sample) {
        let solo_active = self.any_solo();

        let effects_channel = self.effects_unit.channel();
        let effected_sample = inputs
            .get(effects_channel)
            .map(|&raw| self.effects_unit.process(raw));

        let (l_sum, r_sum) = self
            .channels
            .iter_mut()
            .zip(inputs)
            .enumerate()
            .map(|(index, (channel, &input))| {
                let input = if index == effects_channel {
                    effected_sample.unwrap_or(input)
                } else {
                    input
                };

                let (left, right) = channel.process(input);
                let qualifies = channel.status() != InstrumentChannelStatus::Muted
                    && (!solo_active || channel.status() == InstrumentChannelStatus::Solo);

                if qualifies { (left, right) } else { (0.0, 0.0) }
            })
            .fold((0.0, 0.0), |(l_sum, r_sum), (l, r)| (l_sum + l, r_sum + r));

        let (left, right) = (l_sum * self.master_volume, r_sum * self.master_volume);

        self.master_peak_left = (self.master_peak_left * self.master_peak_decay).max(left.abs());
        self.master_peak_right = (self.master_peak_right * self.master_peak_decay).max(right.abs());

        (left, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mixer::effects_unit::EffectType;
    use crate::sequencing::note_division::NoteDivision;

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
    fn new_creates_the_requested_channel_count() {
        let mixer = MixerEngine::new(3, 48_000.0);

        assert_eq!(mixer.channel_count(), 3);
        assert_approx_eq(mixer.master_volume(), 1.0);
        assert_approx_eq(mixer.master_peak_left(), 0.0);
        assert_approx_eq(mixer.master_peak_right(), 0.0);
    }

    #[test]
    fn set_master_volume_clamps() {
        let mut mixer = MixerEngine::new(1, 48_000.0);

        mixer.set_master_volume(2.0);
        assert_approx_eq(mixer.master_volume(), 1.0);

        mixer.set_master_volume(-1.0);
        assert_approx_eq(mixer.master_volume(), 0.0);
    }

    #[test]
    fn channel_mut_returns_none_for_out_of_range_index() {
        let mut mixer = MixerEngine::new(2, 48_000.0);

        assert!(mixer.channel_mut(999).is_none());
    }

    #[test]
    fn channel_mut_reaches_an_independent_channel() {
        let mut mixer = MixerEngine::new(2, 48_000.0);

        mixer.channel_mut(0).unwrap().set_volume(0.25);

        assert_approx_eq(mixer.channel_mut(0).unwrap().volume(), 0.25);
        assert_approx_eq(mixer.channel_mut(1).unwrap().volume(), 1.0);
    }

    #[test]
    fn process_sums_every_active_channel() {
        let mut mixer = MixerEngine::new(3, 48_000.0);
        let center = std::f32::consts::FRAC_PI_4.cos();

        // default status Active, volume 1.0, eq unity gain, pan center
        // on every channel -- process() reduces to a straight sum of
        // inputs, split evenly left/right
        let (left, right) = mixer.process(&[1.0, 2.0, 3.0]);

        assert_approx_eq(left, 6.0 * center);
        assert_approx_eq(right, 6.0 * center);
    }

    #[test]
    fn process_excludes_muted_channels() {
        let mut mixer = MixerEngine::new(2, 48_000.0);
        let center = std::f32::consts::FRAC_PI_4.cos();

        mixer
            .channel_mut(1)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);

        let (left, right) = mixer.process(&[1.0, 1.0]);

        assert_approx_eq(left, center);
        assert_approx_eq(right, center);
    }

    #[test]
    fn process_restricts_to_soloed_channels_when_any_is_soloed() {
        let mut mixer = MixerEngine::new(3, 48_000.0);
        let center = std::f32::consts::FRAC_PI_4.cos();

        mixer
            .channel_mut(1)
            .unwrap()
            .set_status(InstrumentChannelStatus::Solo);

        let (left, right) = mixer.process(&[1.0, 1.0, 1.0]);

        assert_approx_eq(left, center);
        assert_approx_eq(right, center);
    }

    #[test]
    fn process_applies_each_channels_own_volume() {
        let mut mixer = MixerEngine::new(2, 48_000.0);
        let center = std::f32::consts::FRAC_PI_4.cos();

        mixer.channel_mut(0).unwrap().set_volume(0.5);
        mixer.channel_mut(1).unwrap().set_volume(0.25);

        let (left, right) = mixer.process(&[1.0, 1.0]);

        assert_approx_eq(left, 0.75 * center);
        assert_approx_eq(right, 0.75 * center);
    }

    #[test]
    fn process_pans_a_channel_independently_of_the_others() {
        let mut mixer = MixerEngine::new(2, 48_000.0);

        mixer.channel_mut(0).unwrap().set_pan(-1.0); // channel 0 hard left
        mixer.channel_mut(1).unwrap().set_pan(1.0); // channel 1 hard right

        let (left, right) = mixer.process(&[1.0, 1.0]);

        assert_approx_eq(left, 1.0);
        assert_approx_eq(right, 1.0);
    }

    #[test]
    fn process_applies_master_volume_after_summing() {
        let mut mixer = MixerEngine::new(2, 48_000.0);
        let center = std::f32::consts::FRAC_PI_4.cos();

        mixer.set_master_volume(0.5);

        let (left, right) = mixer.process(&[1.0, 1.0]);

        assert_approx_eq(left, 2.0 * center * 0.5);
        assert_approx_eq(right, 2.0 * center * 0.5);
    }

    #[test]
    fn master_peak_rises_instantly_and_reflects_master_volume() {
        let mut mixer = MixerEngine::new(1, 48_000.0);

        mixer.set_master_volume(0.5);
        mixer.process(&[1.0]);

        let center = std::f32::consts::FRAC_PI_4.cos();
        assert_approx_eq(mixer.master_peak_left(), center * 0.5);
        assert_approx_eq(mixer.master_peak_right(), center * 0.5);
    }

    #[test]
    fn master_peak_decays_toward_zero_once_signal_stops() {
        let mut mixer = MixerEngine::new(1, 48_000.0);

        mixer.process(&[1.0]);
        let initial_peak = mixer.master_peak_left();

        for _ in 0..48_000 {
            mixer.process(&[0.0]);
        }

        assert!(
            mixer.master_peak_left() < initial_peak * 0.01,
            "expected master_peak_left to have decayed close to zero after 1 second of silence, got {}",
            mixer.master_peak_left()
        );
        assert!(
            mixer.master_peak_right() < initial_peak * 0.01,
            "expected master_peak_right to have decayed close to zero after 1 second of silence, got {}",
            mixer.master_peak_right()
        );
    }

    #[test]
    fn master_peak_left_and_right_track_independently() {
        let mut mixer = MixerEngine::new(1, 48_000.0);

        // hard-panned left -- only the left master peak should register
        // anything, proving this isn't just one combined meter mirrored
        // onto two bars
        mixer.channel_mut(0).unwrap().set_pan(-1.0);
        mixer.process(&[1.0]);

        assert_approx_eq(mixer.master_peak_left(), 1.0);
        assert_approx_eq(mixer.master_peak_right(), 0.0);
    }

    #[test]
    fn process_does_not_panic_on_input_length_mismatch() {
        let mut mixer = MixerEngine::new(2, 48_000.0);

        // must not panic -- fewer inputs than channels
        mixer.process(&[1.0]);

        // must not panic -- more inputs than channels
        mixer.process(&[1.0, 1.0, 1.0]);
    }

    #[test]
    fn excluded_channels_keep_their_eq_state_advancing() {
        let mut warmed_up = MixerEngine::new(1, 48_000.0);
        warmed_up.channel_mut(0).unwrap().eq_mut().set_low_gain(0.0);
        warmed_up
            .channel_mut(0)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);

        // drive the low-band crossover filter toward its settled
        // response while muted -- it must not be frozen by exclusion
        // from the sum
        for _ in 0..10_000 {
            warmed_up.process(&[1.0]);
        }

        warmed_up
            .channel_mut(0)
            .unwrap()
            .set_status(InstrumentChannelStatus::Active);
        let (warmed_output, _) = warmed_up.process(&[1.0]);

        let mut fresh = MixerEngine::new(1, 48_000.0);
        fresh.channel_mut(0).unwrap().eq_mut().set_low_gain(0.0);
        let (fresh_output, _) = fresh.process(&[1.0]);

        // a freshly-created channel is still at its very first sample (no
        // accumulated filter state) -- if muting had frozen advancement
        // instead of letting it keep running, these two would match
        // exactly
        assert!(
            (warmed_output - fresh_output).abs() > EPSILON,
            "expected muted channel's filter state to have advanced: warmed={warmed_output}, fresh={fresh_output}"
        );
    }

    #[test]
    fn effects_unit_mut_reaches_the_underlying_effects_unit() {
        let mut mixer = MixerEngine::new(2, 48_000.0);

        mixer.effects_unit_mut().set_channel(1);

        assert_eq!(mixer.effects_unit().channel(), 1);
    }

    #[test]
    fn enabled_effects_unit_reaches_its_own_channel() {
        let mut mixer = MixerEngine::new(2, 48_000.0);

        mixer.effects_unit_mut().set_enabled(true);
        mixer.effects_unit_mut().set_dry_wet(1.0);
        mixer.effects_unit_mut().set_active(EffectType::Delay);
        mixer.effects_unit_mut().set_channel(1);
        // fast tempo/division so the delayed echo lands inside a
        // reasonably small test window
        mixer.effects_unit_mut().set_bpm(300.0);
        mixer.effects_unit_mut().set_division(NoteDivision::ThirtySecond);

        // isolate channel 1 -- only it should carry the effect's echo
        mixer
            .channel_mut(0)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);

        mixer.process(&[0.0, 1.0]);

        let echo_audible = (0..3000).any(|_| mixer.process(&[0.0, 0.0]) != (0.0, 0.0));

        assert!(echo_audible, "expected the effect's delayed echo to reach channel 1's output");
    }

    #[test]
    fn effects_unit_does_not_affect_channels_it_is_not_routed_to() {
        let mut mixer = MixerEngine::new(2, 48_000.0);
        let center = std::f32::consts::FRAC_PI_4.cos();

        mixer.effects_unit_mut().set_enabled(true);
        mixer.effects_unit_mut().set_dry_wet(1.0);
        mixer.effects_unit_mut().set_active(EffectType::Delay);
        mixer.effects_unit_mut().set_channel(1); // NOT channel 0

        // isolate channel 0 -- the effect is routed to channel 1, so
        // channel 0's output should be identical to a mixer that never
        // had an effects unit involved at all
        mixer
            .channel_mut(1)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);

        let (left, right) = mixer.process(&[1.0, 0.0]);

        assert_approx_eq(left, center);
        assert_approx_eq(right, center);
    }
}
