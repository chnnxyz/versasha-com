use crate::dsp::types::{DryWet, Sample, SampleRate};

const COMB_COUNT: usize = 4;
const ALLPASS_COUNT: usize = 2;

// classic Freeverb-style tuning (samples @ 44.1kHz) -- scaled to
// whatever sample_rate this Reverb is actually built at in new(). No
// two lengths equal or small integer multiples of each other, so their
// resonant peaks don't reinforce into one audible pitch instead of a
// smooth wash.
const COMB_LENGTHS_AT_44100: [usize; COMB_COUNT] = [1557, 1617, 1422, 1277];
const ALLPASS_LENGTHS_AT_44100: [usize; ALLPASS_COUNT] = [556, 441];

// fixed diffusion coefficient for every allpass stage -- unlike decay,
// this isn't user-controllable
const ALLPASS_COEFFICIENT: f32 = 0.5;

// leans toward the lush, spacious hall character a Pioneer-style Beat
// FX reverb reaches for on a drop/transition, rather than a small,
// tight room -- comfortably short of MAX_DECAY's runaway territory
const DEFAULT_SIZE: f32 = 0.65;
const DEFAULT_DECAY: f32 = 0.7;
const DEFAULT_MIX: DryWet = 0.3;

// decay is each comb's own feedback coefficient (see process()) --
// clamped below 1.0 the same way Delay::MAX_FEEDBACK is, since >=1.0
// feedback never settles and just gets louder forever
const MAX_DECAY: f32 = 0.98;

// A simple algorithmic reverb -- comb filters in parallel feeding into
// allpass filters in series (the classic Schroeder/Freeverb shape), not
// a convolution reverb. size/decay control the comb stage's feedback
// and delay lengths; mix blends the result with the input, same
// convention as Delay's own `mix` in dsp/fx/delay.rs.
pub struct Reverb {
    comb_buffers: Vec<Vec<Sample>>,
    comb_indices: Vec<usize>,
    allpass_buffers: Vec<Vec<Sample>>,
    allpass_indices: Vec<usize>,

    size: f32,
    decay: f32,
    mix: DryWet,
}

impl Reverb {
    pub fn new(sample_rate: SampleRate) -> Self {
        // scale the 44.1kHz-tuned lengths so they land on the same
        // real-world millisecond values at whatever rate this runs at
        let scale = sample_rate / 44_100.0;

        let comb_buffers = COMB_LENGTHS_AT_44100
            .iter()
            .map(|&length| vec![0.0; ((length as f32) * scale).max(1.0) as usize])
            .collect();

        let allpass_buffers = ALLPASS_LENGTHS_AT_44100
            .iter()
            .map(|&length| vec![0.0; ((length as f32) * scale).max(1.0) as usize])
            .collect();

        Self {
            comb_buffers,
            comb_indices: vec![0; COMB_COUNT],
            allpass_buffers,
            allpass_indices: vec![0; ALLPASS_COUNT],
            size: DEFAULT_SIZE,
            decay: DEFAULT_DECAY,
            mix: DEFAULT_MIX,
        }
    }

    // getters
    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn decay(&self) -> f32 {
        self.decay
    }

    pub fn mix(&self) -> DryWet {
        self.mix
    }

