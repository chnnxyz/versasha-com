use crate::drums::audio_sample::AudioSample;
use crate::drums::sample_track::{SampleTrack, SampleTrackStatus, SampleTrackType};
use crate::drums::sequencer::Sequencer;
use crate::dsp::types::SampleRate;
use crate::sequencing::transport::SequencerStatus;

// TODO(session-refactor): once engine::session::Session exists and owns a
// shared sequencing::transport::Transport, this field plus
// play()/pause()/stop()/bpm()/set_bpm()/current_step()/sequencer_status()
// below and the advance-and-trigger block at the top of next_sample()
// all become redundant -- that responsibility moves to Transport, and
// DrumMachine gets driven via trigger_step() instead. Left in place for
// now so DrumMachine keeps working standalone (drums.html) until that
// switch actually happens.
pub struct DrumMachine {
    sequencer_status: SequencerStatus,
    sequencer: Sequencer,
    tracks: Vec<SampleTrack>,
    active_track: Option<usize>,
    master_volume: f32,
}

impl DrumMachine {
    // one parameter per one-shot; explicit names rather than a positional
    // Vec<Vec<f32>> so a mixed-up ordering fails to compile / is obvious to
    // spot, rather than silently loading the wrong sample onto a track
    #[allow(clippy::too_many_arguments)]
    fn build_all_tracks(
        sample_rate: SampleRate,
        kick: Vec<f32>,
        snare: Vec<f32>,
        clap: Vec<f32>,
        rimshot: Vec<f32>,
        tom_low: Vec<f32>,
        tom_mid: Vec<f32>,
        tom_hi: Vec<f32>,
        hihat_closed: Vec<f32>,
        hihat_open: Vec<f32>,
        crash: Vec<f32>,
        ride: Vec<f32>,
    ) -> Vec<SampleTrack> {
        vec![
            SampleTrack::new(
                AudioSample::new(sample_rate, kick),
                sample_rate,
                SampleTrackType::Kick,
            ),
            SampleTrack::new(
                AudioSample::new(sample_rate, snare),
                sample_rate,
                SampleTrackType::Snare,
            ),
            SampleTrack::new(
                AudioSample::new(sample_rate, clap),
                sample_rate,
                SampleTrackType::Clap,
            ),
            SampleTrack::new(
                AudioSample::new(sample_rate, rimshot),
                sample_rate,
                SampleTrackType::RimShot,
            ),
            SampleTrack::new(
                AudioSample::new(sample_rate, tom_low),
                sample_rate,
                SampleTrackType::Tom,
            ),
            SampleTrack::new(
                AudioSample::new(sample_rate, tom_mid),
                sample_rate,
                SampleTrackType::Tom,
            ),
            SampleTrack::new(
                AudioSample::new(sample_rate, tom_hi),
                sample_rate,
                SampleTrackType::Tom,
            ),
            SampleTrack::new(
                AudioSample::new(sample_rate, hihat_closed),
                sample_rate,
                SampleTrackType::HiHat,
            ),
            SampleTrack::new(
                AudioSample::new(sample_rate, hihat_open),
                sample_rate,
                SampleTrackType::HiHat,
            ),
            SampleTrack::new(
                AudioSample::new(sample_rate, crash),
                sample_rate,
                SampleTrackType::Cymbal,
            ),
            SampleTrack::new(
                AudioSample::new(sample_rate, ride),
                sample_rate,
                SampleTrackType::Cymbal,
            ),
        ]
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rate: SampleRate,
        kick: Vec<f32>,
        snare: Vec<f32>,
        clap: Vec<f32>,
        rimshot: Vec<f32>,
        tom_low: Vec<f32>,
        tom_mid: Vec<f32>,
        tom_hi: Vec<f32>,
        hihat_closed: Vec<f32>,
        hihat_open: Vec<f32>,
        crash: Vec<f32>,
        ride: Vec<f32>,
    ) -> Self {
        Self {
            sequencer_status: SequencerStatus::Stop,
            active_track: None,
            tracks: Self::build_all_tracks(
                rate,
                kick,
                snare,
                clap,
                rimshot,
                tom_low,
                tom_mid,
                tom_hi,
                hihat_closed,
                hihat_open,
                crash,
                ride,
            ),
            sequencer: Sequencer::new(rate, 16, 11, 120.0, None),
            master_volume: 0.5,
        }
    }

