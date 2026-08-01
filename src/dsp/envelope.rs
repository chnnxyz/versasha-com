use super::types::{Sample, SampleRate};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Envelope {
    // Defines current state, to use in impl for self
    state: EnvelopeState,

    // ADSR params
    attack: Sample,
    decay: Sample,
    sustain: Sample,
    release: Sample,

    attack_increment: Sample,
    decay_increment: Sample,
    release_increment: Sample,

    // Drywet/volumme
    level: Sample,

    sample_rate: SampleRate,
}

impl Envelope {
    pub fn new(rate: SampleRate) -> Self {
        let attack = 0.01;
        let decay = 0.1;
        let sustain = 0.8;
        let release = 0.2;

        Self {
            state: EnvelopeState::Idle,

            attack,
            decay,
            sustain,
            release,

            level: 0.0,

            attack_increment: 1.0 / (attack * rate),
            decay_increment: (1.0 - sustain) / (decay * rate),
            release_increment: 0.0,

            sample_rate: rate,
        }
    }

    // Setters
    //
    // Getters
    pub fn state(&self) -> EnvelopeState {
        self.state
    }

    pub fn level(&self) -> Sample {
        self.level
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    // When note is played launch attack
    pub fn note_on(&mut self) {
        self.state = EnvelopeState::Attack;
    }

    // When note is released set release state
    pub fn note_off(&mut self) {
        self.release_increment = self.level / (self.release * self.sample_rate);

        self.state = EnvelopeState::Release;
    }

    pub fn next_sample(&mut self) -> Sample {
        match self.state {
            EnvelopeState::Idle => {
                self.level = 0.0;
            }

            EnvelopeState::Attack => {
                self.level += self.attack_increment;

                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.state = EnvelopeState::Decay;
                }
            }

            EnvelopeState::Decay => {
                self.level -= self.decay_increment;

                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.state = EnvelopeState::Sustain;
                }
            }

            EnvelopeState::Sustain => {
                self.level = self.sustain;
            }

            EnvelopeState::Release => {
                self.level -= self.release_increment;

                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.state = EnvelopeState::Idle;
                }
            }
        }

        self.level
    }

    // ye good ole reset
    pub fn reset(&mut self) {
        self.state = EnvelopeState::Idle;
        self.level = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_idle() {
        let env = Envelope::new(48_000.0);

        assert_eq!(env.state(), EnvelopeState::Idle);
        assert_eq!(env.level(), 0.0);
    }

    #[test]
    fn note_on_enters_attack() {
        let mut env = Envelope::new(48_000.0);

        env.note_on();

        assert_eq!(env.state(), EnvelopeState::Attack);
    }

    #[test]
    fn attack_reaches_full_level() {
        let sample_rate = 48_000.0;
        let attack_time = 0.01;

        let mut env = Envelope::new(sample_rate);

        env.note_on();

        let attack_samples = (sample_rate * attack_time) as usize;

        for _ in 0..attack_samples {
            env.next_sample();
        }

        assert_eq!(env.level(), 1.0);
        assert_eq!(env.state(), EnvelopeState::Decay);
    }

    #[test]
    fn attack_is_monotonic() {
        let mut env = Envelope::new(48_000.0);

        env.note_on();

        let mut previous = 0.0;

        for _ in 0..100 {
            let current = env.next_sample();

            assert!(current >= previous);

            previous = current;
        }
    }
    #[test]
    fn decay_reaches_sustain() {
        let mut env = Envelope::new(48_000.0);

        env.note_on();

        // enough samples to finish attack + decay
        for _ in 0..20_000 {
            env.next_sample();
        }

        assert_eq!(env.state(), EnvelopeState::Sustain);
        assert!((env.level() - 0.8).abs() < 1e-6);
    }

    #[test]
    fn sustain_stays_constant() {
        let mut env = Envelope::new(48_000.0);

        env.note_on();

        for _ in 0..20_000 {
            env.next_sample();
        }

        let first = env.next_sample();
        let second = env.next_sample();
        let third = env.next_sample();

        assert_eq!(first, 0.8);
        assert_eq!(second, 0.8);
        assert_eq!(third, 0.8);
    }
    #[test]
    fn note_off_enters_release() {
        let mut env = Envelope::new(48_000.0);

        env.note_on();

        for _ in 0..20_000 {
            env.next_sample();
        }

        env.note_off();

        assert_eq!(env.state(), EnvelopeState::Release);
    }

    #[test]
    fn release_reaches_zero() {
        let mut env = Envelope::new(48_000.0);

        env.note_on();

        for _ in 0..20_000 {
            env.next_sample();
        }

        env.note_off();

        for _ in 0..20_000 {
            env.next_sample();
        }

        assert_eq!(env.level(), 0.0);
        assert_eq!(env.state(), EnvelopeState::Idle);
    }

    #[test]
    fn release_decreases_monotonically() {
        let mut env = Envelope::new(48_000.0);

        env.note_on();

        for _ in 0..20_000 {
            env.next_sample();
        }

        env.note_off();

        let mut previous = env.level();

        for _ in 0..100 {
            let current = env.next_sample();

            assert!(current <= previous);

            previous = current;
        }
    }
}
