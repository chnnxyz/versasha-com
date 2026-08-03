use super::types::{Frequency, Sample, SampleRate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
}

pub struct Filter {
    sample_rate: SampleRate,
    cutoff: Frequency,
    resonance: f32,
    // trapezoidal-integrator state (Cytomic/"TPT" SVF); unlike the naive
    // Chamberlin SVF this stays numerically stable across the whole cutoff
    // range instead of diverging to infinity as cutoff approaches Nyquist
    ic1eq: Sample,
    ic2eq: Sample,
    filter_type: FilterType,
}

impl Filter {
    pub fn new(rate: SampleRate) -> Self {
        Self {
            sample_rate: rate,
            cutoff: 2000.0,
            resonance: 0.0,
            ic1eq: 0.0,
            ic2eq: 0.0,
            filter_type: FilterType::LowPass,
        }
    }

    pub fn set_cutoff(&mut self, cutoff: Frequency) {
        self.cutoff = cutoff;
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.resonance = resonance.clamp(0.0, 1.0);
    }

    pub fn set_filter_type(&mut self, filter_type: FilterType) {
        self.filter_type = filter_type;
    }

    pub fn cutoff(&self) -> Frequency {
        self.cutoff
    }

    pub fn resonance(&self) -> f32 {
        self.resonance
    }

    pub fn filter_type(&self) -> FilterType {
        self.filter_type
    }

    pub fn reset(&mut self) {
        self.ic1eq = 0.0;
        self.ic2eq = 0.0;
    }

    // Topology-preserving-transform state-variable filter (Cytomic/Andrew
    // Simper's formulation). Derives low/high/band outputs from the same two
    // integrator states each sample, with resonance as feedback damping (low
    // damping = high resonance, near 0 approaches self-oscillation).
    pub fn process(&mut self, input: Sample) -> Sample {
        let nyquist = self.sample_rate * 0.5;
        let cutoff = self.cutoff.clamp(1.0, nyquist * 0.9);

        let g = (std::f32::consts::PI * cutoff / self.sample_rate).tan();
        let damping = (2.0 * (1.0 - self.resonance)).max(0.05);

        let a1 = 1.0 / (1.0 + g * (g + damping));
        let a2 = g * a1;
        let a3 = g * a2;

        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;

        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        let low = v2;
        let band = v1;
        let high = input - damping * band - low;

        match self.filter_type {
            FilterType::LowPass => low,
            FilterType::HighPass => high,
            FilterType::BandPass => band,
        }
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
        let filter = Filter::new(48_000.0);

        assert_eq!(filter.cutoff(), 2000.0);
        assert_eq!(filter.resonance(), 0.0);
        assert_eq!(filter.filter_type(), FilterType::LowPass);
    }

    #[test]
    fn cutoff_can_be_changed() {
        let mut filter = Filter::new(48_000.0);

        filter.set_cutoff(800.0);

        assert_eq!(filter.cutoff(), 800.0);
    }

    #[test]
    fn resonance_is_clamped() {
        let mut filter = Filter::new(48_000.0);

        filter.set_resonance(5.0);
        assert_approx_eq(filter.resonance(), 1.0);

        filter.set_resonance(-1.0);
        assert_approx_eq(filter.resonance(), 0.0);
    }

    #[test]
    fn filter_type_can_be_changed() {
        let mut filter = Filter::new(48_000.0);

        filter.set_filter_type(FilterType::HighPass);

        assert_eq!(filter.filter_type(), FilterType::HighPass);
    }

    #[test]
    fn reset_clears_state() {
        let mut filter = Filter::new(48_000.0);

        for _ in 0..100 {
            filter.process(1.0);
        }

        filter.reset();

        assert_approx_eq(filter.process(0.0), 0.0);
    }

    #[test]
    fn lowpass_tracks_constant_input() {
        let mut filter = Filter::new(48_000.0);

        filter.set_cutoff(500.0);

        let mut output = 0.0;

        for _ in 0..10_000 {
            output = filter.process(1.0);
        }

        assert_approx_eq(output, 1.0);
    }

