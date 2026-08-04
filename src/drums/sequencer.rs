use crate::dsp::types::SampleRate;

pub struct Sequencer {
    bpm: f32,
    sample_rate: SampleRate,
    steps_per_beat: usize,
    samples_per_step: f32,
    step_phase: f32,
    current_step: usize,
    pattern: Vec<Vec<bool>>,
}

impl Sequencer {
    pub fn new(
        sample_rate: SampleRate,
        num_steps: usize,
        num_tracks: usize,
        bpm: f32,
        steps_per_beat: Option<usize>,
    ) -> Self {
        let steps_per_beat = steps_per_beat.unwrap_or(num_steps / 4).max(1);

        Self {
            sample_rate,
            bpm,
            steps_per_beat,
            samples_per_step: Self::compute_samples_per_step(bpm, steps_per_beat, sample_rate),
            step_phase: 0.0,
            current_step: 0,
            pattern: vec![vec![false; num_tracks]; num_steps],
        }
    }

    fn compute_samples_per_step(bpm: f32, steps_per_beat: usize, sample_rate: SampleRate) -> f32 {
        let seconds_per_beat = 60.0 / bpm;
        let seconds_per_step = seconds_per_beat / steps_per_beat as f32;

        seconds_per_step * sample_rate
    }

    // Getters
    pub fn bpm(&self) -> f32 {
        self.bpm
    }

    pub fn current_step(&self) -> usize {
        self.current_step
    }

    pub fn step_count(&self) -> usize {
        self.pattern.len()
    }