    //helpers
    pub fn get_solo_tracks(&self) -> Vec<bool> {
        self.tracks
            .iter()
            .map(|track| track.status() == SampleTrackStatus::Solo)
            .collect()
    }

    pub fn any_solo(&self) -> bool {
        self.tracks
            .iter()
            .any(|track| track.status() == SampleTrackStatus::Solo)
    }

    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    //getters
    pub fn active_track(&self) -> Option<usize> {
        self.active_track
    }

    pub fn tracks(&self) -> &[SampleTrack] {
        &self.tracks
    }

    pub fn sequencer(&self) -> &Sequencer {
        &self.sequencer
    }

    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn bpm(&self) -> f32 {
        self.sequencer.bpm()
    }

    pub fn sequencer_status(&self) -> SequencerStatus {
        self.sequencer_status
    }

    pub fn current_step(&self) -> usize {
        self.sequencer.current_step()
    }

    // pass-through to Sequencer::active_tracks -- named differently from
    // `active_track` (singular, the UI-selected track for editing) on
    // purpose, since the two are easy to confuse otherwise
    pub fn active_step_tracks(&self) -> &[bool] {
        self.sequencer().active_tracks()
    }

    // Option, not a direct/panicking index, since these take an index
    // supplied from outside (UI/wasm) that could be out of range
    pub fn track_volume(&self, index: usize) -> Option<f32> {
        self.tracks.get(index).map(|track| track.volume())
    }

    pub fn track_status(&self, index: usize) -> Option<SampleTrackStatus> {
        self.tracks.get(index).map(|track| track.status())
    }

    pub fn track_type(&self, index: usize) -> Option<SampleTrackType> {
        self.tracks.get(index).map(|track| track.track_type())
    }

    // Option<Option<f32>> would leak the "bad index" vs "type doesn't
    // support this param" distinction into the wrong layer -- flatten it,
    // since callers here only need "is there a usable value or not"
    pub fn track_tune(&self, index: usize) -> Option<f32> {
        self.tracks
            .get(index)
            .and_then(|track| track.tune_semitones())
    }

    pub fn track_attack(&self, index: usize) -> Option<f32> {
        self.tracks.get(index).and_then(|track| track.attack_time())
    }

    pub fn track_decay(&self, index: usize) -> Option<f32> {
        self.tracks.get(index).and_then(|track| track.decay_time())
    }

    pub fn track_snappy(&self, index: usize) -> Option<f32> {
        self.tracks.get(index).and_then(|track| track.snappy())
    }

    // setters
    pub fn set_master_volume(&mut self, vol: f32) {
        self.master_volume = vol.clamp(0.0, 1.0);
    }

    pub fn set_active_track(&mut self, index: Option<usize>) {
        self.active_track = index;
    }

    // thin delegation to self.sequencer, no separate stored bpm here
    pub fn set_bpm(&mut self, bpm: f32) {
        self.sequencer.set_bpm(bpm);
    }

    pub fn play(&mut self) {
        self.sequencer_status = SequencerStatus::Play;
    }

    pub fn pause(&mut self) {
        self.sequencer_status = SequencerStatus::Pause;
    }

    pub fn stop(&mut self) {
        self.sequencer_status = SequencerStatus::Stop;
        self.sequencer.stop();
    }

    pub fn set_track_volume(&mut self, index: usize, volume: f32) {
        if let Some(track) = self.tracks.get_mut(index) {
            track.set_volume(volume);
        }
    }

