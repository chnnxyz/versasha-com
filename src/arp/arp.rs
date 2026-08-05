use crate::arp::arp_step::ArpStep;
use crate::arp::pattern::ArpPattern;
use crate::dsp::oscillators::waveform::Waveform;
use crate::dsp::types::{Frequency, Sample, SampleRate};
use crate::sequencing::note_division::NoteDivision;
use crate::sequencing::step_clock::StepClock;
use crate::synth::voice::SynthVoice;

// 4 chord slots, one per beat of a 4/4 bar -- simpler than a full
// 16-step grid, and lines up with how a bar is actually counted.
const CHORD_COUNT: usize = 4;

// The arpeggiator: CHORD_COUNT chord slots (one per beat), where each
// slot holds a whole chord instead of one note. Session's shared
// Transport drives trigger_step() below on the same clock as
// everything else -- when a raw step lands on a chord boundary (see
// steps_per_chord()) and that slot has notes, those notes become
// ArpPattern's held chord; landing on a boundary with an empty slot
// changes nothing, so whatever chord was last held just keeps
// arpeggiating until a slot with notes comes along. `clock`/`division`
// here are a second, faster clock -- separate from the shared
// Transport -- that decides how quickly ArpPattern cycles through the
// *current* chord's notes; `chord_division` is a third rate, deciding
// how many raw Transport steps each of the CHORD_COUNT slots holds for
// before advancing to the next one (see steps_per_chord()) -- these
// three rates are independent by design: how fast notes arpeggiate
// within a chord has nothing to do with how long that chord is held,
// which has nothing to do with the shared Transport's own step rate.
//
// `playing` mirrors Transport's own play/pause/stop split: while
// false, `clock` never advances and no new notes ever trigger, same as
// Transport::advance() short-circuiting while not Play. Unlike
// Drums/Bass (whose voices are naturally percussive and just ring out
// once new triggers stop coming), Arp's voice is a full ADSR that
// holds its sustain level forever once triggered -- so pause()/stop()
// also call voice.note_off() to actually let it fall silent, not just
// stop retriggering it.
//
// Chords come from a UI-driven piano-roll editor via set_step(), never
// the computer keyboard -- that stays reserved for the live Synth.
pub struct Arp {
    pattern: ArpPattern,
    clock: StepClock,
    voice: SynthVoice,
    division: NoteDivision,
    chord_division: NoteDivision,
    bpm: f32,
    master_volume: f32,
    steps: Vec<ArpStep>,
    playing: bool,

    // how many raw steps the shared Transport considers one full bar --
    // needed by steps_per_chord() below. StepClock has no way to change
    // steps_per_beat after construction (only set_bpm) -- set_division
    // has to rebuild it from scratch, so the sample rate has to survive
    // somewhere past new() too
    bar_steps: usize,
    rate: SampleRate,
}

impl Arp {
    // the arpeggiation clock's own num_steps is always 1 -- it has no
    // fixed pattern length the way the CHORD_COUNT-slot chord grid
    // below does, it just keeps walking ArpPattern's notes forever.
    // Only advance()'s boundary-crossed bool matters here, never
    // current_step(). `num_steps` is the shared Transport's own bar
    // length (16 everywhere else in this codebase).
    pub fn new(rate: SampleRate, bpm: f32, num_steps: usize) -> Self {
        let division = NoteDivision::Quarter;

        Self {
            pattern: ArpPattern::new(),
            clock: StepClock::new(rate, 1, bpm, Some(division.steps_per_beat())),
            voice: SynthVoice::new(rate),
            division,
            chord_division: NoteDivision::Quarter,
            bpm,
            master_volume: 1.0,
            steps: (0..CHORD_COUNT).map(|_| ArpStep::new()).collect(),
            playing: false,
            bar_steps: num_steps,
            rate,
        }
    }

    // full pattern access -- exposes ArpPattern directly rather than
    // Arp re-exposing set_mode/set_octave_range itself, same reasoning
    // as Session's drum_machine_mut()/acid_synth_mut() in
    // engine/session.rs
    pub fn pattern_mut(&mut self) -> &mut ArpPattern {
        &mut self.pattern
    }

    // getters
    pub fn bpm(&self) -> f32 {
        self.bpm
    }

