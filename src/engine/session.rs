use crate::acid_bass::acid_step::AcidStep;
use crate::arp::arp::Arp;
use crate::dsp::types::{Frequency, Sample, SampleRate};
use crate::engine::acid_synth::AcidSynth;
use crate::engine::drum_machine::DrumMachine;
use crate::engine::mixer_engine::MixerEngine;
use crate::engine::synth::Synth;
use crate::mixer::instrument_channel::InstrumentChannel;
use crate::sequencing::transport::{SequencerStatus, Transport};

// channel order into MixerEngine -- keep this in sync with wherever
// mixer_channel_mut() gets called from (UI/wasm). pub(crate) so the wasm
// binding layer can address channels by name instead of a raw index.
pub(crate) const DRUM_CHANNEL: usize = 0;
pub(crate) const ACID_CHANNEL: usize = 1;
pub(crate) const SYNTH_CHANNEL: usize = 2;
pub(crate) const ARP_CHANNEL: usize = 3;
const CHANNEL_COUNT: usize = 4;

// The top-level orchestrator for the unified page: one shared Transport
// driving DrumMachine and AcidSynth together, Synth playing live on top
// (never touches the transport at all), and MixerEngine combining all
// three into the final output. This is what a future wasm binding would
// wrap for the merged synth+drums+acid+mixer page -- replacing the
// three separate SynthEngine/DrumMachineEngine/AcidSynthEngine bindings
// (and their three separate AudioWorkletProcessors) with one.
//
// NOTE: DrumMachine and AcidSynth still own their own internal clocks
// today (see the TODO(session-refactor) notes atop drum_machine.rs and
// acid_synth.rs) -- Session below assumes that's been dealt with and
// they're being driven purely via trigger_step(). Wiring this up for
// real means either finishing that refactor first, or having Session
// simply never call their play()/pause()/stop() and trusting
// trigger_step() alone, which works but leaves those now-pointless
// methods reachable and a stray sequencer_status field ticking away
// unused inside each -- worth cleaning up before this is done, not
// necessarily before it's started.
pub struct Session {
    transport: Transport,
    drum_machine: DrumMachine,
    acid_synth: AcidSynth,
    synth: Synth,
    arp: Arp,
    mixer: MixerEngine,
}

