use crate::dsp::types::{Sample, SampleRate, Time};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrumEnvelopeState {
    Idle,
    Attack,
    Decay,
}

const MIN_ATTACK: Time = 0.0005; // 0.5ms floor -- 0.0 would divide by zero below
const MIN_DECAY: Time = 0.001;
const DEFAULT_ATTACK: Time = 0.001; // near-instant, favors punch/click by default
const DEFAULT_DECAY: Time = 0.2;

// -60dB is the conventional "effectively silent" floor for measuring a
// decay time constant -- past this point the ear can't distinguish the
// tail from the noise floor, so there's no audible benefit in decaying
// further, and it gives next_sample() a concrete point to snap to Idle
const SILENCE_FLOOR: Sample = 0.001;

pub struct DrumEnvelope {
    state: DrumEnvelopeState,
    level: Sample,

    attack_time: Time,
    decay_time: Time,

    sample_rate: SampleRate,
    attack_increment: Sample,
    decay_coefficient: Sample,
}

impl DrumEnvelope {
    pub fn new(sample_rate: SampleRate) -> Self {
        let mut envelope = Self {
            state: DrumEnvelopeState::Idle,
            level: 0.0,
            attack_time: DEFAULT_ATTACK,
            decay_time: DEFAULT_DECAY,
            sample_rate,
            attack_increment: 0.0,
            decay_coefficient: 0.0,
        };

        envelope.recompute_attack();
        envelope.recompute_decay();

        envelope
    }

    // linear ramp envelope
    fn recompute_attack(&mut self) {
        self.attack_increment = 1.0 / (self.attack_time * self.sample_rate);
    }

    // exponential decay: level *= coefficient every sample
    fn recompute_decay(&mut self) {
        let samples = self.decay_time * self.sample_rate;
        self.decay_coefficient = SILENCE_FLOOR.powf(1.0 / samples);
    }

    // getters
    pub fn state(&self) -> DrumEnvelopeState {
        self.state
    }

    pub fn level(&self) -> Sample {
        self.level
    }

    pub fn attack_time(&self) -> Time {
        self.attack_time
    }

    pub fn decay_time(&self) -> Time {
        self.decay_time
    }

    // setters
    pub fn set_attack_time(&mut self, time: Time) {
        self.attack_time = time.max(MIN_ATTACK);
        self.recompute_attack();
    }

    pub fn set_decay_time(&mut self, time: Time) {
        self.decay_time = time.max(MIN_DECAY);
        self.recompute_decay();
    }

    // hard reset as in SamplePlayer::trigger()
    pub fn trigger(&mut self) {
        self.level = 0.0;
        self.state = DrumEnvelopeState::Attack;
    }

    pub fn next_sample(&mut self) -> Sample {
        match self.state {
            DrumEnvelopeState::Idle => {
                self.level = 0.0;
            }

            DrumEnvelopeState::Attack => {
                self.level += self.attack_increment;

                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.state = DrumEnvelopeState::Decay;
                }
            }

            DrumEnvelopeState::Decay => {
                self.level *= self.decay_coefficient;

                if self.level <= SILENCE_FLOOR {
                    self.level = 0.0;
                    self.state = DrumEnvelopeState::Idle;
                }
            }
        }

        self.level
    }

    pub fn reset(&mut self) {
        self.state = DrumEnvelopeState::Idle;
        self.level = 0.0;
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
    fn starts_idle_and_silent() {
        let env = DrumEnvelope::new(48_000.0);

        assert_eq!(env.state(), DrumEnvelopeState::Idle);
        assert_eq!(env.level(), 0.0);
    }

    #[test]
    fn trigger_enters_attack_from_zero() {
        let mut env = DrumEnvelope::new(48_000.0);

        env.trigger();

        assert_eq!(env.state(), DrumEnvelopeState::Attack);
        assert_eq!(env.level(), 0.0);
    }

    #[test]
    fn attack_reaches_full_level_then_moves_to_decay() {
        let sample_rate = 48_000.0;
        let mut env = DrumEnvelope::new(sample_rate);

        env.set_attack_time(0.01);
        env.trigger();

        let attack_samples = (sample_rate * 0.01) as usize;

        for _ in 0..attack_samples {
            env.next_sample();
        }

        assert_approx_eq(env.level(), 1.0);
        assert_eq!(env.state(), DrumEnvelopeState::Decay);
    }

    #[test]
    fn attack_is_monotonically_increasing() {
        let mut env = DrumEnvelope::new(48_000.0);

        env.set_attack_time(0.05);
        env.trigger();

        let mut previous = 0.0;

        for _ in 0..100 {
            let current = env.next_sample();

            assert!(current >= previous);

            previous = current;
        }
    }

    #[test]
    fn decay_falls_to_silence_and_returns_to_idle() {
        let sample_rate = 48_000.0;
        let mut env = DrumEnvelope::new(sample_rate);

        env.set_attack_time(0.001);
        env.set_decay_time(0.05);
        env.trigger();

        for _ in 0..sample_rate as usize {
            env.next_sample();
        }

        assert_eq!(env.level(), 0.0);
        assert_eq!(env.state(), DrumEnvelopeState::Idle);
    }

    #[test]
    fn decay_is_monotonically_decreasing() {
        let mut env = DrumEnvelope::new(48_000.0);

        env.set_attack_time(0.001);
        env.set_decay_time(0.2);
        env.trigger();

        // drain the (very short) attack first
        for _ in 0..100 {
            env.next_sample();
        }

        let mut previous = env.level();

        for _ in 0..1000 {
            let current = env.next_sample();

            assert!(current <= previous);

            previous = current;
        }
    }

    #[test]
    fn retrigger_restarts_cleanly_even_mid_decay() {
        let mut env = DrumEnvelope::new(48_000.0);

        env.set_attack_time(0.001);
        env.set_decay_time(0.2);
        env.trigger();

        for _ in 0..5000 {
            env.next_sample();
        }

        assert_eq!(env.state(), DrumEnvelopeState::Decay);

        env.trigger();

        assert_eq!(env.state(), DrumEnvelopeState::Attack);
        assert_eq!(env.level(), 0.0);
    }

    #[test]
    fn shorter_decay_time_reaches_silence_sooner() {
        let sample_rate = 48_000.0;

        let mut short = DrumEnvelope::new(sample_rate);
        short.set_attack_time(0.001);
        short.set_decay_time(0.05);
        short.trigger();

        let mut long = DrumEnvelope::new(sample_rate);
        long.set_attack_time(0.001);
        long.set_decay_time(0.5);
        long.trigger();

        // drain attack on both
        for _ in 0..100 {
            short.next_sample();
            long.next_sample();
        }

        for _ in 0..(0.1 * sample_rate) as usize {
            short.next_sample();
            long.next_sample();
        }

        // by 100ms in, the short-decay envelope should already be quieter
        assert!(short.level() < long.level());
    }

    #[test]
    fn set_attack_time_rejects_non_positive_values() {
        let mut env = DrumEnvelope::new(48_000.0);

        env.set_attack_time(-1.0);

        assert!(env.attack_time() >= MIN_ATTACK);
    }

    #[test]
    fn set_decay_time_rejects_non_positive_values() {
        let mut env = DrumEnvelope::new(48_000.0);

        env.set_decay_time(0.0);

        assert!(env.decay_time() >= MIN_DECAY);
    }
}