    pub fn division(&self) -> NoteDivision {
        self.division
    }

    pub fn chord_division(&self) -> NoteDivision {
        self.chord_division
    }

    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn step(&self, index: usize) -> Option<&ArpStep> {
        self.steps.get(index)
    }

    pub fn waveform(&self) -> Waveform {
        self.voice.params().osc1.waveform
    }

    // setters
    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
        self.clock.set_bpm(bpm);
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    pub fn set_division(&mut self, division: NoteDivision) {
        self.division = division;
        self.clock = StepClock::new(self.rate, 1, self.bpm, Some(division.steps_per_beat()));
    }

    pub fn set_chord_division(&mut self, division: NoteDivision) {
        self.chord_division = division;
    }

    // both oscillators share one waveform choice -- Arp is a simpler,
    // single-voice instrument, unlike Synth's independent osc1/osc2
    pub fn set_waveform(&mut self, waveform: Waveform) {
        let mut params = self.voice.params();
        params.osc1.waveform = waveform;
        params.osc2.waveform = waveform;
        self.voice.set_params(params);
    }

    // index comes from outside (UI/wasm), so it can be out of range --
    // silently ignored, same convention as DrumMachine::set_step
    pub fn set_step(&mut self, index: usize, notes: Vec<Frequency>) {
        if let Some(step) = self.steps.get_mut(index) {
            step.set_notes(notes);
        }
    }

    // transport controls -- called by Session alongside its own shared
    // Transport's play()/pause()/stop(), since Arp's clock is a second,
    // independent one that Transport has no reach into
    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn pause(&mut self) {
        self.playing = false;
        self.voice.note_off();
    }

    pub fn stop(&mut self) {
        self.playing = false;
        self.voice.note_off();
        self.clock.stop();
    }

    // How many raw shared-Transport steps each of the CHORD_COUNT slots
    // holds for before advancing to the next one. At the default
    // Quarter chord_division that's bar_steps / CHORD_COUNT -- one slot
    // per beat, so the whole grid fills exactly one bar. A faster
    // chord_division (Eighth, Sixteenth, ...) shortens each hold, so
    // the same CHORD_COUNT slots cycle through more than once per bar
    // instead. Clamped to at least 1 so a division steep enough to
    // divide bar_steps down to zero can never cause a divide-by-zero or
    // collapse every slot onto the same raw step.
    fn steps_per_chord(&self) -> usize {
        (self.bar_steps / (CHORD_COUNT * self.chord_division.steps_per_beat())).max(1)
    }

    // Which of the CHORD_COUNT chord slots is current -- or would be --
    // for a given raw shared-Transport step. trigger_step below reuses
    // this same mapping to decide both whether `step` lands on a chord
    // boundary and which slot to swap in; also exposed publicly so the
    // UI can highlight the right slot as the shared playhead moves,
    // without duplicating this math on the JS side.
    pub fn chord_index_for_step(&self, step: usize) -> usize {
        (step / self.steps_per_chord()) % self.steps.len()
    }

    // Called by Session when its shared Transport crosses a step
    // boundary, same as DrumMachine::trigger_step/AcidSynth::trigger_step.
    // Most raw steps land between chord boundaries and are a no-op;
    // landing exactly on one swaps ArpPattern's held chord to that
    // slot's notes, unless the slot is empty, in which case whatever
    // chord was last held just keeps arpeggiating.
    pub fn trigger_step(&mut self, step: usize) {
        if step % self.steps_per_chord() != 0 {
            return;
        }

        let chord_index = self.chord_index_for_step(step);

        if let Some(arp_step) = self.steps.get(chord_index) {
            if !arp_step.notes().is_empty() {
                self.pattern.set_notes(arp_step.notes().to_vec());
            }
        }
    }

