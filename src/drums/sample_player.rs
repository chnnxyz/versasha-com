use crate::drums::audio_sample::AudioSample;
use crate::dsp::types::SampleRate;

pub struct SamplePlayer {
    position: f32,
    sample_rate: SampleRate,
    start_position: f32,
    end_position: f32,
    pitch_multiplier: f32,
    triggered: bool,
}

impl SamplePlayer {
    pub fn new(
        rate: SampleRate,
        start_pos: Option<f32>,
        end_pos: Option<f32>,
        sample: &AudioSample,
    ) -> Self {
        Self {
            position: start_pos.unwrap_or(0.0),
            sample_rate: rate,
            start_position: start_pos.unwrap_or(0.0),
            end_position: end_pos.unwrap_or(sample.len() as f32),
            pitch_multiplier: 1.0,
            triggered: false,
        }
    }

    // getters
    pub fn position(&self) -> f32 {
        self.position
    }

    pub fn start_position(&self) -> f32 {
        self.start_position
    }

    pub fn end_position(&self) -> f32 {
        self.end_position
    }

    pub fn pitch_multiplier(&self) -> f32 {
        self.pitch_multiplier
    }

    pub fn is_triggered(&self) -> bool {
        self.triggered
    }

    //settters (for potential future things like starting the sample at a different point or ending it earlt)
    pub fn set_start_position(&mut self, pos: f32) {
        self.start_position = pos;
    }

    pub fn set_end_position(&mut self, pos: f32) {
        self.end_position = pos;
    }

    // fed by Tune::ratio() -- 1.0 leaves the natural advance rate below
    // untouched, >1.0 reads through the buffer faster (higher pitch,
    // shorter duration), <1.0 slower (lower pitch, longer duration)
    pub fn set_pitch_multiplier(&mut self, multiplier: f32) {
        self.pitch_multiplier = multiplier;
    }

    pub fn next_sample(&mut self, sample: &AudioSample) -> f32 {
        if !self.triggered || self.position >= self.end_position {
            return 0.0;
        }
        let value: f32 = sample.sample_at(self.position);
        let adv_rate: f32 = (sample.sample_rate() / self.sample_rate) * self.pitch_multiplier;
        self.position += adv_rate;
        value
    }

    pub fn is_finished(&self) -> bool {
        !self.triggered || self.position >= self.end_position
    }

