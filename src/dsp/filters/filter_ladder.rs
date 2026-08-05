use std::f32::consts::PI;

use crate::dsp::types::{Frequency, Sample, SampleRate};

// 3-pole (18dB/octave) diode-ladder-style lowpass, the topology the TB-303
// actually used -- distinct from Filter's 2-pole TPT/Cytomic SVF in
// filter.rs both in pole count and in character: this one saturates
// (tanh) inside its resonance feedback loop, which is where the
// "squelchy"/self-oscillating acid tone comes from. The SVF's resonance
// is clean/linear by comparison.
pub struct LadderFilter {
    sample_rate: SampleRate,
    cutoff: Frequency,
    resonance: f32,

    // one-pole lowpass state per ladder stage, each stage's output feeds
    // the next stage's input; stage3's output is the filter's output
    stage1: Sample,
    stage2: Sample,
    stage3: Sample,
}

impl LadderFilter {
    pub fn new(rate: SampleRate) -> Self {
        // mirrors crate::dsp::Filter, providing default values for when
        // opening the app
        Self {
            sample_rate: rate,
            cutoff: 2000.0,
            resonance: 0.0,
            stage1: 0.0,
            stage2: 0.0,
            stage3: 0.0,
        }
    }

    //getters
    pub fn cutoff(&self) -> Frequency {
        self.cutoff
    }

    pub fn resonance(&self) -> f32 {
        self.resonance
    }

    // setters
    pub fn set_cutoff(&mut self, cutoff: Frequency) {
        self.cutoff = cutoff;
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance.clamp(0.0, 1.0);
    }

    pub fn reset(&mut self) {
        self.stage1 = 0.0;
        self.stage2 = 0.0;
        self.stage3 = 0.0;
    }