    // Setters

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
        self.samples_per_step =
            Self::compute_samples_per_step(bpm, self.steps_per_beat, self.sample_rate);
    }

    pub fn set_step(&mut self, step: usize, track: usize, active: bool) {
        self.pattern[step][track] = active;
    }

    pub fn active_tracks(&self) -> &[bool] {
        &self.pattern[self.current_step]
    }

    // Play stop etc
    pub fn advance(&mut self) -> bool {
        self.step_phase += 1.0 / self.samples_per_step;
        if self.step_phase >= 1.0 {
            self.step_phase -= 1.0;
            self.current_step = (self.current_step + 1) % self.pattern.len();
            return true;
        }
        false
    }

    pub fn stop(&mut self) {
        self.current_step = 0;
        self.step_phase = 0.0;
    }

    pub fn clear_one_track(&mut self, track: usize) {
        for step in self.pattern.iter_mut() {
            step[track] = false;
        }
    }

    pub fn clear_all_tracks(&mut self) {
        self.pattern = vec![vec![false; self.pattern[0].len()]; self.pattern.len()]
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
        let mut sequencer = Sequencer::new(48_000.0, 16, 1, 120.0, None);

        let mut step_changes = 0;

        for _ in 0..6_010 {
            if sequencer.advance() {
                step_changes += 1;
            }
        }

        assert_eq!(step_changes, 1);
    }

    #[test]
    fn new_derives_steps_per_beat_when_not_provided() {
        // 16 steps, no explicit steps_per_beat -> derived as 16 / 4 = 4,
        // same one-bar-of-4/4 assumption as the test above
        let mut sequencer = Sequencer::new(16.0, 16, 1, 60.0, None);

        for _ in 0..3 {
            assert!(!sequencer.advance());
        }

        assert!(sequencer.advance());
    }

    #[test]
    fn new_treats_explicit_zero_steps_per_beat_as_one() {
        // regression test: steps_per_beat=Some(0) must not divide by zero
        let mut sequencer = Sequencer::new(4.0, 4, 1, 60.0, Some(0));

        for _ in 0..3 {
            assert!(!sequencer.advance());
        }

        assert!(sequencer.advance());
    }

    #[test]
    fn set_bpm_recomputes_step_timing() {
        let mut sequencer = Sequencer::new(4.0, 4, 1, 60.0, Some(1));

        // at 60 BPM, samples_per_step = 4.0
        assert!(!sequencer.advance());
        assert!(!sequencer.advance());
        assert!(!sequencer.advance());
        assert!(sequencer.advance());

        sequencer.set_bpm(120.0);

        // at 120 BPM, samples_per_step should now be 2.0 -- if set_bpm
        // didn't actually recompute the clock, this would still take 4 calls
        assert!(!sequencer.advance());
        assert!(sequencer.advance());
    }

    #[test]
    fn set_step_and_active_tracks_use_consistent_indexing() {
        // 16 steps but only 11 tracks: step 12 exceeds the track count,
        // which would have panicked under the earlier [track][step] bug
        let mut sequencer = Sequencer::new(16.0, 16, 11, 60.0, Some(4));

        sequencer.set_step(12, 3, true);

        for _ in 0..48 {
            sequencer.advance();
        }

        assert_eq!(sequencer.current_step(), 12);

        assert!(sequencer.active_tracks()[3]);
        assert!(!sequencer.active_tracks()[0]);
    }

    #[test]
    fn active_tracks_reflects_current_step() {
        let mut sequencer = Sequencer::new(4.0, 4, 3, 60.0, Some(1));

        sequencer.set_step(0, 1, true);
        sequencer.set_step(1, 2, true);

        assert!(!sequencer.active_tracks()[0]);
        assert!(sequencer.active_tracks()[1]);
        assert!(!sequencer.active_tracks()[2]);

        for _ in 0..4 {
            sequencer.advance();
        }

        assert_eq!(sequencer.current_step(), 1);

        assert!(!sequencer.active_tracks()[0]);
        assert!(!sequencer.active_tracks()[1]);
        assert!(sequencer.active_tracks()[2]);
    }

    #[test]
    fn advance_wraps_current_step_using_step_count_not_track_count() {
        // 2 steps, 5 tracks -- deliberately different counts to catch
        // wrapping against the wrong dimension
        let mut sequencer = Sequencer::new(4.0, 2, 5, 60.0, Some(1));

        for _ in 0..4 {
            sequencer.advance();
        }

        assert_eq!(sequencer.current_step(), 1);

        for _ in 0..4 {
            sequencer.advance();
        }

        assert_eq!(sequencer.current_step(), 0);
    }

    #[test]
    fn stop_resets_both_current_step_and_phase() {
        let mut sequencer = Sequencer::new(4.0, 4, 3, 60.0, Some(1));

        sequencer.advance();
        sequencer.advance();

        sequencer.stop();

        assert_eq!(sequencer.current_step(), 0);

        // if step_phase weren't also reset, this would wrap after 2 more
        // calls (0.5 carried over + 0.25 + 0.25) instead of a fresh 4
        assert!(!sequencer.advance());
        assert!(!sequencer.advance());
        assert!(!sequencer.advance());
        assert!(sequencer.advance());
    }

    #[test]
    fn clear_one_track_clears_across_all_steps_not_one_step() {
        let mut sequencer = Sequencer::new(4.0, 3, 2, 60.0, Some(1));

        sequencer.set_step(0, 0, true);
        sequencer.set_step(0, 1, true);
        sequencer.set_step(1, 0, true);
        sequencer.set_step(2, 0, true);
        sequencer.set_step(2, 1, true);

        sequencer.clear_one_track(0);

        // step 0: track 0 cleared, track 1 untouched
        assert!(!sequencer.active_tracks()[0]);
        assert!(sequencer.active_tracks()[1]);

        for _ in 0..4 {
            sequencer.advance();
        }

        // step 1: track 0 cleared, track 1 was never set
        assert_eq!(sequencer.current_step(), 1);
        assert!(!sequencer.active_tracks()[0]);
        assert!(!sequencer.active_tracks()[1]);

        for _ in 0..4 {
            sequencer.advance();
        }

        // step 2: track 0 cleared, track 1 still untouched
        assert_eq!(sequencer.current_step(), 2);
        assert!(!sequencer.active_tracks()[0]);
        assert!(sequencer.active_tracks()[1]);
    }

    #[test]
    fn clear_all_tracks_resets_entire_pattern() {
        let mut sequencer = Sequencer::new(4.0, 2, 2, 60.0, Some(1));

        sequencer.set_step(0, 0, true);
        sequencer.set_step(1, 1, true);

        sequencer.clear_all_tracks();

        assert!(!sequencer.active_tracks()[0]);
        assert!(!sequencer.active_tracks()[1]);

        for _ in 0..4 {
            sequencer.advance();
        }

        assert!(!sequencer.active_tracks()[0]);
        assert!(!sequencer.active_tracks()[1]);
    }
}
