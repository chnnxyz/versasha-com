use crate::drums::audio_sample::AudioSample;
use crate::drums::sample_player::SamplePlayer;
use crate::dsp::envelopes::ad_envelope::ADEnvelope;
use crate::dsp::noise::NoiseGenerator;
use crate::dsp::tune::Tune;
use crate::dsp::types::SampleRate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleTrackStatus {
    Active,
    Solo,
    Muted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleTrackType {
    Clap,
    Cymbal,
    HiHat,
    Kick,
    RimShot,
    Snare,
    Tom,
}

impl SampleTrackType {
    fn supports_tune(&self) -> bool {
        matches!(self, Self::Kick | Self::Snare | Self::Tom | Self::Cymbal)
    }

    fn supports_envelope(&self) -> bool {
        matches!(self, Self::Kick | Self::Tom | Self::HiHat)
    }

    fn supports_attack(&self) -> bool {
        matches!(self, Self::Kick)
    }

    fn supports_snappy(&self) -> bool {
        matches!(self, Self::Snare)
    }
}

pub struct SampleTrack {
    status: SampleTrackStatus,
    volume: f32,
    sample: AudioSample,
    track_type: SampleTrackType,
    player: SamplePlayer,
    tune: Option<Tune>,
    envelope: Option<ADEnvelope>,
    noise: Option<NoiseGenerator>,
    snappy_amount: f32,
}

impl SampleTrack {
    pub fn new(sample: AudioSample, rate: SampleRate, track_type: SampleTrackType) -> Self {
        Self {
            status: SampleTrackStatus::Active,
            volume: 1.0,
            player: SamplePlayer::new(rate, None, None, &sample),
            tune: track_type.supports_tune().then(Tune::new),
            envelope: track_type
                .supports_envelope()
                .then(|| ADEnvelope::new(rate)),
            noise: track_type.supports_snappy().then(|| NoiseGenerator::new(1)),
            snappy_amount: 0.0,
            track_type,
            sample,
        }
    }

    // Getters
    pub fn status(&self) -> SampleTrackStatus {
        self.status
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn track_type(&self) -> SampleTrackType {
        self.track_type
    }

    // None means this track type doesn't support the param at all (e.g.
    // Tune on a Rimshot) -- distinct from "supports it, currently at 0"
    pub fn tune_semitones(&self) -> Option<f32> {
        self.tune.as_ref().map(Tune::semitones)
    }

    pub fn attack_time(&self) -> Option<f32> {
        if !self.track_type.supports_attack() {
            return None;
        }

        self.envelope.as_ref().map(ADEnvelope::attack_time)
    }

    pub fn decay_time(&self) -> Option<f32> {
        self.envelope.as_ref().map(ADEnvelope::decay_time)
    }

    pub fn snappy(&self) -> Option<f32> {
        self.noise.is_some().then_some(self.snappy_amount)
    }

    // Setters
    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
    }

    pub fn set_status(&mut self, status: SampleTrackStatus) {
        self.status = status;
    }

    pub fn set_tune(&mut self, semitones: f32) {
        if let Some(tune) = &mut self.tune {
            tune.set_semitones(semitones);
            self.player.set_pitch_multiplier(tune.ratio());
        }
    }

    pub fn set_attack_time(&mut self, time: f32) {
        if !self.track_type.supports_attack() {
            return;
        }

        if let Some(envelope) = &mut self.envelope {
            envelope.set_attack_time(time);
        }
    }

    pub fn set_decay_time(&mut self, time: f32) {
        if let Some(envelope) = &mut self.envelope {
            envelope.set_decay_time(time);
        }
    }

    pub fn set_snappy(&mut self, amount: f32) {
        if self.noise.is_some() {
            self.snappy_amount = amount.clamp(0.0, 1.0);
        }
    }

    // Audio functionality
    pub fn trigger(&mut self) {
        self.player.reset();

        if let Some(envelope) = &mut self.envelope {
            envelope.trigger();
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let player_active = !self.player.is_finished();

        let mut output = self.player.next_sample(&self.sample);

        if let Some(envelope) = &mut self.envelope {
            output *= envelope.next_sample();
        }

        if let Some(noise) = &mut self.noise {
            // always advance the generator so its state stays live even
            // while snappy_amount is 0 or the player has finished
            let noise_sample = noise.next_sample();

            if player_active {
                // shape the noise by the sample's own current amplitude
                // instead of mixing it in at a flat level -- otherwise
                // the noise stays just as loud while the sample
                // naturally decays toward its tail (clashing with it),
                // then hard-cuts the instant the one-shot ends instead
                // of fading out along with it
                let shaped_noise = noise_sample * output.abs();
                output = output * (1.0 - self.snappy_amount) + shaped_noise * self.snappy_amount;
            }
        }

        match self.status {
            SampleTrackStatus::Muted => 0.0,
            SampleTrackStatus::Active | SampleTrackStatus::Solo => output * self.volume,
        }
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
    fn new_sets_default_status_volume_and_track_type() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);

        let track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Kick);

        assert_eq!(track.status(), SampleTrackStatus::Active);

        assert_approx_eq(track.volume(), 1.0);

        assert_eq!(track.track_type(), SampleTrackType::Kick);
    }

    #[test]
    fn set_volume_clamps_above_one() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);

        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Kick);

        track.set_volume(2.0);

        assert_approx_eq(track.volume(), 1.0);
    }

    #[test]
    fn set_volume_clamps_below_zero() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);

        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Kick);

        track.set_volume(-1.0);

        assert_approx_eq(track.volume(), 0.0);
    }

    #[test]
    fn set_volume_accepts_value_in_range() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);

        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Kick);

        track.set_volume(0.5);

        assert_approx_eq(track.volume(), 0.5);
    }

    #[test]
    fn set_status_updates_status() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);

        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Kick);

        track.set_status(SampleTrackStatus::Muted);

        assert_eq!(track.status(), SampleTrackStatus::Muted);

        track.set_status(SampleTrackStatus::Solo);

        assert_eq!(track.status(), SampleTrackStatus::Solo);
    }

    #[test]
    fn next_sample_applies_volume() {
        // Clap deliberately: no envelope/tune/snappy attached to it, so
        // this stays a pure test of volume scaling, not entangled with
        // Kick's now-mandatory envelope gating (see the tests further down
        // that specifically cover envelope/tune/snappy wiring)
        let sample = AudioSample::new(48_000.0, vec![2.0, 4.0]);

        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Clap);

        track.set_volume(0.5);

        assert_approx_eq(track.next_sample(), 1.0);
    }

    #[test]
    fn next_sample_returns_silence_when_muted() {
        let sample = AudioSample::new(48_000.0, vec![5.0]);

        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Kick);

        track.set_status(SampleTrackStatus::Muted);

        assert_approx_eq(track.next_sample(), 0.0);
    }

    #[test]
    fn next_sample_treats_solo_same_as_active() {
        let sample = AudioSample::new(48_000.0, vec![5.0]);

        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Clap);

        track.set_status(SampleTrackStatus::Solo);

        assert_approx_eq(track.next_sample(), 5.0);
    }

    #[test]
    fn next_sample_advances_position_even_when_muted() {
        let sample = AudioSample::new(48_000.0, vec![10.0, 20.0, 30.0]);

        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Clap);

        track.set_status(SampleTrackStatus::Muted);

        // silenced, but the underlying player must still advance
        assert_approx_eq(track.next_sample(), 0.0);

        track.set_status(SampleTrackStatus::Active);

        // proves position moved on to index 1 while muted, rather than
        // pausing and resuming from index 0 once unmuted
        assert_approx_eq(track.next_sample(), 20.0);
    }

    #[test]
    fn trigger_restarts_playback_from_the_beginning() {
        let sample = AudioSample::new(48_000.0, vec![1.0, 2.0, 3.0]);

        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Clap);

        track.next_sample();

        track.next_sample();

        track.trigger();

        assert_approx_eq(track.next_sample(), 1.0);
    }

    // --- Tune (pitch) -------------------------------------------------

    #[test]
    fn tune_is_none_for_types_that_dont_support_it() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);
        let track = SampleTrack::new(sample, 48_000.0, SampleTrackType::RimShot);

        assert_eq!(track.tune_semitones(), None);
    }

    #[test]
    fn set_tune_is_a_no_op_for_types_that_dont_support_it() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);
        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::RimShot);

        track.set_tune(12.0);

        assert_eq!(track.tune_semitones(), None);
    }

    #[test]
    fn set_tune_updates_semitones_and_playback_rate() {
        // Snare: supports tune but not envelope, so this stays a pure
        // tune test without needing an explicit trigger() first. rate =
        // 4.0 so the sample's own native rate (4.0) matches the engine
        // rate, leaving pitch_multiplier as the only thing scaling the
        // advance rate below
        let sample = AudioSample::new(4.0, vec![10.0, 20.0, 30.0, 40.0]);
        let mut track = SampleTrack::new(sample, 4.0, SampleTrackType::Snare);

        track.set_tune(12.0); // +1 octave -> 2x playback rate

        assert_approx_eq(track.tune_semitones().unwrap(), 12.0);

        // an octave up reads every other source sample
        assert_approx_eq(track.next_sample(), 10.0);
        assert_approx_eq(track.next_sample(), 30.0);
    }

    // --- ADEnvelope (attack/decay) -----------------------------------

    #[test]
    fn envelope_is_silent_until_triggered() {
        let sample = AudioSample::new(16.0, vec![7.0, 7.0]);
        let mut track = SampleTrack::new(sample, 16.0, SampleTrackType::Kick);

        // Kick's ADEnvelope starts Idle -- unlike SamplePlayer, which is
        // always "live" from position 0, a percussive envelope must be
        // triggered before it contributes any sound
        assert_approx_eq(track.next_sample(), 0.0);
    }

    #[test]
    fn trigger_opens_the_envelope_and_produces_sound() {
        let sample = AudioSample::new(16.0, vec![7.0, 7.0]);
        let mut track = SampleTrack::new(sample, 16.0, SampleTrackType::Kick);

        track.trigger();

        // at 16Hz, ADEnvelope's default (near-instant) attack time
        // completes within a single sample, so the very first sample
        // after a trigger is already at full envelope level
        assert_approx_eq(track.next_sample(), 7.0);
    }

    #[test]
    fn decay_time_is_none_for_types_that_dont_support_envelope() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);
        let track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Cymbal);

        assert_eq!(track.decay_time(), None);
    }

    #[test]
    fn attack_time_is_none_for_types_that_support_envelope_but_not_attack() {
        // Tom has Decay but no exposed Attack knob
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);
        let track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Tom);

        assert_eq!(track.attack_time(), None);
    }

    #[test]
    fn set_attack_time_is_a_no_op_on_a_type_without_an_attack_knob() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);
        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Tom);

        track.set_attack_time(0.5);

        assert_eq!(track.attack_time(), None);
    }

    #[test]
    fn set_decay_time_updates_a_supported_track() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);
        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Tom);

        track.set_decay_time(0.5);

        assert_approx_eq(track.decay_time().unwrap(), 0.5);
    }

    // --- Snappy (noise mix) --------------------------------------------

    #[test]
    fn snappy_is_none_for_types_that_dont_support_it() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);
        let track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Kick);

        assert_eq!(track.snappy(), None);
    }

    #[test]
    fn set_snappy_updates_a_supported_track_and_clamps() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 4]);
        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Snare);

        track.set_snappy(0.7);
        assert_approx_eq(track.snappy().unwrap(), 0.7);

        track.set_snappy(2.0);
        assert_approx_eq(track.snappy().unwrap(), 1.0);
    }

    #[test]
    fn full_snappy_replaces_the_sample_entirely_with_noise() {
        let sample = AudioSample::new(48_000.0, vec![0.3, 0.3, 0.3]);
        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Snare);

        track.set_snappy(1.0);

        // at snappy = 1.0 the mix formula collapses to 100% noise -- it
        // must not equal the untouched sample value
        assert_ne!(track.next_sample(), 0.3);
    }

    #[test]
    fn snappy_noise_amplitude_follows_the_samples_own_envelope() {
        // both tracks seed their NoiseGenerator identically (SampleTrack::new
        // hardcodes seed 1), so on this first call each produces the exact
        // same raw noise value -- any difference in the two outputs has to
        // come from the shaping, not from the noise generator itself
        let loud_sample = AudioSample::new(48_000.0, vec![0.9]);
        let mut loud_track = SampleTrack::new(loud_sample, 48_000.0, SampleTrackType::Snare);
        loud_track.set_snappy(1.0);

        let quiet_sample = AudioSample::new(48_000.0, vec![0.1]);
        let mut quiet_track = SampleTrack::new(quiet_sample, 48_000.0, SampleTrackType::Snare);
        quiet_track.set_snappy(1.0);

        let loud_output = loud_track.next_sample().abs();
        let quiet_output = quiet_track.next_sample().abs();

        assert!(
            loud_output > quiet_output,
            "expected the loud sample's noise contribution ({loud_output}) to be louder than the quiet sample's ({quiet_output}) -- noise should track the sample's own amplitude, not mix in at a flat level"
        );
    }

    #[test]
    fn snappy_noise_stops_once_the_underlying_sample_is_exhausted() {
        let sample = AudioSample::new(48_000.0, vec![0.3, 0.3]);
        let mut track = SampleTrack::new(sample, 48_000.0, SampleTrackType::Snare);

        track.set_snappy(1.0);

        track.next_sample();
        track.next_sample();

        // player is now exhausted -- noise must not leak on forever after
        // the one-shot naturally ends
        assert_approx_eq(track.next_sample(), 0.0);
    }
}