    #[test]
    fn highpass_rejects_constant_input() {
        let mut filter = Filter::new(48_000.0);

        filter.set_cutoff(500.0);
        filter.set_filter_type(FilterType::HighPass);

        let mut output = 1.0;

        for _ in 0..10_000 {
            output = filter.process(1.0);
        }

        assert_approx_eq(output, 0.0);
    }

    #[test]
    fn bandpass_rejects_constant_input() {
        let mut filter = Filter::new(48_000.0);

        filter.set_cutoff(500.0);
        filter.set_filter_type(FilterType::BandPass);

        let mut output = 1.0;

        for _ in 0..10_000 {
            output = filter.process(1.0);
        }

        assert_approx_eq(output, 0.0);
    }

    fn peak_amplitude(filter: &mut Filter, frequency: Frequency, sample_rate: SampleRate) -> f32 {
        let mut peak: f32 = 0.0;

        for i in 0..4000 {
            let t = i as f32 / sample_rate;
            let input = (2.0 * std::f32::consts::PI * frequency * t).sin();
            let output = filter.process(input);

            if i > 3000 {
                peak = peak.max(output.abs());
            }
        }

        peak
    }

    #[test]
    fn higher_resonance_increases_peak_at_cutoff() {
        let sample_rate = 48_000.0;
        let cutoff = 1000.0;

        let mut low_resonance = Filter::new(sample_rate);
        low_resonance.set_cutoff(cutoff);
        low_resonance.set_filter_type(FilterType::BandPass);
        low_resonance.set_resonance(0.0);

        let mut high_resonance = Filter::new(sample_rate);
        high_resonance.set_cutoff(cutoff);
        high_resonance.set_filter_type(FilterType::BandPass);
        high_resonance.set_resonance(0.9);

        let low_peak = peak_amplitude(&mut low_resonance, cutoff, sample_rate);
        let high_peak = peak_amplitude(&mut high_resonance, cutoff, sample_rate);

        assert!(
            high_peak > low_peak,
            "expected higher resonance to raise the peak: low={low_peak}, high={high_peak}"
        );
    }

    #[test]
    fn output_stays_finite_at_high_resonance() {
        let mut filter = Filter::new(48_000.0);

        filter.set_cutoff(2000.0);
        filter.set_resonance(1.0);

        for filter_type in [FilterType::LowPass, FilterType::HighPass, FilterType::BandPass] {
            filter.set_filter_type(filter_type);
            filter.reset();

            for i in 0..20_000 {
                let input = if i % 50 < 25 { 1.0 } else { -1.0 };
                let sample = filter.process(input);

                assert!(sample.is_finite(), "non-finite output: {}", sample);
            }
        }
    }

    #[test]
    fn stays_finite_with_cutoff_maxed_out() {
        // regression test: the UI's cutoff knob tops out at 18kHz (48kHz
        // sample rate). A naive Chamberlin SVF previously diverged to
        // infinity within ~50 samples here, even at zero resonance.
        let sample_rate = 48_000.0;
        let cutoff = 18_000.0;

        for resonance in [0.0, 0.5, 1.0] {
            for filter_type in [FilterType::LowPass, FilterType::HighPass, FilterType::BandPass] {
                let mut filter = Filter::new(sample_rate);

                filter.set_cutoff(cutoff);
                filter.set_resonance(resonance);
                filter.set_filter_type(filter_type);

                let mut phase: f32 = 0.0;
                let phase_inc = cutoff / sample_rate;

                for _ in 0..sample_rate as usize {
                    let input = (2.0 * std::f32::consts::PI * phase).sin();
                    phase = (phase + phase_inc) % 1.0;

                    let sample = filter.process(input);

                    assert!(
                        sample.is_finite(),
                        "non-finite output at cutoff={cutoff} resonance={resonance} type={filter_type:?}"
                    );
                }
            }
        }
    }
}