    // On a step boundary, pulls the next note from `pattern` and
    // triggers `voice` with it -- no trigger, just the clock still
    // advancing, if pattern.next_note() returns None (nothing held).
    // voice.next_sample() always runs, trigger or not -- same "always
    // advance, decide at output" rule this whole codebase follows
    // everywhere else, so its envelope/filter state never goes stale
    // between triggers. While not playing, clock.advance() is never
    // even called, same short-circuit Transport::advance() uses.
    pub fn next_sample(&mut self) -> Sample {
        if self.playing && self.clock.advance() {
            if let Some(note) = self.pattern.next_note() {
                self.voice.note_on(note);
            }
        }

        self.voice.next_sample() * self.master_volume
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // rate = 16.0, bpm = 60.0, num_steps (bar_steps) = 16, default
    // Quarter division/chord_division -- steps_per_chord() works out to
    // 16 / (4 * 1) = 4 raw steps per chord slot
    fn test_arp() -> Arp {
        Arp::new(16.0, 60.0, 16)
    }

    #[test]
    fn new_sets_expected_defaults() {
        let arp = test_arp();

        assert_eq!(arp.bpm(), 60.0);
        assert_eq!(arp.division(), NoteDivision::Quarter);
        assert_eq!(arp.chord_division(), NoteDivision::Quarter);
        assert_eq!(arp.master_volume(), 1.0);
        assert_eq!(arp.step_count(), 4);
        assert_eq!(arp.waveform(), Waveform::Sine);
    }

    #[test]
    fn new_steps_start_empty() {
        let arp = test_arp();

        let step = arp.step(0).unwrap();

        assert!(step.notes().is_empty());
    }

    #[test]
    fn step_returns_none_for_out_of_range_index() {
        let arp = test_arp();

        assert!(arp.step(999).is_none());
    }

    #[test]
    fn set_step_ignores_out_of_range_index() {
        let mut arp = test_arp();

        // must not panic
        arp.set_step(999, vec![100.0]);
    }

    #[test]
    fn set_step_updates_the_step() {
        let mut arp = test_arp();

        arp.set_step(3, vec![100.0, 200.0]);

        let step = arp.step(3).unwrap();
        assert_eq!(step.notes(), &[100.0, 200.0]);
    }

    #[test]
    fn set_waveform_changes_the_getter() {
        let mut arp = test_arp();

        arp.set_waveform(Waveform::Saw);

        assert_eq!(arp.waveform(), Waveform::Saw);
    }

    #[test]
    fn chord_index_for_step_matches_the_default_one_slot_per_beat_mapping() {
        let arp = test_arp();

        assert_eq!(arp.chord_index_for_step(0), 0);
        assert_eq!(arp.chord_index_for_step(3), 0);
        assert_eq!(arp.chord_index_for_step(4), 1);
        assert_eq!(arp.chord_index_for_step(8), 2);
        assert_eq!(arp.chord_index_for_step(12), 3);
        assert_eq!(arp.chord_index_for_step(15), 3);
    }

    #[test]
    fn trigger_step_on_a_chord_boundary_with_notes_swaps_the_held_chord() {
        let mut arp = test_arp();
        arp.set_step(0, vec![300.0]);

        arp.trigger_step(0); // step 0 is always a boundary

        assert_eq!(arp.pattern_mut().notes(), &[300.0]);
    }

    #[test]
    fn trigger_step_between_chord_boundaries_is_a_no_op() {
        let mut arp = test_arp();
        arp.set_step(0, vec![300.0]);
        arp.set_step(1, vec![999.0]); // would only ever be reached on a boundary

        arp.trigger_step(0);
        arp.trigger_step(1); // steps_per_chord() is 4 -- 1 is not a boundary

        assert_eq!(
            arp.pattern_mut().notes(),
            &[300.0],
            "a raw step between chord boundaries must not change the currently-held chord"
        );
    }

    #[test]
    fn trigger_step_on_a_boundary_with_an_empty_slot_leaves_the_held_chord_unchanged() {
        let mut arp = test_arp();
        arp.set_step(0, vec![300.0]);
        // slot 1 stays empty

        arp.trigger_step(0);
        arp.trigger_step(4); // next boundary -- maps to slot 1, which is empty

        assert_eq!(
            arp.pattern_mut().notes(),
            &[300.0],
            "an empty chord slot must not change the currently-held chord"
        );
    }

    #[test]
    fn trigger_step_walks_all_four_slots_across_a_bar() {
        let mut arp = test_arp();
        arp.set_step(0, vec![100.0]);
        arp.set_step(1, vec![200.0]);
        arp.set_step(2, vec![300.0]);
        arp.set_step(3, vec![400.0]);

        arp.trigger_step(0);
        assert_eq!(arp.pattern_mut().notes(), &[100.0]);

        arp.trigger_step(4);
        assert_eq!(arp.pattern_mut().notes(), &[200.0]);

        arp.trigger_step(8);
        assert_eq!(arp.pattern_mut().notes(), &[300.0]);

        arp.trigger_step(12);
        assert_eq!(arp.pattern_mut().notes(), &[400.0]);
    }

    #[test]
    fn trigger_step_never_panics_for_large_step_values() {
        let mut arp = test_arp();

        // must not panic -- chord_index_for_step's modulo always keeps
        // the lookup in bounds regardless of how large step gets
        arp.trigger_step(999_999);
    }

    #[test]
    fn set_chord_division_updates_the_getter() {
        let mut arp = test_arp();

        arp.set_chord_division(NoteDivision::Sixteenth);

        assert_eq!(arp.chord_division(), NoteDivision::Sixteenth);
    }

    #[test]
    fn set_chord_division_changes_how_many_raw_steps_each_slot_spans() {
        let mut arp = test_arp();
        arp.set_step(0, vec![100.0]);
        arp.set_step(1, vec![200.0]);

        arp.set_chord_division(NoteDivision::Eighth); // steps_per_chord() -> 16 / (4 * 2) = 2

        arp.trigger_step(0);
        assert_eq!(arp.pattern_mut().notes(), &[100.0]);

        // step 2 would NOT be a boundary under the default Quarter
        // (needs steps_per_chord() == 4), but IS one now
        arp.trigger_step(2);
        assert_eq!(arp.pattern_mut().notes(), &[200.0]);
    }

    #[test]
    fn set_master_volume_clamps_and_scales_output() {
        let mut arp = test_arp();
        arp.play();
        arp.pattern_mut().set_notes(vec![2.0]);
        arp.set_master_volume(2.0);
        assert_eq!(arp.master_volume(), 1.0);

        arp.set_master_volume(0.0);
        assert_eq!(arp.master_volume(), 0.0);

        // even after a step boundary triggers a note, zero master
        // volume means silence
        for _ in 0..32 {
            assert_eq!(arp.next_sample(), 0.0);
        }
    }

    #[test]
    fn set_bpm_updates_the_getter() {
        let mut arp = test_arp();

        arp.set_bpm(140.0);

        assert_eq!(arp.bpm(), 140.0);
    }

    #[test]
    fn set_bpm_actually_changes_the_step_rate() {
        let mut arp = test_arp(); // 60 bpm -> samples_per_step = 16.0
        arp.play();
        arp.pattern_mut().set_notes(vec![2.0]);

        arp.set_bpm(120.0); // double the tempo -> samples_per_step becomes 8.0

        for _ in 0..7 {
            assert_eq!(arp.next_sample(), 0.0);
        }

        // tight window: long enough to catch the correct (doubled-rate)
        // trigger at call 8, but short enough to end before the stale,
        // un-doubled rate would have triggered at call 16 -- if set_bpm
        // only updated the getter and never reached self.clock, every
        // one of these would still be silent
        let audible = (0..8).any(|_| arp.next_sample() != 0.0);

        assert!(audible, "expected the doubled bpm to trigger a note twice as fast");
    }

    #[test]
    fn set_division_updates_the_getter() {
        let mut arp = test_arp();

        arp.set_division(NoteDivision::Sixteenth);

        assert_eq!(arp.division(), NoteDivision::Sixteenth);
    }

    #[test]
    fn voice_keeps_advancing_after_the_pattern_stops_producing_triggers() {
        // a low note relative to the sample rate, so its oscillator
        // phase moves cleanly instead of aliasing wildly
        let mut arp = test_arp();
        arp.play();
        arp.pattern_mut().set_notes(vec![2.0]);

        // cross the first step boundary to trigger a note, then clear
        // the pattern so no future call will ever trigger another one
        for _ in 0..16 {
            arp.next_sample();
        }
        arp.pattern_mut().set_notes(vec![]);

        // with no more triggers coming, the oscillator's phase should
        // still move from sample to sample -- if voice.next_sample()
        // were ever skipped instead of always running, every call from
        // here on would return the exact same frozen value
        let first = arp.next_sample();
        let second = arp.next_sample();

        assert_ne!(
            first, second,
            "expected the voice to keep advancing even after the pattern stopped producing new triggers"
        );
    }

    #[test]
    fn next_sample_stays_silent_when_no_notes_are_held() {
        let mut arp = test_arp();
        arp.play();
        arp.pattern_mut().set_notes(vec![]);

        for _ in 0..100 {
            assert_eq!(arp.next_sample(), 0.0);
        }
    }

    #[test]
    fn next_sample_stays_silent_before_the_first_step_boundary() {
        let mut arp = test_arp();
        arp.play();
        arp.pattern_mut().set_notes(vec![440.0]);

        // samples_per_step is 16.0 -- nothing should trigger before that
        for _ in 0..15 {
            assert_eq!(arp.next_sample(), 0.0);
        }
    }

    #[test]
    fn next_sample_triggers_a_note_on_the_first_step_boundary() {
        let mut arp = test_arp();
        arp.play();
        arp.pattern_mut().set_notes(vec![440.0]);

        // Sine starts at phase 0.0 (silent on the very first sample
        // regardless of the envelope), same caveat as everywhere else
        // in this codebase that tests a freshly-triggered voice -- check
        // across a window instead of the one triggering sample
        let audible = (0..50).any(|_| arp.next_sample() != 0.0);

        assert!(audible, "expected the first step boundary to trigger an audible note");
    }

    #[test]
    fn set_division_changes_how_often_next_sample_triggers() {
        let mut arp = test_arp();
        arp.play();
        arp.pattern_mut().set_notes(vec![2.0]);

        // Eighth halves steps_per_beat's samples_per_step from 16.0 to 8.0
        arp.set_division(NoteDivision::Eighth);

        for _ in 0..7 {
            assert_eq!(arp.next_sample(), 0.0);
        }

        // tight window -- see set_bpm_actually_changes_the_step_rate's
        // own comment for why this can't just be a generous "audible
        // somewhere in the next 50 calls" check
        let audible = (0..8).any(|_| arp.next_sample() != 0.0);

        assert!(audible, "expected the faster division to still trigger a note");
    }

    #[test]
    fn new_arp_does_not_play_until_told_to() {
        let mut arp = test_arp();
        arp.pattern_mut().set_notes(vec![440.0]);

        // never called play() -- the clock must never advance, no
        // matter how many samples are pulled
        for _ in 0..100 {
            assert_eq!(arp.next_sample(), 0.0);
        }
    }

    #[test]
    fn pause_stops_new_triggers_and_silences_the_current_note() {
        let mut arp = test_arp();
        arp.play();
        arp.pattern_mut().set_notes(vec![2.0]);

        // cross the first step boundary so the voice is actually sounding
        for _ in 0..16 {
            arp.next_sample();
        }

        arp.pause();

        // release is 0.2s hardcoded in Envelope::new -- at this test's
        // 16Hz rate that's ~3-4 samples, so a generous window comfortably
        // covers it reaching true silence and staying there
        let mut last = 1.0;
        for _ in 0..64 {
            last = arp.next_sample();
        }

        assert_eq!(last, 0.0, "expected the voice to fully release after pause()");
    }

    #[test]
    fn stop_stops_new_triggers_and_silences_the_current_note() {
        let mut arp = test_arp();
        arp.play();
        arp.pattern_mut().set_notes(vec![2.0]);

        for _ in 0..16 {
            arp.next_sample();
        }

        arp.stop();

        let mut last = 1.0;
        for _ in 0..64 {
            last = arp.next_sample();
        }

        assert_eq!(last, 0.0, "expected the voice to fully release after stop()");
    }

    #[test]
    fn play_after_stop_can_trigger_again() {
        let mut arp = test_arp();
        arp.pattern_mut().set_notes(vec![2.0]);

        arp.play();
        arp.stop();
        arp.play();

        // samples_per_step is 16.0 -- crossing it again after a fresh
        // play() should retrigger just like a brand new Arp would
        let audible = (0..30).any(|_| arp.next_sample() != 0.0);

        assert!(audible, "expected play() after stop() to be able to trigger again");
    }
}