impl Session {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rate: SampleRate,
        num_steps: usize,
        bpm: f32,
        steps_per_beat: Option<usize>,
        voice_count: usize,
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
            transport: Transport::new(rate, num_steps, bpm, steps_per_beat),
            drum_machine: DrumMachine::new(
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
            acid_synth: AcidSynth::new(rate, num_steps, bpm, steps_per_beat),
            synth: Synth::new(rate, voice_count),
            arp: Arp::new(rate, bpm, num_steps),
            mixer: MixerEngine::new(CHANNEL_COUNT, rate),
        }
    }

    // transport controls -- shared across both sequenced instruments;
    // Synth is never touched by any of these, it's always live
    //
    // Fires whatever's already programmed on the current step
    // immediately, rather than waiting for the next natural step
    // boundary (which is the *next* step, or -- if nothing else is
    // programmed -- a full pattern wrap all the way back around).
    // Barely noticeable for Drums/Bass, since another active step
    // almost always fires within a fraction of a second regardless;
    // very noticeable for Arp if only the current step has a chord on
    // it, since without this "press Play" would otherwise mean total
    // silence until the whole 16-step pattern wraps around once.
    pub fn play(&mut self) {
        self.transport.play();
        self.arp.play();
        self.trigger_current_step();
    }

    pub fn pause(&mut self) {
        self.transport.pause();
        self.arp.pause();
    }

    pub fn stop(&mut self) {
        self.transport.stop();
        self.arp.stop();
    }

    pub fn bpm(&self) -> f32 {
        self.transport.bpm()
    }

    // Arp isn't sequenced by the shared Transport the way Drums/Bass
    // are (see arp.rs -- it runs its own StepClock), and the mixer's
    // Beat FX send isn't sequenced by anything at all (see
    // mixer/effects_unit.rs), but both should still track the same
    // tempo as everything else, so set_bpm pushes to all three rather
    // than leaving either to drift independently.
    pub fn set_bpm(&mut self, bpm: f32) {
        self.transport.set_bpm(bpm);
        self.arp.set_bpm(bpm);
        self.mixer.effects_unit_mut().set_bpm(bpm);
    }

    pub fn current_step(&self) -> usize {
        self.transport.current_step()
    }

    pub fn sequencer_status(&self) -> SequencerStatus {
        self.transport.status()
    }

    // full-instrument access -- exposes each engine directly rather than
    // Session re-exposing its whole knob/param surface as passthroughs,
    // same reasoning as mixer_channel_mut/eq_mut below. Session itself
    // only needs to care about the cross-cutting concerns above
    // (transport, live note passthrough, pattern edits, mixer access);
    // everything instrument-specific (track params, voice knobs, waveform,
    // ...) goes through these instead.
    pub fn drum_machine(&self) -> &DrumMachine {
        &self.drum_machine
    }

    pub fn drum_machine_mut(&mut self) -> &mut DrumMachine {
        &mut self.drum_machine
    }

    pub fn acid_synth(&self) -> &AcidSynth {
        &self.acid_synth
    }

    pub fn acid_synth_mut(&mut self) -> &mut AcidSynth {
        &mut self.acid_synth
    }

    pub fn synth_mut(&mut self) -> &mut Synth {
        &mut self.synth
    }

    pub fn arp(&self) -> &Arp {
        &self.arp
    }

    pub fn arp_mut(&mut self) -> &mut Arp {
        &mut self.arp
    }

    pub fn mixer(&self) -> &MixerEngine {
        &self.mixer
    }

    pub fn mixer_mut(&mut self) -> &mut MixerEngine {
        &mut self.mixer
    }

    // live synth playing -- passes straight through, unaffected by the
    // transport
    pub fn note_on(&mut self, frequency: Frequency) {
        self.synth.note_on(frequency);
    }

    pub fn note_off(&mut self, frequency: Frequency) {
        self.synth.note_off(frequency);
    }

    // live drum pad hits -- same idea as note_on/note_off for Synth:
    // independent of the sequencer/transport entirely. Works regardless
    // of transport state since next_sample() below always mixes
    // drum_machine's output, so a track triggered while paused/stopped
    // still rings out.
    pub fn trigger_drum_track(&mut self, index: usize) {
        self.drum_machine.trigger_track(index);
    }

    // pattern editing -- passes straight through to whichever instrument
    // owns that pattern; DrumMachine/AcidSynth still validate bounds
    // themselves the same way they already do today
    pub fn set_drum_step(&mut self, step: usize, track: usize, active: bool) {
        self.drum_machine.set_step(step, track, active);
    }

    pub fn set_acid_step(&mut self, index: usize, step: AcidStep) {
        self.acid_synth.set_step(index, step);
    }

    pub fn set_arp_step(&mut self, index: usize, notes: Vec<Frequency>) {
        self.arp.set_step(index, notes);
    }

    // mixer access -- exposes the channel so callers reach its own
    // mute/solo/volume/eq setters directly, same reasoning as
    // InstrumentChannel::eq_mut not re-exposing Eq3's whole API. Use the
    // DRUM_CHANNEL/ACID_CHANNEL/SYNTH_CHANNEL constants above rather
    // than raw indices when calling this internally.
    pub fn mixer_channel_mut(&mut self, index: usize) -> Option<&mut InstrumentChannel> {
        self.mixer.channel_mut(index)
    }

    // fans a step index out to every sequenced instrument -- shared by
    // next_sample()'s own boundary-crossed trigger below and by play()
    // above, which needs the exact same fan-out for the step the
    // transport is already sitting on
    fn trigger_current_step(&mut self) {
        let step = self.transport.current_step();
        self.drum_machine.trigger_step(step);
        self.acid_synth.trigger_step(step);
        self.arp.trigger_step(step);
    }

    pub fn next_sample(&mut self) -> (Sample, Sample) {
        if self.transport.advance() {
            self.trigger_current_step();
        }

        let mut inputs = [0.0; CHANNEL_COUNT];
        inputs[DRUM_CHANNEL] = self.drum_machine.next_sample();
        inputs[ACID_CHANNEL] = self.acid_synth.next_sample();
        inputs[SYNTH_CHANNEL] = self.synth.next_sample();
        inputs[ARP_CHANNEL] = self.arp.next_sample();

        self.mixer.process(&inputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mixer::instrument_channel::InstrumentChannelStatus;

    // rate = 16.0, num_steps = 16, bpm = 120.0 -- must match
    // DrumMachine::new's own hardcoded 16-step/120bpm Sequencer, since
    // Session's Transport and DrumMachine's internal clock are two
    // separate StepClocks that only stay in lockstep if both are
    // configured the same way. Gives samples_per_step = 2.0 for both,
    // same reasoning as drum_machine's own tests. Every drum track gets
    // the same 2-sample [7.0, 7.0] one-shot, so a fresh session's very
    // first sample is not silence -- kick etc. start "live" at position 0.
    fn test_session() -> Session {
        let data = vec![7.0, 7.0];

        Session::new(
            16.0,
            16,
            120.0,
            None,
            1,
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
        let session = test_session();

        assert_eq!(session.bpm(), 120.0);
        assert_eq!(session.current_step(), 0);
        assert_eq!(session.sequencer_status(), SequencerStatus::Stop);
    }

    #[test]
    fn sequencer_status_reflects_transport_state() {
        let mut session = test_session();

        session.play();
        assert_eq!(session.sequencer_status(), SequencerStatus::Play);

        session.pause();
        assert_eq!(session.sequencer_status(), SequencerStatus::Pause);

        session.stop();
        assert_eq!(session.sequencer_status(), SequencerStatus::Stop);
    }

    #[test]
    fn drum_machine_accessors_reach_the_real_drum_machine() {
        let mut session = test_session();

        session.drum_machine_mut().set_master_volume(0.25);

        assert_eq!(session.drum_machine().master_volume(), 0.25);
    }

    #[test]
    fn mixer_accessors_reach_the_real_mixer() {
        let mut session = test_session();

        session.mixer_mut().set_master_volume(0.25);

        assert_eq!(session.mixer().master_volume(), 0.25);
    }

    #[test]
    fn acid_synth_accessors_reach_the_real_acid_synth() {
        let mut session = test_session();

        session.acid_synth_mut().set_master_volume(0.25);

        assert_eq!(session.acid_synth().master_volume(), 0.25);
    }

    #[test]
    fn arp_accessors_reach_the_real_arp() {
        let mut session = test_session();

        session.arp_mut().set_master_volume(0.25);

        assert_eq!(session.arp().master_volume(), 0.25);
    }

    #[test]
    fn set_bpm_also_updates_the_arps_tempo() {
        let mut session = test_session();

        session.set_bpm(140.0);

        assert_eq!(session.arp().bpm(), 140.0);
    }

    #[test]
    fn arp_channel_reaches_the_mixer() {
        let mut session = test_session();

        session.arp_mut().pattern_mut().set_notes(vec![2.0]);

        // arp's own inner clock only ticks while playing (see arp.rs)
        session.play();

        // isolate the arp channel so this only measures whether it's
        // actually wired into the mixer sum, not accidentally silent
        // for some other reason
        session
            .mixer_channel_mut(DRUM_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);
        session
            .mixer_channel_mut(ACID_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);
        session
            .mixer_channel_mut(SYNTH_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);

        let audible = (0..50).any(|_| session.next_sample() != (0.0, 0.0));

        assert!(audible, "expected the arp channel to reach the final mix");
    }

    #[test]
    fn synth_mut_reaches_the_real_synth() {
        let mut session = test_session();

        // isolate the synth channel so this only measures synth_mut()'s effect
        session
            .mixer_channel_mut(DRUM_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);
        session
            .mixer_channel_mut(ACID_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);

        // thin delegation smoke test -- Synth::set_master_volume's own
        // clamping/effect is already covered by engine::synth's own
        // tests, this just proves synth_mut() reaches the same instance
        // next_sample() plays through
        session.synth_mut().set_master_volume(0.0);
        session.note_on(440.0);

        let silent = (0..50).all(|_| session.next_sample() == (0.0, 0.0));

        assert!(silent, "expected synth_mut() to reach the live synth instance");
    }

    #[test]
    fn set_bpm_updates_transport_bpm() {
        let mut session = test_session();

        session.set_bpm(140.0);

        assert_eq!(session.bpm(), 140.0);
    }

    #[test]
    fn play_pause_stop_transition_current_step_correctly() {
        let mut session = test_session();

        assert_eq!(session.current_step(), 0);

        session.play();
        session.next_sample(); // step_phase 0 -> 0.5, still step 0
        session.next_sample(); // step_phase -> 1.0, wraps, current_step -> 1

        assert_eq!(session.current_step(), 1);

        session.pause();
        assert_eq!(session.current_step(), 1); // untouched by pause

        session.stop();
        assert_eq!(session.current_step(), 0); // reset by stop
    }

    #[test]
    fn advance_does_nothing_while_stopped() {
        let mut session = test_session();

        for _ in 0..10 {
            session.next_sample();
        }

        assert_eq!(session.current_step(), 0);
    }

    #[test]
    fn transport_triggers_both_drum_and_acid_on_the_same_step_boundary() {
        let mut session = test_session();

        for _ in 0..3 {
            session.next_sample(); // drain the drum tracks' initial "live" buffers
        }

        session.set_drum_step(1, 0, true); // kick fires on step 1

        let mut acid_step = AcidStep::new();
        acid_step.set_note(440.0);
        acid_step.set_gate(true);
        session.set_acid_step(1, acid_step); // acid note fires on step 1 too

        session.play();
        session.next_sample(); // still step 0
        let output = session.next_sample(); // crosses into step 1, both trigger

        assert_eq!(session.current_step(), 1);
        assert_ne!(output, (0.0, 0.0));
    }

    #[test]
    fn transport_also_triggers_the_arp_on_the_same_step_boundary() {
        let mut session = test_session();

        // chord slot 1 (of 4) -- at the default chord_division, that's
        // a boundary every 4 raw steps, so this one lands at raw step 4
        session.set_arp_step(1, vec![2.0]);

        session.play();

        // samples_per_step is 2.0 at this test's rate/bpm (see
        // play_pause_stop_transition_current_step_correctly above) --
        // 8 calls crosses from step 0 to step 4, landing exactly on
        // slot 1's boundary
        for _ in 0..8 {
            session.next_sample();
        }

        assert_eq!(session.current_step(), 4);

        // isolate the arp channel -- its own inner arpeggiation clock
        // (separate from the shared Transport) still needs a few more
        // samples after the chord swap to actually trigger a note, so
        // this checks audibility across a short window rather than the
        // exact step-crossing sample
        session
            .mixer_channel_mut(DRUM_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);
        session
            .mixer_channel_mut(ACID_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);
        session
            .mixer_channel_mut(SYNTH_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);

        let audible = (0..50).any(|_| session.next_sample() != (0.0, 0.0));

        assert!(
            audible,
            "expected the arp's chord (set on slot 1) to become audible once the shared transport reached its boundary step"
        );
    }

    #[test]
    fn play_immediately_triggers_whatever_is_on_the_current_step() {
        let mut session = test_session();

        // step 0 is where the transport already sits before any step
        // boundary is ever crossed -- without play() firing it
        // directly, this chord wouldn't trigger until the pattern
        // wrapped all the way back around (a full 16 steps)
        session.set_arp_step(0, vec![2.0]);

        session
            .mixer_channel_mut(DRUM_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);
        session
            .mixer_channel_mut(ACID_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);
        session
            .mixer_channel_mut(SYNTH_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);

        session.play();

        let audible = (0..50).any(|_| session.next_sample() != (0.0, 0.0));

        assert!(
            audible,
            "expected play() to trigger step 0's chord immediately, not wait for a full pattern wrap"
        );
    }

    #[test]
    fn stop_actually_silences_the_arp_even_though_its_clock_is_independent() {
        // Arp's own inner arpeggiation clock is separate from the
        // shared Transport (see arp.rs), so it doesn't get silenced
        // just because Transport::advance() stops returning true --
        // Session::stop() has to reach into Arp explicitly too.
        let mut session = test_session();

        session.set_arp_step(0, vec![2.0]);

        session
            .mixer_channel_mut(DRUM_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);
        session
            .mixer_channel_mut(ACID_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);
        session
            .mixer_channel_mut(SYNTH_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);

        session.play();

        // let the arp actually start sounding first
        for _ in 0..30 {
            session.next_sample();
        }

        session.stop();

        // release is 0.2s hardcoded in Envelope::new -- comfortably
        // covered by a generous window at this test's tiny sample rate
        let mut last = (1.0, 1.0);
        for _ in 0..64 {
            last = session.next_sample();
        }

        assert_eq!(
            last,
            (0.0, 0.0),
            "expected stop() to fully silence the arp, not just the shared transport"
        );
    }

    #[test]
    fn trigger_drum_track_plays_live_independent_of_the_transport() {
        let mut session = test_session();

        for _ in 0..3 {
            session.next_sample(); // drain initial "live" buffers first
        }

        session.trigger_drum_track(0); // kick -- transport is never played

        assert_ne!(session.next_sample(), (0.0, 0.0));
    }

    #[test]
    fn trigger_drum_track_ignores_out_of_range_index() {
        let mut session = test_session();

        // must not panic
        session.trigger_drum_track(999);
    }

    #[test]
    fn set_drum_step_and_set_acid_step_ignore_out_of_range_indices() {
        let mut session = test_session();

        // must not panic
        session.set_drum_step(999, 0, true);
        session.set_acid_step(999, AcidStep::new());
    }

    #[test]
    fn mixer_channel_mut_returns_none_for_out_of_range_index() {
        let mut session = test_session();

        assert!(session.mixer_channel_mut(999).is_none());
    }

    #[test]
    fn mixer_channel_mut_reaches_the_real_drum_channel() {
        let mut session = test_session();

        for _ in 0..3 {
            session.next_sample();
        }
        session.trigger_drum_track(0);

        session
            .mixer_channel_mut(DRUM_CHANNEL)
            .unwrap()
            .set_status(InstrumentChannelStatus::Muted);

        // drum is the only channel producing sound here -- muting it
        // through the mixer should silence the whole mix
        assert_eq!(session.next_sample(), (0.0, 0.0));
    }

    #[test]
    fn note_on_makes_the_synth_audible() {
        let mut session = test_session();

        session.note_on(440.0);

        // Sine starts at phase 0.0 (silent on the very first sample
        // regardless of the envelope), same caveat as acid_voice's own
        // tests -- check across a short window instead of one sample
        let audible = (0..50).any(|_| session.next_sample() != (0.0, 0.0));

        assert!(audible, "expected note_on to make the synth audible");
    }
}