    pub fn set_track_status(&mut self, index: usize, status: SampleTrackStatus) {
        if let Some(track) = self.tracks.get_mut(index) {
            track.set_status(status);
        }
    }

    // each of these is a no-op on both a bad index (get_mut returns None)
    // and an unsupported track type (SampleTrack's own setter no-ops) --
    // same "silently ignore bad external input" contract as set_track_volume
    pub fn set_track_tune(&mut self, index: usize, semitones: f32) {
        if let Some(track) = self.tracks.get_mut(index) {
            track.set_tune(semitones);
        }
    }

    pub fn set_track_attack(&mut self, index: usize, time: f32) {
        if let Some(track) = self.tracks.get_mut(index) {
            track.set_attack_time(time);
        }
    }

    pub fn set_track_decay(&mut self, index: usize, time: f32) {
        if let Some(track) = self.tracks.get_mut(index) {
            track.set_decay_time(time);
        }
    }

    pub fn set_track_snappy(&mut self, index: usize, amount: f32) {
        if let Some(track) = self.tracks.get_mut(index) {
            track.set_snappy(amount);
        }
    }

    // pass-through to Sequencer::set_step -- Sequencer itself indexes
    // directly and will panic on an out-of-range step/track, so this is
    // where external (UI/wasm) input gets validated before it gets there
    pub fn set_step(&mut self, step: usize, track: usize, active: bool) {
        if step < self.sequencer.step_count() && track < self.track_count() {
            self.sequencer.set_step(step, track, active);
        }
    }

    pub fn clear_track_pattern(&mut self, track: usize) {
        if track < self.track_count() {
            self.sequencer.clear_one_track(track);
        }
    }

    pub fn clear_all_patterns(&mut self) {
        self.sequencer.clear_all_tracks();
    }

    // audio_ tools
    pub fn trigger_track(&mut self, index: usize) {
        // like a real 909: each physical key/pad maps directly to one
        // specific track, decided on the UI/JS side -- the same way your
        // synth keyboard already maps a fixed key to a fixed frequency and
        // just calls note_on(frequency) with it. So this takes the track
        // index straight from the caller, rather than relying on a
        // separately "selected" track.
        //
        // index comes from outside (UI/wasm), so it can be out of range --
        // use Option-based access (self.tracks.get_mut(index)), not direct
        // indexing, so a bad key mapping can't panic the audio thread
        if let Some(track) = self.tracks.get_mut(index) {
            track.trigger();
        }
    }