    pub fn reset(&mut self) {
        self.position = self.start_position;
        self.triggered = true;
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
    fn new_without_start_or_end_uses_defaults() {
        let sample = AudioSample::new(48_000.0, vec![0.1, 0.2, 0.3]);

        let player = SamplePlayer::new(48_000.0, None, None, &sample);

        assert_approx_eq(player.position(), 0.0);

        assert_approx_eq(player.start_position(), 0.0);

        assert_approx_eq(player.end_position(), 3.0);
    }

    #[test]
    fn new_with_custom_start_and_end_positions() {
        let sample = AudioSample::new(48_000.0, vec![0.0; 10]);

        let player = SamplePlayer::new(48_000.0, Some(2.0), Some(5.0), &sample);

        assert_approx_eq(player.position(), 2.0);

        assert_approx_eq(player.start_position(), 2.0);

        assert_approx_eq(player.end_position(), 5.0);
    }

    #[test]
    fn new_player_is_not_triggered_and_stays_silent() {
        let sample = AudioSample::new(48_000.0, vec![1.0, 2.0, 3.0]);

        let mut player = SamplePlayer::new(48_000.0, None, None, &sample);

        assert!(!player.is_triggered());

        assert_approx_eq(player.next_sample(&sample), 0.0);
        assert_approx_eq(player.next_sample(&sample), 0.0);
        assert_approx_eq(player.next_sample(&sample), 0.0);
    }

    #[test]
    fn position_does_not_advance_before_being_triggered() {
        let sample = AudioSample::new(48_000.0, vec![1.0, 2.0, 3.0]);

        let mut player = SamplePlayer::new(48_000.0, None, None, &sample);

        player.next_sample(&sample);
        player.next_sample(&sample);

        assert_approx_eq(player.position(), player.start_position());
    }

    #[test]
    fn is_finished_is_true_before_being_triggered() {
        let sample = AudioSample::new(48_000.0, vec![1.0, 2.0, 3.0]);

        let player = SamplePlayer::new(48_000.0, None, None, &sample);

        assert!(player.is_finished());
    }

    #[test]
    fn reset_marks_the_player_as_triggered() {
        let sample = AudioSample::new(48_000.0, vec![1.0, 2.0, 3.0]);

        let mut player = SamplePlayer::new(48_000.0, None, None, &sample);
        assert!(!player.is_triggered());

        player.reset();

        assert!(player.is_triggered());
    }

    #[test]
    fn next_sample_reads_current_position_before_advancing() {
        let sample = AudioSample::new(48_000.0, vec![1.0, 2.0, 3.0, 4.0]);

        let mut player = SamplePlayer::new(48_000.0, None, None, &sample);
        player.reset();

        assert_approx_eq(player.next_sample(&sample), 1.0);

        assert_approx_eq(player.next_sample(&sample), 2.0);

        assert_approx_eq(player.next_sample(&sample), 3.0);

        assert_approx_eq(player.next_sample(&sample), 4.0);
    }

    #[test]
    fn next_sample_returns_silence_after_end_of_buffer() {
        let sample = AudioSample::new(48_000.0, vec![1.0, 2.0]);

        let mut player = SamplePlayer::new(48_000.0, None, None, &sample);
        player.reset();

        player.next_sample(&sample);

        player.next_sample(&sample);

        assert_approx_eq(player.next_sample(&sample), 0.0);

        assert_approx_eq(player.next_sample(&sample), 0.0);
    }

    #[test]
    fn next_sample_respects_custom_end_position() {
        let sample = AudioSample::new(48_000.0, vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        let mut player = SamplePlayer::new(48_000.0, None, Some(2.0), &sample);
        player.reset();

        assert_approx_eq(player.next_sample(&sample), 1.0);

        assert_approx_eq(player.next_sample(&sample), 2.0);

        // position has now reached end_position (2.0), even though the
        // underlying buffer still has real data at indices 2, 3, and 4
        assert_approx_eq(player.next_sample(&sample), 0.0);
    }

    #[test]
    fn is_finished_is_false_before_reaching_end_position() {
        let sample = AudioSample::new(48_000.0, vec![1.0, 2.0, 3.0]);

        let mut player = SamplePlayer::new(48_000.0, None, None, &sample);
        player.reset();

        assert!(!player.is_finished());
    }

    #[test]
    fn is_finished_is_true_once_position_reaches_end_position() {
        let sample = AudioSample::new(48_000.0, vec![1.0, 2.0]);

        let mut player = SamplePlayer::new(48_000.0, None, None, &sample);
        player.reset();

        player.next_sample(&sample);

        player.next_sample(&sample);

        assert!(player.is_finished());
    }

    #[test]
    fn reset_returns_to_start_position_not_zero() {
        let sample = AudioSample::new(48_000.0, vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        let mut player = SamplePlayer::new(48_000.0, Some(3.0), None, &sample);

        player.reset(); // trigger it so next_sample actually advances
        player.next_sample(&sample); // position moves from 3.0 to 4.0

        player.reset(); // reset again -- should go back to 3.0, not 0.0

        assert_approx_eq(player.position(), 3.0);
    }

    #[test]
    fn reset_allows_replaying_from_start_position() {
        let sample = AudioSample::new(48_000.0, vec![10.0, 20.0, 30.0]);

        let mut player = SamplePlayer::new(48_000.0, Some(1.0), None, &sample);

        player.reset();
        assert_approx_eq(player.next_sample(&sample), 20.0);

        player.reset();
        assert_approx_eq(player.next_sample(&sample), 20.0);
    }

    #[test]
    fn next_sample_advances_slower_when_buffer_rate_is_lower() {
        let sample = AudioSample::new(24_000.0, vec![10.0, 20.0, 30.0, 40.0]);

        let mut player = SamplePlayer::new(48_000.0, None, None, &sample);
        player.reset();

        // buffer rate is half the engine rate, so each source sample is
        // read twice before advancing to the next one
        assert_approx_eq(player.next_sample(&sample), 10.0);

        assert_approx_eq(player.next_sample(&sample), 10.0);

        assert_approx_eq(player.next_sample(&sample), 20.0);

        assert_approx_eq(player.next_sample(&sample), 20.0);
    }

    #[test]
    fn pitch_multiplier_defaults_to_one() {
        let sample = AudioSample::new(48_000.0, vec![1.0, 2.0, 3.0]);

        let player = SamplePlayer::new(48_000.0, None, None, &sample);

        assert_approx_eq(player.pitch_multiplier(), 1.0);
    }

    #[test]
    fn pitch_multiplier_scales_the_advance_rate() {
        let sample = AudioSample::new(48_000.0, vec![10.0, 20.0, 30.0, 40.0]);

        let mut player = SamplePlayer::new(48_000.0, None, None, &sample);
        player.reset();

        player.set_pitch_multiplier(2.0);

        // double-speed playback: reads every other source sample
        assert_approx_eq(player.next_sample(&sample), 10.0);
        assert_approx_eq(player.next_sample(&sample), 30.0);
    }
}
