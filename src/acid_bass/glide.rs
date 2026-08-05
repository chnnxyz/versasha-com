use crate::dsp::types::{DEFAULT_FREQ, Frequency, SampleRate, Time};

const MIN_GLIDE_TIME: Time = 0.0005;
const DEFAULT_GLIDE_TIME: Time = 0.06; // ~60ms, in the neighborhood of a real 303's slide

// Portamento: eases a played frequency toward a new target over `time`
// seconds instead of snapping to it instantly -- what makes a 303's
// "Slide" steps glide into the next note. Tracked in log2(frequency)
// space rather than raw Hz, since pitch is perceived logarithmically (a
// real slide circuit is linear in volts/octave, not volts/Hz); lerping
// raw Hz would glide unevenly across the range.
pub struct Glide {
    sample_rate: SampleRate,
    time: Time,

    current_log2: f32,
    target_log2: f32,
    step_log2: f32, // per-sample increment toward target_log2
}

impl Glide {
    pub fn new(rate: SampleRate) -> Self {
        Self {
            sample_rate: rate,
            time: DEFAULT_GLIDE_TIME,
            current_log2: DEFAULT_FREQ.log2(),
            target_log2: DEFAULT_FREQ.log2(),
            step_log2: 0.0,
        }
    }

    // getters
    pub fn time(&self) -> Time {
        self.time
    }

    pub fn current(&self) -> Frequency {
        2.0f32.powf(self.current_log2)
    }

    // setters
    pub fn set_time(&mut self, time: Time) {
        self.time = time.max(MIN_GLIDE_TIME);
    }

    // slide = false snaps straight to the target (a normal retrigger, no
    // glide); slide = true eases toward it over `self.time` seconds
    pub fn set_target(&mut self, target: Frequency, slide: bool) {
        if !slide {
            self.current_log2 = target.log2();
            self.target_log2 = target.log2();
            self.step_log2 = 0.0;
            return;
        }

        self.target_log2 = target.log2();
        self.step_log2 = (self.target_log2 - self.current_log2) / (self.time * self.sample_rate);
    }

    pub fn next_frequency(&mut self) -> Frequency {
        // clamp direction depends on which way we're gliding -- comparing
        // the step against target_log2 directly (rather than remaining
        // distance) would never actually catch the overshoot, since the
        // per-sample step is tiny next to an absolute log2-frequency value
        self.current_log2 = if self.step_log2 >= 0.0 {
            (self.current_log2 + self.step_log2).min(self.target_log2)
        } else {
            (self.current_log2 + self.step_log2).max(self.target_log2)
        };

        self.current()
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
        let glide = Glide::new(48_000.0);

        assert_approx_eq(glide.time(), DEFAULT_GLIDE_TIME);
        assert_approx_eq(glide.current(), DEFAULT_FREQ);
    }

    #[test]
    fn set_time_clamps_to_minimum() {
        let mut glide = Glide::new(48_000.0);

        glide.set_time(-1.0);

        assert_approx_eq(glide.time(), MIN_GLIDE_TIME);
    }

    #[test]
    fn set_target_without_slide_snaps_immediately() {
        let mut glide = Glide::new(48_000.0);

        glide.set_target(220.0, false);

        assert_approx_eq(glide.current(), 220.0);
        // no glide in progress -- the very next sample should still read
        // exactly the target, not ease toward it
        assert_approx_eq(glide.next_frequency(), 220.0);
    }

    #[test]
    fn set_target_with_slide_does_not_move_until_next_frequency_is_called() {
        let mut glide = Glide::new(4.0);

        glide.set_target(880.0, true);

        // set_target only records where to glide to -- it must not move
        // current() on its own
        assert_approx_eq(glide.current(), DEFAULT_FREQ);
    }

    #[test]
    fn slide_glides_up_in_even_log2_steps_and_settles_without_overshoot() {
        // rate = 4.0, time = 1.0 -> exactly 4 samples to cover the glide,
        // matching this codebase's convention of using tiny sample rates
        // so the math works out to exact, easily-checked call counts.
        // 880.0 is exactly one octave above the default 440.0, so the
        // total distance in log2 space is exactly 1.0 and each of the 4
        // steps should land on a predictable frequency.
        let mut glide = Glide::new(4.0);

        glide.set_time(1.0);
        glide.set_target(880.0, true);

        assert_approx_eq(glide.next_frequency(), 523.251); // +0.25 octave
        assert_approx_eq(glide.next_frequency(), 622.254); // +0.50 octave
        assert_approx_eq(glide.next_frequency(), 739.989); // +0.75 octave
        assert_approx_eq(glide.next_frequency(), 880.0); // arrived

        // regression check for the overshoot bug: continuing to call
        // next_frequency() after arriving must keep returning the target,
        // not keep climbing past it
        for _ in 0..5 {
            assert_approx_eq(glide.next_frequency(), 880.0);
        }
    }

    #[test]
    fn slide_glides_down_and_settles_without_undershoot() {
        let mut glide = Glide::new(4.0);

        glide.set_time(1.0);
        glide.set_target(220.0, true); // one octave below the default 440.0

        for _ in 0..4 {
            glide.next_frequency();
        }

        assert_approx_eq(glide.current(), 220.0);

        // same overshoot regression check, but gliding downward -- this
        // is the branch that a step_log2 >= 0.0-only clamp would miss
        for _ in 0..5 {
            assert_approx_eq(glide.next_frequency(), 220.0);
        }
    }

    #[test]
    fn retriggering_without_slide_mid_glide_cancels_the_glide() {
        let mut glide = Glide::new(4.0);

        glide.set_time(1.0);
        glide.set_target(880.0, true);

        glide.next_frequency(); // partway through the glide, not yet arrived

        glide.set_target(330.0, false);

        assert_approx_eq(glide.current(), 330.0);
        assert_approx_eq(glide.next_frequency(), 330.0);
    }
}
