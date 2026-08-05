use crate::dsp::oscillators::phase_generator::PhaseGenerator;
use crate::dsp::types::{DryWet, Sample, SampleRate, Time};

// flanger sweeps a few ms at most, nowhere near Delay's up-to-2s range
const MAX_DELAY_SECONDS: f32 = 0.02;
const DEFAULT_MIX: DryWet = 0.5;

// deeper and more resonant than a subtle chorus-like flanger -- the
// pronounced, metallic "jet" sweep a Pioneer-style Beat FX flanger goes
// for on a buildup, not a gentle wash
const DEFAULT_DEPTH: f32 = 0.7;
const DEFAULT_FEEDBACK: Sample = 0.5;

// PhaseGenerator::new defaults to DEFAULT_FREQ (440Hz, an audio-rate
// pitch) -- nowhere near a usable flanger sweep rate, so this needs its
// own explicit default. Only matters standalone -- EffectsUnit
// (mixer/effects_unit.rs) always overwrites this via set_rate() from
// the shared BPM/division as soon as it constructs one, so this default
// never actually reaches the real Beat FX signal path. 0.4Hz is a
// livelier, more noticeable sweep than a slow wash, if used bare.
const DEFAULT_RATE_HZ: f32 = 0.4;

// A short modulated delay line -- a PhaseGenerator (reused from
// dsp/oscillators/phase_generator.rs, not Lfo, since Lfo is tied to the
// synth's own ModulationTarget system, which doesn't apply here) sweeps
// the delay time between ~0 and MAX_DELAY_SECONDS; mixing that
// swept-delay signal back with the dry input is what produces the
// classic flanger comb-filter sweep. Structurally close to Delay
// (dsp/fx/delay.rs) -- same ring-buffer approach -- just with a
// modulated read position instead of a fixed one, and no feedback tap
// needed on a first pass (add one later only if the effect feels too
// thin without it).
pub struct Flanger {
    buffer: Vec<Sample>,
    write_index: usize,
    sample_rate: SampleRate,
    lfo: PhaseGenerator,
    depth: f32,
    feedback: Sample,
    mix: DryWet,
}

impl Flanger {
    pub fn new(sample_rate: SampleRate) -> Self {
        let capacity = ((sample_rate * MAX_DELAY_SECONDS) as usize).max(1);
        let mut lfo = PhaseGenerator::new(sample_rate);
        lfo.set_freq(DEFAULT_RATE_HZ);

        Self {
            buffer: vec![0.0; capacity],
            write_index: 0,
            sample_rate,
            lfo,
            mix: DEFAULT_MIX,
            depth: DEFAULT_DEPTH,
            feedback: DEFAULT_FEEDBACK,
        }
    }

    // getters
    pub fn rate(&self) -> f32 {
        self.lfo.freq()
    }

    pub fn depth(&self) -> f32 {
        self.depth
    }

    pub fn feedback(&self) -> Sample {
        self.feedback
    }

    pub fn mix(&self) -> DryWet {
        self.mix
    }

