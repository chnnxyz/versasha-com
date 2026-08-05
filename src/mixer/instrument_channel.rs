use crate::dsp::eq::eq3::Eq3;
use crate::dsp::types::{Sample, SampleRate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentChannelStatus {
    Solo,
    Muted,
    Active,
}

// VU-style peak meter release time -- how long a peak takes to decay back
// down to (near) silence after the signal drops, not how fast it rises
// (rise is instant). 300ms matches typical analog VU ballistics.
const PEAK_DECAY_SECONDS: f32 = 0.3;
const PEAK_DECAY_FLOOR: f32 = 0.001; // -60dB -- "decayed" target after PEAK_DECAY_SECONDS

// pub(crate) -- MixerEngine reuses this for its own master_peak, so both
// meters share identical ballistics
pub(crate) fn peak_decay_per_sample(rate: SampleRate) -> f32 {
    PEAK_DECAY_FLOOR.powf(1.0 / (PEAK_DECAY_SECONDS * rate))
}

// a single mixer channel, a vector of these is associated to a vector of
// different instruments.
pub struct InstrumentChannel {
    status: InstrumentChannelStatus,
    volume: f32,
    eq: Eq3,
    pan: f32,

    // this channel's own post-eq/volume/pan output level, for a UI VU
    // meter -- rises instantly on a transient, decays exponentially
    // afterward rather than following the raw signal sample-to-sample
    peak: Sample,
    peak_decay: f32,
}

impl InstrumentChannel {
    pub fn new(rate: SampleRate) -> Self {
        Self {
            status: InstrumentChannelStatus::Active,
            volume: 1.0,
            eq: Eq3::new(rate),
            pan: 0.0,
            peak: 0.0,
            peak_decay: peak_decay_per_sample(rate),
        }
    }

    // getters

    pub fn status(&self) -> InstrumentChannelStatus {
        self.status
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    // exposes the Eq3 directly for its own set_low_gain/set_high_freq/etc,
    // rather than InstrumentChannel re-exposing every Eq3 setter itself --
    // keeps this type from having to grow a passthrough method every time
    // Eq3 grows a knob
    pub fn eq_mut(&mut self) -> &mut Eq3 {
        &mut self.eq
    }
    pub fn pan(&self) -> f32 {
        self.pan
    }

    pub fn peak(&self) -> Sample {
        self.peak
    }

    // setters
    pub fn set_status(&mut self, status: InstrumentChannelStatus) {
        self.status = status
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    // Runs `input` through this channel's EQ and volume, then splits the
    // result into a stereo pair with an equal-power ("constant-power")
    // pan law -- keeps perceived loudness constant while panning, unlike
    // a plain linear left/right split which dips in the middle. Does NOT
    // apply mute or solo -- MixerEngine handles both of those itself,
    // since solo needs to compare across every channel at once, not
    // something one channel can decide alone.
    pub fn process(&mut self, input: Sample) -> (Sample, Sample) {
        let mono = self.eq.process(input) * self.volume;

        // pan -1.0..1.0 -> angle 0..PI/2, so (cos, sin) sweeps from
        // (1, 0) at full left to (0, 1) at full right, with
        // cos^2 + sin^2 == 1 at every point in between
        let angle = (self.pan + 1.0) * std::f32::consts::FRAC_PI_4;
        let (left, right) = (mono * angle.cos(), mono * angle.sin());

        let instantaneous = left.abs().max(right.abs());
        self.peak = (self.peak * self.peak_decay).max(instantaneous);

        (left, right)
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
    fn new_has_defaults() {
        let channel = InstrumentChannel::new(48_000.0);

        assert_eq!(channel.status(), InstrumentChannelStatus::Active);
        assert_approx_eq(channel.volume(), 1.0);
        assert_approx_eq(channel.pan(), 0.0);
        assert_approx_eq(channel.peak(), 0.0);
    }

    #[test]
    fn set_status_updates_value() {
        let mut channel = InstrumentChannel::new(48_000.0);

        channel.set_status(InstrumentChannelStatus::Solo);
        assert_eq!(channel.status(), InstrumentChannelStatus::Solo);

        channel.set_status(InstrumentChannelStatus::Muted);
        assert_eq!(channel.status(), InstrumentChannelStatus::Muted);
    }

    #[test]
    fn set_volume_clamps() {
        let mut channel = InstrumentChannel::new(48_000.0);

        channel.set_volume(2.0);
        assert_approx_eq(channel.volume(), 1.0);

        channel.set_volume(-1.0);
        assert_approx_eq(channel.volume(), 0.0);
    }

    #[test]
    fn set_pan_clamps() {
        let mut channel = InstrumentChannel::new(48_000.0);

        channel.set_pan(2.0);
        assert_approx_eq(channel.pan(), 1.0);

        channel.set_pan(-2.0);
        assert_approx_eq(channel.pan(), -1.0);
    }

    #[test]
    fn eq_mut_reaches_the_underlying_eq3() {
        let mut channel = InstrumentChannel::new(48_000.0);

        channel.eq_mut().set_low_gain(0.0);

        assert_approx_eq(channel.eq_mut().low_gain(), 0.0);
    }

    #[test]
    fn process_applies_eq_and_volume() {
        let mut channel = InstrumentChannel::new(48_000.0);

        channel.set_volume(0.5);

        // Eq3 defaults to unity gain on every band, which reconstructs
        // the input exactly (an algebraic identity, no settling needed --
        // see Eq3's own unity_gain_reconstructs_the_input test), so this
        // reduces to a straight volume scale, split evenly left/right
        // since pan defaults to center
        let (left, right) = channel.process(1.0);
        let center = std::f32::consts::FRAC_PI_4.cos();

        assert_approx_eq(left, 0.5 * center);
        assert_approx_eq(right, 0.5 * center);
    }

    #[test]
    fn process_reflects_changes_made_through_eq_mut() {
        let mut channel = InstrumentChannel::new(48_000.0);

        channel.eq_mut().set_low_gain(0.0);
        channel.eq_mut().set_mid_gain(0.0);
        channel.eq_mut().set_high_gain(0.0);

        let mut output = (1.0, 1.0);
        for _ in 0..10_000 {
            output = channel.process(1.0);
        }

        assert_approx_eq(output.0, 0.0);
        assert_approx_eq(output.1, 0.0);
    }

    #[test]
    fn process_does_not_apply_mute_or_solo() {
        let mut channel = InstrumentChannel::new(48_000.0);

        channel.set_status(InstrumentChannelStatus::Muted);

        // process() only ever applies EQ + volume + pan -- mute/solo is
        // MixerEngine's job, not this channel's
        let (left, right) = channel.process(1.0);
        let center = std::f32::consts::FRAC_PI_4.cos();

        assert_approx_eq(left, center);
        assert_approx_eq(right, center);
    }

    #[test]
    fn process_pans_fully_left() {
        let mut channel = InstrumentChannel::new(48_000.0);

        channel.set_pan(-1.0);
        let (left, right) = channel.process(1.0);

        assert_approx_eq(left, 1.0);
        assert_approx_eq(right, 0.0);
    }

    #[test]
    fn process_pans_fully_right() {
        let mut channel = InstrumentChannel::new(48_000.0);

        channel.set_pan(1.0);
        let (left, right) = channel.process(1.0);

        assert_approx_eq(left, 0.0);
        assert_approx_eq(right, 1.0);
    }

    #[test]
    fn process_splits_evenly_at_center_pan() {
        let mut channel = InstrumentChannel::new(48_000.0);

        // pan defaults to 0.0 (center)
        let (left, right) = channel.process(1.0);

        assert_approx_eq(left, right);
    }

    #[test]
    fn equal_power_pan_preserves_total_signal_power() {
        let mut channel = InstrumentChannel::new(48_000.0);

        for pan in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            channel.set_pan(pan);
            let (left, right) = channel.process(1.0);

            // constant-power pan law: left^2 + right^2 stays 1.0 (the
            // mono input's own power) at every pan position, not just
            // the extremes -- that's what keeps perceived loudness
            // constant while panning
            assert_approx_eq(left * left + right * right, 1.0);
        }
    }

    #[test]
    fn peak_rises_instantly_on_a_loud_sample() {
        let mut channel = InstrumentChannel::new(48_000.0);

        channel.process(1.0);

        // unity gain, unity volume, center pan -- process()'s own
        // constant-power split puts each channel at cos(45deg), so peak
        // should track that immediately, not ramp up to it
        assert_approx_eq(channel.peak(), std::f32::consts::FRAC_PI_4.cos());
    }

    #[test]
    fn peak_decays_toward_zero_once_the_signal_stops() {
        let mut channel = InstrumentChannel::new(48_000.0);

        channel.process(1.0);
        let initial_peak = channel.peak();

        for _ in 0..48_000 {
            channel.process(0.0);
        }

        assert!(
            channel.peak() < initial_peak * 0.01,
            "expected peak to have decayed close to zero after 1 second of silence, got {}",
            channel.peak()
        );
    }

    #[test]
    fn peak_reflects_zero_volume() {
        let mut channel = InstrumentChannel::new(48_000.0);

        channel.set_volume(0.0);
        channel.process(1.0);

        assert_approx_eq(channel.peak(), 0.0);
    }
}
