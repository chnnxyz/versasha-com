use crate::dsp::types::SampleRate;

use crate::sequencing::step_clock::StepClock;

pub struct Sequencer {
    clock: StepClock,
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
        Self {
            clock: StepClock::new(sample_rate, num_steps, bpm, steps_per_beat),
            pattern: vec![vec![false; num_tracks]; num_steps],
        }
    }

    // Getters
    pub fn bpm(&self) -> f32 {
        self.clock.bpm()
    }

    pub fn current_step(&self) -> usize {
        self.clock.current_step()
    }

    pub fn step_count(&self) -> usize {
        self.pattern.len()
    }

    // Setters

    pub fn set_bpm(&mut self, bpm: f32) {
        self.clock.set_bpm(bpm);
    }

    pub fn set_step(&mut self, step: usize, track: usize, active: bool) {
        self.pattern[step][track] = active;
    }

    pub fn active_tracks(&self) -> &[bool] {
        &self.pattern[self.clock.current_step()]
    }

    // looks up an arbitrary step's row, unlike active_tracks() which is
    // pinned to the clock's own current step -- unchecked, same
    // convention as set_step
    pub fn tracks_at(&self, step: usize) -> &[bool] {
        &self.pattern[step]
    }

    // Play stop etc

    pub fn advance(&mut self) -> bool {
        self.clock.advance()
    }

    pub fn stop(&mut self) {
        self.clock.stop()
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

    // full timing-correctness (samples-per-step math, steps-per-beat
    // derivation, the zero-guard, bpm recompute, wraparound, stop/reset) is
    // now StepClock's own responsibility and tested directly in
    // sequencing::step_clock -- this is just a thin smoke test proving
    // Sequencer's methods actually reach the clock, not a re-verification
    // of its math
    #[test]
    fn sequencer_methods_delegate_to_step_clock() {
        let mut sequencer = Sequencer::new(4.0, 4, 1, 60.0, Some(1));

        assert_eq!(sequencer.bpm(), 60.0);

        sequencer.set_bpm(120.0);
        assert_eq!(sequencer.bpm(), 120.0);

        // at 120 BPM / 1 step-per-beat / 4Hz, samples_per_step = 2.0
        assert!(!sequencer.advance());
        assert!(sequencer.advance());
        assert_eq!(sequencer.current_step(), 1);

        sequencer.stop();
        assert_eq!(sequencer.current_step(), 0);
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
    fn tracks_at_reads_an_arbitrary_step_regardless_of_current_step() {
        let mut sequencer = Sequencer::new(4.0, 4, 3, 60.0, Some(1));

        sequencer.set_step(0, 1, true);
        sequencer.set_step(2, 2, true);

        // clock never advances -- current_step stays 0 throughout, but
        // tracks_at should still reach step 2's row directly
        assert!(sequencer.tracks_at(0)[1]);
        assert!(!sequencer.tracks_at(0)[2]);
        assert!(sequencer.tracks_at(2)[2]);
        assert!(!sequencer.tracks_at(2)[1]);
    }

    #[test]
    fn tracks_at_matches_active_tracks_at_the_current_step() {
        let mut sequencer = Sequencer::new(4.0, 4, 2, 60.0, Some(1));

        sequencer.set_step(1, 0, true);

        for _ in 0..4 {
            sequencer.advance();
        }

        assert_eq!(sequencer.current_step(), 1);
        assert_eq!(sequencer.tracks_at(1), sequencer.active_tracks());
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