    // Recomputes coefficients inline each call, same non-cached style as
    // Filter::process. `g` is the standard exponential one-pole
    // coefficient (this filter's stages are plain one-poles, not the
    // SVF's trapezoidal integrator, so no tan()-based prewarping needed
    // here). `k` scales normalized resonance into a feedback gain that
    // reaches self-oscillation near resonance = 1.0 -- 3.5 was picked by
    // testing against that target, not derived; retune if it doesn't
    // self-oscillate cleanly.
    //
    // Each stage saturates its full new value through tanh(), not just the
    // per-sample delta -- that's what actually bounds the state to (-1, 1)
    // regardless of input size (an increment-only tanh would still let the
    // accumulated state drift unbounded), and it's what gives each stage
    // its own bit of diode-like compression rather than concentrating all
    // the nonlinearity in the feedback path alone.
    pub fn process(&mut self, input: Sample) -> Sample {
        let cutoff: Frequency = self.cutoff.clamp(1.0, 20_000.0);
        let g: f32 = 1.0 - (-2.0 * PI * cutoff / self.sample_rate).exp();
        let k: f32 = self.resonance * 3.5;

        // stage3 here is last sample's output, not this sample's -- a
        // one-sample-delayed feedback path rather than a true
        // zero-delay-feedback loop. Simpler (no iterative solving needed),
        // at the cost of self-oscillation pitch being a slight
        // approximation of the analog original.
        let feedback = self.stage3.tanh();
        let stage_1_input = input - k * feedback;

        self.stage1 = (self.stage1 + g * (stage_1_input - self.stage1)).tanh();
        self.stage2 = (self.stage2 + g * (self.stage1 - self.stage2)).tanh();
        self.stage3 = (self.stage3 + g * (self.stage2 - self.stage3)).tanh();

        self.stage3
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
    fn new_filter_has_defaults() {
        let filter = LadderFilter::new(48_000.0);

        assert_eq!(filter.cutoff(), 2000.0);
        assert_eq!(filter.resonance(), 0.0);
    }

    #[test]
    fn cutoff_can_be_changed() {
        let mut filter = LadderFilter::new(48_000.0);

        filter.set_cutoff(800.0);

        assert_eq!(filter.cutoff(), 800.0);
    }

    #[test]
    fn resonance_is_clamped() {
        let mut filter = LadderFilter::new(48_000.0);

        filter.set_resonance(5.0);
        assert_approx_eq(filter.resonance(), 1.0);

        filter.set_resonance(-1.0);
        assert_approx_eq(filter.resonance(), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut filter = LadderFilter::new(48_000.0);

        for _ in 0..100 {
            filter.process(1.0);
        }

        filter.reset();

        assert_approx_eq(filter.process(0.0), 0.0);
    }

    #[test]
    fn tracks_small_constant_input_near_its_value() {
        // small-signal regime (well inside tanh's near-linear range around
        // 0) is where this filter behaves close to a textbook one-pole
        // cascade -- at full-scale input the per-stage saturation
        // deliberately compresses the settled level below the input, so
        // that's not a fair thing to assert against here. Even 0.05 turned
        // out not small enough: three cascaded saturating stages compound
        // tanh's compression more than a single stage would, so this
        // needs a genuinely tiny amplitude (checked numerically -- 0.01
        // settles within 2e-5 of its target here, comfortably inside
        // EPSILON) to actually test "close to unity gain at DC" instead of
        // just re-measuring the saturation curve
        let mut filter = LadderFilter::new(48_000.0);

        filter.set_cutoff(500.0);

        let mut output = 0.0;

        for _ in 0..10_000 {
            output = filter.process(0.01);
        }

        assert_approx_eq(output, 0.01);
    }

    // small amplitude (0.1) for the same reason as the DC test above --
    // keeps tanh close to identity through most of the swing, so this is
    // a clean-ish read of the filter's frequency response rather than its
    // saturation curve
    fn peak_amplitude(filter: &mut LadderFilter, frequency: Frequency, sample_rate: SampleRate) -> f32 {
        let mut peak: f32 = 0.0;

        for i in 0..4000 {
            let t = i as f32 / sample_rate;
            let input = 0.1 * (2.0 * PI * frequency * t).sin();
            let output = filter.process(input);

            if i > 3000 {
                peak = peak.max(output.abs());
            }
        }

        peak
    }

    #[test]
    fn lower_cutoff_attenuates_high_frequency_content_more() {
        let sample_rate = 48_000.0;
        let test_frequency = 5000.0;

        let mut low_cutoff = LadderFilter::new(sample_rate);
        low_cutoff.set_cutoff(500.0);

        let mut high_cutoff = LadderFilter::new(sample_rate);
        high_cutoff.set_cutoff(15_000.0);

        let low_peak = peak_amplitude(&mut low_cutoff, test_frequency, sample_rate);
        let high_peak = peak_amplitude(&mut high_cutoff, test_frequency, sample_rate);

        assert!(
            high_peak > low_peak,
            "expected the higher cutoff to pass more of a {test_frequency}Hz tone: low={low_peak}, high={high_peak}"
        );
    }

    #[test]
    fn higher_resonance_increases_peak_at_cutoff() {
        // 0.0 vs 1.0 (fully open) isn't a safe pair to compare here: at
        // this k-scaling, the peak at cutoff actually rises then falls
        // back down as resonance approaches 1.0 (checked numerically --
        // the heavy per-stage saturation kicks in and changes character
        // rather than just narrowing the passband further). 0.0 vs 0.5
        // sits solidly in the monotonically-increasing part of that curve
        let sample_rate = 48_000.0;
        let cutoff = 1000.0;

        let mut low_resonance = LadderFilter::new(sample_rate);
        low_resonance.set_cutoff(cutoff);
        low_resonance.set_resonance(0.0);

        let mut high_resonance = LadderFilter::new(sample_rate);
        high_resonance.set_cutoff(cutoff);
        high_resonance.set_resonance(0.5);

        let low_peak = peak_amplitude(&mut low_resonance, cutoff, sample_rate);
        let high_peak = peak_amplitude(&mut high_resonance, cutoff, sample_rate);

        assert!(
            high_peak > low_peak,
            "expected higher resonance to raise the peak at cutoff: low={low_peak}, high={high_peak}"
        );
    }

    #[test]
    fn each_stage_saturates_so_output_never_exceeds_unit_magnitude() {
        // regression test for the per-stage tanh fix: it saturates the
        // full new stage value, not just the per-sample delta, which is
        // what actually bounds the state -- an increment-only tanh would
        // still let the accumulated state drift past +-1 given a large
        // enough or long enough input, even though each individual step
        // looked bounded
        let mut filter = LadderFilter::new(48_000.0);

        filter.set_resonance(1.0);

        for i in 0..20_000 {
            let input = if i % 50 < 25 { 1000.0 } else { -1000.0 };
            let output = filter.process(input);

            assert!(output.is_finite(), "non-finite output: {output}");
            assert!(output.abs() <= 1.0, "output exceeded unit magnitude: {output}");
        }
    }
}
