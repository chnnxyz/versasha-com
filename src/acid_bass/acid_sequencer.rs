use crate::acid_bass::acid_step::AcidStep;
use crate::dsp::types::SampleRate;
use crate::sequencing::step_clock::StepClock;

// Monophonic step Sequencer, since shares step clock with drums,
// uses a single active/inactive vector.
pub struct AcidSequencer {
    clock: StepClock,
    steps: Vec<AcidStep>,
}

impl AcidSequencer {
    pub fn new(
        sample_rate: SampleRate,
        num_steps: usize,
        bpm: f32,
        steps_per_beat: Option<usize>,
    ) -> Self {
        Self {
            clock: StepClock::new(sample_rate, num_steps, bpm, steps_per_beat),
            steps: vec![AcidStep::default(); num_steps],
        }
    }

    // getters
    pub fn bpm(&self) -> f32 {
        self.clock.bpm()
    }

    pub fn current_step(&self) -> usize {
        self.clock.current_step()
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn step(&self, index: usize) -> &AcidStep {
        &self.steps[index]
    }

    pub fn active_step(&self) -> &AcidStep {
        self.step(self.current_step())
    }

    // setters
    pub fn set_bpm(&mut self, bpm: f32) {
        self.clock.set_bpm(bpm);
    }

    pub fn set_step(&mut self, index: usize, step: AcidStep) {
        self.steps[index] = step;
    }

    pub fn clear_all_steps(&mut self) {
        self.steps = vec![AcidStep::default(); self.steps.len()];
    }

    // play/stop
    pub fn advance(&mut self) -> bool {
        self.clock.advance()
    }

    pub fn stop(&mut self) {
        self.clock.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // full timing-correctness (samples-per-step math, wraparound,
    // stop/reset) is StepClock's own responsibility, tested directly in
    // sequencing::step_clock -- this is just a thin smoke test proving
    // AcidSequencer's methods actually reach the clock, same convention
    // as Sequencer's own delegation test
    #[test]
    fn sequencer_methods_delegate_to_step_clock() {
        let mut sequencer = AcidSequencer::new(4.0, 4, 60.0, Some(1));

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
    fn new_fills_every_step_with_defaults() {
        let sequencer = AcidSequencer::new(4.0, 8, 60.0, None);

        assert_eq!(sequencer.step_count(), 8);

        for index in 0..sequencer.step_count() {
            let step = sequencer.step(index);

            assert!(!step.gate());
            assert!(!step.accent());
            assert!(!step.slide());
        }
    }

    #[test]
    fn set_step_and_step_use_consistent_indexing() {
        let mut sequencer = AcidSequencer::new(4.0, 4, 60.0, Some(1));

        let mut step = AcidStep::new();
        step.set_note(110.0);
        step.set_gate(true);

        sequencer.set_step(2, step);

        assert_eq!(sequencer.step(2).note(), 110.0);
        assert!(sequencer.step(2).gate());

        // untouched neighbors stay default
        assert!(!sequencer.step(0).gate());
        assert!(!sequencer.step(1).gate());
        assert!(!sequencer.step(3).gate());
    }

    #[test]
    fn active_step_reflects_current_step() {
        let mut sequencer = AcidSequencer::new(4.0, 4, 60.0, Some(1));

        let mut step0 = AcidStep::new();
        step0.set_note(110.0);
        sequencer.set_step(0, step0);

        let mut step1 = AcidStep::new();
        step1.set_note(220.0);
        sequencer.set_step(1, step1);

        assert_eq!(sequencer.active_step().note(), 110.0);

        for _ in 0..4 {
            sequencer.advance();
        }

        assert_eq!(sequencer.current_step(), 1);
        assert_eq!(sequencer.active_step().note(), 220.0);
    }

    #[test]
    fn clear_all_steps_resets_every_step_to_default() {
        let mut sequencer = AcidSequencer::new(4.0, 4, 60.0, Some(1));

        let mut step = AcidStep::new();
        step.set_gate(true);
        step.set_accent(true);

        sequencer.set_step(0, step);
        sequencer.set_step(3, step);

        sequencer.clear_all_steps();

        for index in 0..sequencer.step_count() {
            let step = sequencer.step(index);

            assert!(!step.gate());
            assert!(!step.accent());
            assert!(!step.slide());
        }
    }
}