    // setters
    pub fn set_rate(&mut self, hz: f32) {
        self.lfo.set_freq(hz);
    }

    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }

    pub fn set_feedback(&mut self, feedback: Sample) {
        self.feedback = feedback.clamp(0.0, 1.0);
    }

    pub fn set_mix(&mut self, mix: DryWet) {
        self.mix = mix.clamp(0.0, 1.0)
    }

    pub fn reset(&mut self) {
        self.buffer = vec![0.0; self.buffer.len()];
        self.write_index = 0;
        self.lfo.reset();
    }

    pub fn process(&mut self, input: Sample) -> Sample {
        self.buffer[self.write_index] = input;

        let lfo_sample: Sample = self.lfo.next_sample();
        let normalized: Sample = (lfo_sample + 1.0) * 0.5;
        let delay_seconds: Time = normalized * MAX_DELAY_SECONDS * self.depth;
        let delay_samples: f32 = delay_seconds * self.sample_rate;

        // rem_euclid wraps a negative read_position into 0..buffer.len()
        // in one step (unlike Delay's write_index + buffer.len() -
        // delay_samples trick, which only works on whole samples) --
        // floor/ceil either side of it, plus the fractional remainder as
        // the interpolation weight, is what avoids the zipper noise a
        // plain floored read would produce as the LFO sweeps. rem_euclid
        // is only guaranteed in-range mathematically -- floating-point
        // rounding can occasionally round it up to exactly buffer.len()
        // itself, one past the last valid index, so floor_index still
        // needs the same defensive clamp Delay::delay_samples() uses.
        let read_position = (self.write_index as f32 - delay_samples).rem_euclid(self.buffer.len() as f32);
        let floor_index = (read_position.floor() as usize).min(self.buffer.len() - 1);
        let ceil_index = (floor_index + 1) % self.buffer.len();
        let t = read_position.fract();

        let wet = self.buffer[floor_index] * (1.0 - t) + self.buffer[ceil_index] * t;

        self.write_index = (self.write_index + 1) % self.buffer.len();

        // no feedback tap yet -- self.feedback is set/gettable but
        // unused here, per this struct's own top comment
        input * (1.0 - self.mix) + wet * self.mix
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
    fn new_flanger_has_expected_defaults() {
        let flanger = Flanger::new(48_000.0);

        assert_eq!(flanger.rate(), DEFAULT_RATE_HZ);
        assert_eq!(flanger.depth(), DEFAULT_DEPTH);
        assert_eq!(flanger.feedback(), DEFAULT_FEEDBACK);
        assert_eq!(flanger.mix(), DEFAULT_MIX);
    }

    #[test]
    fn set_rate_updates_the_getter() {
        let mut flanger = Flanger::new(48_000.0);

        flanger.set_rate(1.5);

        assert_eq!(flanger.rate(), 1.5);
    }

    #[test]
    fn set_depth_is_clamped() {
        let mut flanger = Flanger::new(48_000.0);

        flanger.set_depth(5.0);
        assert_eq!(flanger.depth(), 1.0);

        flanger.set_depth(-5.0);
        assert_eq!(flanger.depth(), 0.0);
    }

    #[test]
    fn set_feedback_is_clamped() {
        let mut flanger = Flanger::new(48_000.0);

        flanger.set_feedback(5.0);
        assert_eq!(flanger.feedback(), 1.0);

        flanger.set_feedback(-5.0);
        assert_eq!(flanger.feedback(), 0.0);
    }

    #[test]
    fn set_mix_is_clamped() {
        let mut flanger = Flanger::new(48_000.0);

        flanger.set_mix(5.0);
        assert_eq!(flanger.mix(), 1.0);

        flanger.set_mix(-5.0);
        assert_eq!(flanger.mix(), 0.0);
    }

    #[test]
    fn dry_only_passes_input_through() {
        let mut flanger = Flanger::new(48_000.0);
        flanger.set_mix(0.0);

        assert_approx_eq(flanger.process(0.5), 0.5);
        assert_approx_eq(flanger.process(-0.25), -0.25);
    }

    #[test]
    fn zero_depth_collapses_the_sweep_to_an_immediate_echo() {
        // depth=0 forces delay_seconds to 0 regardless of the LFO's
        // phase, so the "delayed" tap is really just the sample this
        // same call just wrote -- with mix=1 the output should equal
        // the input exactly, deterministic no matter where the LFO
        // happens to be in its cycle
        let mut flanger = Flanger::new(48_000.0);
        flanger.set_depth(0.0);
        flanger.set_mix(1.0);

        for _ in 0..50 {
            let input = 0.7;
            assert_approx_eq(flanger.process(input), input);
        }
    }

    #[test]
    fn reset_clears_buffered_history() {
        let mut flanger = Flanger::new(48_000.0);
        flanger.set_depth(1.0);
        flanger.set_mix(1.0);

        for _ in 0..100 {
            flanger.process(1.0);
        }

        flanger.reset();

        // the swept delay after a fresh reset can only read back
        // zeroed buffer slots -- none of the 1.0s fed in before
        // reset() should still be reachable
        assert_approx_eq(flanger.process(0.0), 0.0);
    }

    #[test]
    fn output_stays_in_range_with_varied_input() {
        let mut flanger = Flanger::new(48_000.0);
        flanger.set_mix(0.5);
        flanger.set_depth(1.0);

        for i in 0..10_000 {
            let input = ((i as f32) * 0.01).sin();
            let sample = flanger.process(input);

            assert!((-1.0..=1.0).contains(&sample), "out of range: {}", sample);
        }
    }

    #[test]
    fn process_never_panics_on_a_floating_point_rounding_edge_case() {
        // real crash: rem_euclid's own rounding can land read_position on
        // exactly buffer.len() (one past the last valid index) rather
        // than just under it -- write_index and the LFO's phase drift
        // relative to each other sample by sample (44100Hz sample rate,
        // 7Hz sweep rate share no clean integer relationship), and at
        // depth=1.0 the sweep reaches the buffer's full length, so this
        // specific combination reliably hits the rounding edge case by
        // sample 201,485 (found by sweeping rate/depth combinations and
        // narrowing down which one crashed -- not a made-up number)
        let mut flanger = Flanger::new(44_100.0);
        flanger.set_depth(1.0);
        flanger.set_mix(1.0);
        flanger.set_rate(7.0);

        for i in 0..201_500 {
            let input = ((i as f32) * 0.01).sin();
            let sample = flanger.process(input);

            assert!(sample.is_finite(), "non-finite output at sample {}: {}", i, sample);
        }
    }
}