    // TODO(session-refactor): the method a shared Transport will call
    // when it crosses a step boundary. Same trigger loop as the one
    // inlined at the top of next_sample() below, reading an
    // externally-supplied step instead of advancing its own clock.
    pub fn trigger_step(&mut self, step: usize) {
        if step < self.sequencer.step_count() {
            let active_tracks = self.sequencer.tracks_at(step);

            for (index, &is_active) in active_tracks.iter().enumerate() {
                if is_active {
                    self.tracks[index].trigger();
                }
            }
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        // only new triggers are gated on Play -- a track triggered right
        // before a pause/stop keeps ringing out naturally afterward below,
        // it just won't receive any *new* triggers until playback resumes
        if self.sequencer_status == SequencerStatus::Play && self.sequencer.advance() {
            let active_tracks = self.sequencer.active_tracks();

            for (index, &is_active) in active_tracks.iter().enumerate() {
                if is_active {
                    self.tracks[index].trigger();
                }
            }
        }

        let solo_active = self.any_solo();

        self.tracks
            .iter_mut()
            .map(|track| {
                // always advance every track's own playback position,
                // regardless of solo state -- same reasoning as why
                // SampleTrack keeps advancing while muted: skipping this
                // call while solo-excluded would freeze that track's
                // position, so un-soloing later would resume from a
                // stale, out-of-sync spot instead of playing in real time
                let sample = track.next_sample();
                let qualifies = !solo_active || track.status() == SampleTrackStatus::Solo;

                if qualifies { sample } else { 0.0 }
            })
            .sum::<f32>()
            * self.master_volume
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

    // rate = 16.0 (small, power-of-2 friendly, same reasoning as
    // Sequencer's own tests) gives samples_per_step = 2.0 exactly at the
    // hardcoded 120 BPM / 16-step / 4-steps-per-beat construction, so a
    // step boundary is reachable in just 2 calls. Every track gets the
    // same 2-sample [7.0, 7.0] one-shot, so a fresh machine's very first
    // sample is *not* silence -- every track starts "live" at position 0.
    fn test_machine() -> DrumMachine {
        let sample_rate = 16.0;
        let data = vec![7.0, 7.0];

        DrumMachine::new(
            sample_rate,
            data.clone(),
            data.clone(),
            data.clone(),
            data.clone(),
            data.clone(),
            data.clone(),
            data.clone(),
            data.clone(),
            data.clone(),
            data.clone(),
            data,
        )
    }

    #[test]
    fn new_sets_expected_defaults() {
        let machine = test_machine();

        assert_eq!(machine.sequencer_status(), SequencerStatus::Stop);
        assert_eq!(machine.active_track(), None);
        assert_approx_eq(machine.master_volume(), 0.5);
        assert_eq!(machine.track_count(), 11);
    }

    #[test]
    fn set_master_volume_clamps() {
        let mut machine = test_machine();

        machine.set_master_volume(2.0);
        assert_approx_eq(machine.master_volume(), 1.0);

        machine.set_master_volume(-1.0);
        assert_approx_eq(machine.master_volume(), 0.0);
    }

    #[test]
    fn set_active_track_updates_value() {
        let mut machine = test_machine();

        machine.set_active_track(Some(3));
        assert_eq!(machine.active_track(), Some(3));

        machine.set_active_track(None);
        assert_eq!(machine.active_track(), None);
    }

    #[test]
    fn track_getters_return_none_for_out_of_range_index() {
        let machine = test_machine();

        assert_eq!(machine.track_volume(999), None);
        assert_eq!(machine.track_status(999), None);
        assert_eq!(machine.track_type(999), None);
    }

    #[test]
    fn track_getters_return_data_for_valid_index() {
        let machine = test_machine();

        // index 0 is always kick, per build_all_tracks' fixed ordering
        assert_approx_eq(machine.track_volume(0).unwrap(), 1.0);
        assert_eq!(machine.track_status(0), Some(SampleTrackStatus::Active));
        assert_eq!(machine.track_type(0), Some(SampleTrackType::Kick));
    }

    #[test]
    fn set_track_volume_updates_and_ignores_bad_index() {
        let mut machine = test_machine();

        machine.set_track_volume(0, 0.3);
        assert_approx_eq(machine.track_volume(0).unwrap(), 0.3);

        // must not panic
        machine.set_track_volume(999, 0.3);
    }

    #[test]
    fn set_track_status_updates_and_ignores_bad_index() {
        let mut machine = test_machine();

        machine.set_track_status(0, SampleTrackStatus::Muted);
        assert_eq!(machine.track_status(0), Some(SampleTrackStatus::Muted));

        // must not panic
        machine.set_track_status(999, SampleTrackStatus::Muted);
    }

    #[test]
    fn set_track_tune_updates_a_supporting_track_and_ignores_bad_index() {
        let mut machine = test_machine();

        // index 0 is kick, which supports tune
        machine.set_track_tune(0, 7.0);
        assert_approx_eq(machine.track_tune(0).unwrap(), 7.0);

        // must not panic
        machine.set_track_tune(999, 7.0);
    }

    #[test]
    fn track_tune_is_none_for_a_type_that_doesnt_support_it() {
        let machine = test_machine();

        // index 3 is rimshot, which has no tune knob
        assert_eq!(machine.track_tune(3), None);
    }

    #[test]
    fn set_track_attack_updates_only_kick() {
        let mut machine = test_machine();

        // index 0 is kick, the only type with an exposed attack knob
        machine.set_track_attack(0, 0.02);
        assert_approx_eq(machine.track_attack(0).unwrap(), 0.02);

        // index 4 is a tom -- has decay, but no attack knob
        machine.set_track_attack(4, 0.02);
        assert_eq!(machine.track_attack(4), None);
    }

    #[test]
    fn set_track_decay_updates_a_supporting_track_and_ignores_bad_index() {
        let mut machine = test_machine();

        // index 7 is closed hi-hat, which has decay but no attack
        machine.set_track_decay(7, 0.3);
        assert_approx_eq(machine.track_decay(7).unwrap(), 0.3);

        // must not panic
        machine.set_track_decay(999, 0.3);
    }

    #[test]
    fn set_track_snappy_updates_only_snare() {
        let mut machine = test_machine();

        // index 1 is snare, the only type with a snappy knob
        machine.set_track_snappy(1, 0.6);
        assert_approx_eq(machine.track_snappy(1).unwrap(), 0.6);

        // index 0 is kick -- no snappy knob
        machine.set_track_snappy(0, 0.6);
        assert_eq!(machine.track_snappy(0), None);
    }

    #[test]
    fn any_solo_and_get_solo_tracks_reflect_status() {
        let mut machine = test_machine();

        assert!(!machine.any_solo());
        assert!(machine.get_solo_tracks().iter().all(|&solo| !solo));

        machine.set_track_status(2, SampleTrackStatus::Solo);

        assert!(machine.any_solo());

        let solo_tracks = machine.get_solo_tracks();
        assert!(solo_tracks[2]);
        assert!(!solo_tracks[0]);
    }

    #[test]
    fn play_pause_stop_update_status() {
        let mut machine = test_machine();

        assert_eq!(machine.sequencer_status(), SequencerStatus::Stop);

        machine.play();
        assert_eq!(machine.sequencer_status(), SequencerStatus::Play);

        machine.pause();
        assert_eq!(machine.sequencer_status(), SequencerStatus::Pause);

        machine.stop();
        assert_eq!(machine.sequencer_status(), SequencerStatus::Stop);
    }

    #[test]
    fn stop_resets_sequencer_position_but_pause_does_not() {
        let mut machine = test_machine();

        machine.play();
        machine.next_sample(); // step_phase 0 -> 0.5, still step 0
        machine.next_sample(); // step_phase -> 1.0, wraps, current_step -> 1

        assert_eq!(machine.current_step(), 1);

        machine.pause();
        assert_eq!(machine.current_step(), 1); // untouched by pause

        machine.stop();
        assert_eq!(machine.current_step(), 0); // reset by stop
    }

    #[test]
    fn bpm_set_bpm_delegate_to_sequencer() {
        let mut machine = test_machine();

        assert_approx_eq(machine.bpm(), 120.0);

        machine.set_bpm(140.0);
        assert_approx_eq(machine.bpm(), 140.0);
    }

    #[test]
    fn set_step_updates_active_step_tracks() {
        let mut machine = test_machine();

        machine.set_step(0, 0, true);

        assert!(machine.active_step_tracks()[0]);
    }

    #[test]
    fn set_step_ignores_out_of_range_indices() {
        let mut machine = test_machine();

        // must not panic
        machine.set_step(999, 0, true);
        machine.set_step(0, 999, true);
    }

    #[test]
    fn trigger_track_retriggers_only_that_track() {
        let mut machine = test_machine();

        for _ in 0..3 {
            machine.next_sample(); // drain every track's 2-sample buffer to silence
        }

        machine.trigger_track(0); // kick

        let output = machine.next_sample();

        // kick's own first sample (7.0) * its volume (1.0) * master (0.5);
        // every other track is still silent from the drain above
        assert_approx_eq(output, 7.0 * 1.0 * 0.5);
    }

    #[test]
    fn trigger_track_ignores_out_of_range_index() {
        let mut machine = test_machine();

        // must not panic
        machine.trigger_track(999);
    }

    #[test]
    fn sequencer_triggers_flagged_tracks_when_step_advances() {
        let mut machine = test_machine();

        for _ in 0..3 {
            machine.next_sample(); // drain everything to silence first
        }

        machine.set_step(1, 0, true); // kick fires on step 1

        machine.play();
        machine.next_sample(); // step_phase -> 0.5, still step 0
        let triggered = machine.next_sample(); // crosses into step 1, kick triggers

        assert_eq!(machine.current_step(), 1);
        assert_approx_eq(triggered, 7.0 * 1.0 * 0.5);
    }

    #[test]
    fn trigger_step_fires_flagged_tracks_at_a_given_step() {
        let mut machine = test_machine();

        for _ in 0..3 {
            machine.next_sample(); // drain everything to silence first
        }

        machine.set_step(5, 0, true); // kick flagged on step 5, clock never gets there

        machine.trigger_step(5);

        let output = machine.next_sample();

        // every other track is still silent from the drain above
        assert_approx_eq(output, 7.0 * 1.0 * 0.5);
    }

    #[test]
    fn trigger_step_ignores_out_of_range_step() {
        let mut machine = test_machine();

        // must not panic
        machine.trigger_step(999);
    }

    #[test]
    fn trigger_step_does_not_move_current_step() {
        let mut machine = test_machine();

        // reads an externally-supplied step -- it must not advance
        // self.sequencer's own clock as a side effect
        machine.trigger_step(3);

        assert_eq!(machine.current_step(), 0);
    }

    #[test]
    fn solo_restricts_mix_to_soloed_tracks_only() {
        let mut machine = test_machine();

        // every track starts "live" at position 0 on a fresh machine;
        // soloing snare (index 1) should exclude kick's (index 0) output
        // from the mix even though kick is still genuinely playing
        machine.set_track_status(1, SampleTrackStatus::Solo);

        let output = machine.next_sample();

        assert_approx_eq(output, 7.0 * 1.0 * 0.5);
    }

    #[test]
    fn master_volume_scales_output() {
        let mut machine = test_machine();

        machine.set_master_volume(0.25);

        // Kick/Tom/HiHat tracks now carry an ADEnvelope that starts Idle
        // (silent) until triggered, unlike SamplePlayer which is always
        // "live" from position 0 -- trigger every track explicitly so
        // this test still measures the full 11-track mix
        for index in 0..machine.track_count() {
            machine.trigger_track(index);
        }

        let output = machine.next_sample();

        // at this sample rate ADEnvelope's default attack completes
        // within a single sample, so every track is at full level here
        assert_approx_eq(output, 7.0 * 11.0 * 0.25);
    }

    #[test]
    fn pause_stops_new_triggers_but_lets_existing_sound_continue() {
        let mut machine = test_machine();

        for _ in 0..3 {
            machine.next_sample(); // drain everything to silence first
        }

        // snare (index 1), not kick: it has no ADEnvelope on it, so both of
        // its raw samples play back at the same level and this test stays
        // focused purely on play/pause/ring-out timing
        machine.set_step(1, 1, true);

        machine.play();
        machine.next_sample(); // still step 0
        let triggered = machine.next_sample(); // crosses into step 1, snare triggers

        assert_approx_eq(triggered, 7.0 * 1.0 * 0.5);

        machine.pause();

        // snare's second and final real sample still plays out even while
        // paused -- pause only blocks *new* triggers, not existing ones
        let still_ringing = machine.next_sample();
        assert_approx_eq(still_ringing, 7.0 * 1.0 * 0.5);

        // by now snare's 2-sample buffer is exhausted -- it goes silent on
        // its own, not because pause forced it to
        let now_silent = machine.next_sample();
        assert_approx_eq(now_silent, 0.0);
    }
}