    // setters -- clamp the same way every other 0..1-ish knob in this
    // codebase does (see Delay::set_feedback/set_mix for the pattern)
    pub fn set_size(&mut self, size: f32) {
        self.size = size.clamp(0.0, 1.0);
    }

    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay.clamp(0.0, MAX_DECAY);
    }

    pub fn set_mix(&mut self, mix: DryWet) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    pub fn reset(&mut self) {
        for buffer in self.comb_buffers.iter_mut() {
            buffer.iter_mut().for_each(|sample| *sample = 0.0);
        }
        self.comb_indices.iter_mut().for_each(|index| *index = 0);

        for buffer in self.allpass_buffers.iter_mut() {
            buffer.iter_mut().for_each(|sample| *sample = 0.0);
        }
        self.allpass_indices.iter_mut().for_each(|index| *index = 0);
    }

    pub fn process(&mut self, input: Sample) -> Sample {
        // Each comb is allocated at its own generous fixed capacity in
        // new() and never resized -- `size` instead scales how far back
        // into that capacity this reads, the same relationship
        // Delay::set_time has to Delay::delay_samples(). This is
        // exactly Delay::process's own read/write shape
        // (`read_index = (write_index + len - delay_samples) % len`),
        // just run once per comb, with `decay` standing in for
        // feedback, and every comb reading/writing its own buffer at
        // its own index -- they never feed each other, only the same
        // dry `input`.
        let mut sum = 0.0;

        for i in 0..self.comb_buffers.len() {
            let buffer = &mut self.comb_buffers[i];
            let length = buffer.len();
            let reach = ((length as f32) * self.size).max(1.0) as usize;
            let read_index = (self.comb_indices[i] + length - reach) % length;

            let delayed = buffer[read_index];
            buffer[self.comb_indices[i]] = input + delayed * self.decay;
            self.comb_indices[i] = (self.comb_indices[i] + 1) % length;

            sum += delayed;
        }

        // averaging (not summing) keeps the overall level roughly
        // constant regardless of how many combs there are
        let mut diffused = sum / self.comb_buffers.len() as f32;

        // each allpass stage feeds the previous stage's output in as
        // its own input -- this specific feedback/feedforward shape
        // (as opposed to the comb stage above) is what makes it
        // "allpass": flat frequency response, so it smears energy in
        // time without coloring it, in contrast to the comb stage
        // above which deliberately does color the signal
        for i in 0..self.allpass_buffers.len() {
            let buffer = &mut self.allpass_buffers[i];
            let length = buffer.len();
            let index = self.allpass_indices[i];

            let buffered = buffer[index];
            let feedback_in = diffused - ALLPASS_COEFFICIENT * buffered;
            buffer[index] = feedback_in;
            diffused = buffered + ALLPASS_COEFFICIENT * feedback_in;

            self.allpass_indices[i] = (index + 1) % length;
        }

        input * (1.0 - self.mix) + diffused * self.mix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_reverb_has_expected_defaults() {
        let reverb = Reverb::new(48_000.0);

        assert_eq!(reverb.size(), DEFAULT_SIZE);
        assert_eq!(reverb.decay(), DEFAULT_DECAY);
        assert_eq!(reverb.mix(), DEFAULT_MIX);
    }

    #[test]
    fn set_size_is_clamped() {
        let mut reverb = Reverb::new(48_000.0);

        reverb.set_size(5.0);
        assert_eq!(reverb.size(), 1.0);

        reverb.set_size(-5.0);
        assert_eq!(reverb.size(), 0.0);
    }

    #[test]
    fn set_decay_is_clamped_below_one() {
        let mut reverb = Reverb::new(48_000.0);

        reverb.set_decay(5.0);
        assert_eq!(reverb.decay(), MAX_DECAY);

        reverb.set_decay(-5.0);
        assert_eq!(reverb.decay(), 0.0);
    }

    #[test]
    fn set_mix_is_clamped() {
        let mut reverb = Reverb::new(48_000.0);

        reverb.set_mix(5.0);
        assert_eq!(reverb.mix(), 1.0);

        reverb.set_mix(-5.0);
        assert_eq!(reverb.mix(), 0.0);
    }

    #[test]
    fn dry_only_passes_input_through_when_mix_is_zero() {
        let mut reverb = Reverb::new(48_000.0);
        reverb.set_mix(0.0);

        assert_eq!(reverb.process(0.5), 0.5);
        assert_eq!(reverb.process(-0.25), -0.25);
    }

    #[test]
    fn higher_decay_sustains_the_tail_longer() {
        // the allpass stage has its own internal feedback loop, so it
        // sustains some tail on its own regardless of the comb decay --
        // isolating decay's actual effect means comparing two decay
        // values against each other, not checking either one in
        // isolation against an absolute silence threshold
        let mut short = Reverb::new(48_000.0);
        short.set_mix(1.0);
        short.set_decay(0.1);

        let mut long = Reverb::new(48_000.0);
        long.set_mix(1.0);
        long.set_decay(0.95);

        short.process(1.0);
        long.process(1.0);

        for _ in 0..6000 {
            short.process(0.0);
            long.process(0.0);
        }

        let short_energy: f32 = (0..1000).map(|_| short.process(0.0).abs()).sum();
        let long_energy: f32 = (0..1000).map(|_| long.process(0.0).abs()).sum();

        assert!(
            long_energy > short_energy,
            "expected higher decay to sustain more tail energy than lower decay: long={}, short={}",
            long_energy,
            short_energy
        );
    }

    #[test]
    fn reset_clears_the_tail() {
        let mut reverb = Reverb::new(48_000.0);
        reverb.set_mix(1.0);
        reverb.set_decay(0.9);

        for _ in 0..1000 {
            reverb.process(1.0);
        }

        reverb.reset();

        // a linear system fed nothing but zeros after a full reset can
        // only ever output zero -- proves reset() actually zeroed every
        // comb/allpass buffer, not just the ones it happened to touch
        for _ in 0..500 {
            assert_eq!(reverb.process(0.0), 0.0);
        }
    }

    #[test]
    fn set_size_changes_the_output() {
        let mut small = Reverb::new(48_000.0);
        small.set_size(0.1);
        small.set_mix(1.0);

        let mut large = Reverb::new(48_000.0);
        large.set_size(0.9);
        large.set_mix(1.0);

        small.process(1.0);
        large.process(1.0);

        let differs = (0..2000).any(|_| small.process(0.0) != large.process(0.0));

        assert!(differs, "expected different `size` values to produce a different reverb tail");
    }

    #[test]
    fn output_stays_finite_with_high_decay() {
        let mut reverb = Reverb::new(48_000.0);
        reverb.set_decay(MAX_DECAY);
        reverb.set_mix(0.5);

        for i in 0..100_000 {
            let input = if i % 200 < 100 { 1.0 } else { -1.0 };
            let sample = reverb.process(input);

            assert!(sample.is_finite(), "non-finite output: {}", sample);
        }
    }
}
