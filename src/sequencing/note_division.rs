use crate::dsp::types::Time;

// A "pick a beat fraction, we derive real units from BPM" control --
// shared by anything whose rate should sync to tempo instead of running
// off a raw Hz/seconds knob: Arp's own step rate (arp/arp.rs) and the
// mixer's Beat FX rate (mixer/effects_unit.rs) both use this same enum
// rather than each defining their own near-identical one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteDivision {
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
}

impl NoteDivision {
    // How many of this division fit in one beat (one quarter note) --
    // the integer form StepClock::new's steps_per_beat wants, as
    // opposed to seconds()'s continuous-time form below.
    pub fn steps_per_beat(&self) -> usize {
        match self {
            Self::Quarter => 1,
            Self::Eighth => 2,
            Self::Sixteenth => 4,
            Self::ThirtySecond => 8,
        }
    }

    // Feeds straight into Delay::set_time (dsp/fx/delay.rs) for the
    // Beat FX delay. Other consumers want a related but not identical
    // conversion -- a flanger's sweep rate is usually Hz, not a
    // one-shot time -- so work those out at each call site rather than
    // bending this method to fit every caller.
    pub fn seconds(&self, bpm: f32) -> Time {
        // guards against div-by-zero/negative BPM producing an
        // infinite or negative time
        let bpm = bpm.max(0.001);

        60.0 / bpm / self.steps_per_beat() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-6;

    fn assert_approx_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "expected {}, got {}",
            expected,
            actual
        );
    }

    #[test]
    fn steps_per_beat_doubles_each_division() {
        assert_eq!(NoteDivision::Quarter.steps_per_beat(), 1);
        assert_eq!(NoteDivision::Eighth.steps_per_beat(), 2);
        assert_eq!(NoteDivision::Sixteenth.steps_per_beat(), 4);
        assert_eq!(NoteDivision::ThirtySecond.steps_per_beat(), 8);
    }

    #[test]
    fn quarter_at_120_bpm_is_half_a_second() {
        assert_approx_eq(NoteDivision::Quarter.seconds(120.0), 0.5);
    }

    #[test]
    fn each_division_is_half_the_previous_ones_time() {
        let bpm = 120.0;

        let quarter = NoteDivision::Quarter.seconds(bpm);
        let eighth = NoteDivision::Eighth.seconds(bpm);
        let sixteenth = NoteDivision::Sixteenth.seconds(bpm);
        let thirty_second = NoteDivision::ThirtySecond.seconds(bpm);

        assert_approx_eq(eighth, quarter / 2.0);
        assert_approx_eq(sixteenth, eighth / 2.0);
        assert_approx_eq(thirty_second, sixteenth / 2.0);
    }

    #[test]
    fn higher_bpm_produces_shorter_times() {
        let slow = NoteDivision::Quarter.seconds(60.0);
        let fast = NoteDivision::Quarter.seconds(140.0);

        assert!(fast < slow);
    }

    #[test]
    fn zero_and_negative_bpm_stay_finite_and_positive() {
        for division in [
            NoteDivision::Quarter,
            NoteDivision::Eighth,
            NoteDivision::Sixteenth,
            NoteDivision::ThirtySecond,
        ] {
            for bpm in [0.0, -10.0] {
                let time = division.seconds(bpm);

                assert!(time.is_finite(), "non-finite time for bpm={bpm}: {time}");
                assert!(time > 0.0, "expected a positive time for bpm={bpm}, got {time}");
            }
        }
    }
}
