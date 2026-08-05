use crate::dsp::types::SampleRate;
use crate::sequencing::step_clock::StepClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequencerStatus {
    Play,
    Pause,
    Stop,
}

// One single transport layer  that does not derive clone or copy so all
// sequenced instruments refer to the same instance.
pub struct Transport {
    status: SequencerStatus,
    clock: StepClock,
}

impl Transport {
    pub fn new(
        sample_rate: SampleRate,
        num_steps: usize,
        bpm: f32,
        steps_per_beat: Option<usize>,
    ) -> Self {
        Self {
            status: SequencerStatus::Stop,
            clock: StepClock::new(sample_rate, num_steps, bpm, steps_per_beat),
        }
    }

    // getters
    pub fn status(&self) -> SequencerStatus {
        self.status
    }

    pub fn bpm(&self) -> f32 {
        self.clock.bpm()
    }

    pub fn current_step(&self) -> usize {
        self.clock.current_step()
    }

    // setters / playback controls
    pub fn set_bpm(&mut self, bpm: f32) {
        self.clock.set_bpm(bpm);
    }

    pub fn play(&mut self) {
        self.status = SequencerStatus::Play;
    }

    pub fn pause(&mut self) {
        self.status = SequencerStatus::Pause;
    }

    pub fn stop(&mut self) {
        self.status = SequencerStatus::Stop;
        self.clock.stop();
    }

    // Advances the clock by one sample and reports whether a step
    // boundary was just crossed -- but only actually advances while
    // status == Play (same "only new triggers are gated on Play" rule
    // DrumMachine::next_sample used to enforce internally; while
    // paused/stopped this should just return false without touching the
    // clock). Whoever owns this Transport is responsible for calling it
    // once per sample and, when it returns true, fanning
    // trigger_step(current_step()) out to every attached instrument.
    pub fn advance(&mut self) -> bool {
        if self.status == SequencerStatus::Play {
            return self.clock.advance();
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_defaults() {
        let transport = Transport::new(4.0, 4, 60.0, Some(1));

        assert_eq!(transport.status(), SequencerStatus::Stop);
        assert_eq!(transport.bpm(), 60.0);
        assert_eq!(transport.current_step(), 0);
    }

    #[test]
    fn set_bpm_delegates_to_clock() {
        let mut transport = Transport::new(4.0, 4, 60.0, Some(1));

        transport.set_bpm(120.0);

        assert_eq!(transport.bpm(), 120.0);
    }

    #[test]
    fn play_pause_stop_update_status() {
        let mut transport = Transport::new(4.0, 4, 60.0, Some(1));

        assert_eq!(transport.status(), SequencerStatus::Stop);

        transport.play();
        assert_eq!(transport.status(), SequencerStatus::Play);

        transport.pause();
        assert_eq!(transport.status(), SequencerStatus::Pause);

        transport.stop();
        assert_eq!(transport.status(), SequencerStatus::Stop);
    }

    #[test]
    fn advance_does_nothing_while_stopped() {
        let mut transport = Transport::new(4.0, 4, 60.0, Some(1));

        // default status is Stop -- never played
        for _ in 0..10 {
            assert!(!transport.advance());
        }

        assert_eq!(transport.current_step(), 0);
    }

    #[test]
    fn advance_moves_the_clock_only_while_playing() {
        let mut transport = Transport::new(4.0, 4, 60.0, Some(1));

        transport.play();

        // at 60 BPM / 1 step-per-beat / 4Hz, samples_per_step = 4.0
        assert!(!transport.advance());
        assert!(!transport.advance());
        assert!(!transport.advance());
        assert!(transport.advance());

        assert_eq!(transport.current_step(), 1);
    }

    #[test]
    fn advance_does_nothing_while_paused() {
        let mut transport = Transport::new(4.0, 4, 60.0, Some(1));

        transport.play();
        transport.advance();
        transport.advance();

        transport.pause();

        // paused mid-step -- further advance() calls must not move
        // anything, not even the fractional step_phase underneath
        for _ in 0..10 {
            assert!(!transport.advance());
        }

        assert_eq!(transport.current_step(), 0);

        // resuming should pick up exactly where it left off (2 of the 4
        // samples already accumulated), not restart from scratch
        transport.play();
        assert!(!transport.advance());
        assert!(transport.advance());

        assert_eq!(transport.current_step(), 1);
    }

    #[test]
    fn stop_resets_position_but_pause_does_not() {
        let mut transport = Transport::new(4.0, 4, 60.0, Some(1));

        transport.play();
        transport.advance();
        transport.advance();
        transport.advance();
        transport.advance(); // crosses into step 1

        assert_eq!(transport.current_step(), 1);

        transport.pause();
        assert_eq!(transport.current_step(), 1); // untouched by pause

        transport.stop();
        assert_eq!(transport.current_step(), 0); // reset by stop
    }

    #[test]
    fn advance_returns_true_only_on_the_boundary_crossing_call() {
        let mut transport = Transport::new(4.0, 4, 60.0, Some(1));

        transport.play();

        assert!(!transport.advance());
        assert!(!transport.advance());
        assert!(!transport.advance());
        assert!(transport.advance());
        assert!(!transport.advance());
    }
}
