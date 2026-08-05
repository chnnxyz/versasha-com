use crate::dsp::types::SampleRate;

pub struct StepClock {
    bpm: f32,
    sample_rate: SampleRate,
    steps_per_beat: usize,
    samples_per_step: f32,
    step_phase: f32,
    current_step: usize,
    num_steps: usize,
}

impl StepClock {
    fn compute_samples_per_step(bpm: f32, steps_per_beat: usize, sample_rate: SampleRate) -> f32 {
        let seconds_per_beat = 60.0 / bpm;
        let seconds_per_step = seconds_per_beat / steps_per_beat as f32;

        seconds_per_step * sample_rate
    }

    pub fn new(
        sample_rate: SampleRate,
        num_steps: usize,
        bpm: f32,
        steps_per_beat: Option<usize>,
    ) -> Self {
        let steps_per_beat = steps_per_beat.unwrap_or(num_steps / 4).max(1);

        Self {
            sample_rate,
            num_steps,
            bpm,
            steps_per_beat,
            samples_per_step: Self::compute_samples_per_step(bpm, steps_per_beat, sample_rate),
            step_phase: 0.0,
            current_step: 0,
        }
    }

    // getters
    pub fn bpm(&self) -> f32 {
        self.bpm
    }

    pub fn current_step(&self) -> usize {
        self.current_step
    }

    // setters
    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
        self.samples_per_step =
            Self::compute_samples_per_step(bpm, self.steps_per_beat, self.sample_rate);
    }

    // Play stop
    pub fn advance(&mut self) -> bool {
        self.step_phase += 1.0 / self.samples_per_step;
        if self.step_phase >= 1.0 {
            self.step_phase -= 1.0;
            self.current_step = (self.current_step + 1) % self.num_steps;
            return true;
        }
        false
    }

    pub fn stop(&mut self) {
        self.current_step = 0;
        self.step_phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_computes_samples_per_step_from_bpm_and_subdivision() {
        // 120 BPM, 48kHz, 16-step pattern (4 steps per beat / 16th notes):
        // seconds_per_beat = 0.5, seconds_per_step = 0.125, samples_per_step = 6000.0.
        // counting crossings over a padded window rather than asserting an exact
        // call index: 1/6000 isn't exactly representable in f32, and measuring
        // the real accumulation showed the crossing lands on call 6001, not
        // 6000, so a tight 6000-call window is one call too short
        let mut clock = StepClock::new(48_000.0, 16, 120.0, None);

        let mut step_changes = 0;

        for _ in 0..6_010 {
            if clock.advance() {
                step_changes += 1;
            }
        }

        assert_eq!(step_changes, 1);
    }

    #[test]
    fn new_derives_steps_per_beat_when_not_provided() {
        // 16 steps, no explicit steps_per_beat -> derived as 16 / 4 = 4,
        // same one-bar-of-4/4 assumption as the test above
        let mut clock = StepClock::new(16.0, 16, 60.0, None);

        for _ in 0..3 {
            assert!(!clock.advance());
        }

        assert!(clock.advance());
    }

    #[test]
    fn new_treats_explicit_zero_steps_per_beat_as_one() {
        // regression test: steps_per_beat=Some(0) must not divide by zero
        let mut clock = StepClock::new(4.0, 4, 60.0, Some(0));

        for _ in 0..3 {
            assert!(!clock.advance());
        }

        assert!(clock.advance());
    }

    #[test]
    fn set_bpm_recomputes_step_timing() {
        let mut clock = StepClock::new(4.0, 4, 60.0, Some(1));

        // at 60 BPM, samples_per_step = 4.0
        assert!(!clock.advance());
        assert!(!clock.advance());
        assert!(!clock.advance());
        assert!(clock.advance());

        clock.set_bpm(120.0);

        // at 120 BPM, samples_per_step should now be 2.0 -- if set_bpm
        // didn't actually recompute the clock, this would still take 4 calls
        assert!(!clock.advance());
        assert!(clock.advance());
    }

    #[test]
    fn advance_wraps_current_step_at_num_steps_boundary() {
        // num_steps is the clock's only notion of size -- unlike the old
        // Sequencer-owned wraparound, there's no separate "track count"
        // dimension left to confuse it with
        let mut clock = StepClock::new(4.0, 2, 60.0, Some(1));

        for _ in 0..4 {
            clock.advance();
        }

        assert_eq!(clock.current_step(), 1);

        for _ in 0..4 {
            clock.advance();
        }

        assert_eq!(clock.current_step(), 0);
    }

    #[test]
    fn stop_resets_both_current_step_and_phase() {
        let mut clock = StepClock::new(4.0, 4, 60.0, Some(1));

        clock.advance();
        clock.advance();

        clock.stop();

        assert_eq!(clock.current_step(), 0);

        // if step_phase weren't also reset, this would wrap after 2 more
        // calls (0.5 carried over + 0.25 + 0.25) instead of a fresh 4
        assert!(!clock.advance());
        assert!(!clock.advance());
        assert!(!clock.advance());
        assert!(clock.advance());
    }
}
