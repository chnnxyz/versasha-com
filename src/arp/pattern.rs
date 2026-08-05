use crate::dsp::noise::NoiseGenerator;
use crate::dsp::types::Frequency;

// positive integer seed for random pattern
const RANDOM_MODE_SEED: u32 = 7;

// up: increasing freq.
// down: decreasing freq
// updown: increase, reach limit, decrease.
// random: self explanatory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpMode {
    Up,
    Down,
    UpDown,
    Random,
}

// Chord pattern, mode and octave holder. rng just holds info for random
// arpeggiation.
pub struct ArpPattern {
    notes: Vec<Frequency>,
    mode: ArpMode,
    octave_range: u8,

    index: usize,

    rng: NoiseGenerator,
    shuffled_order: Vec<Frequency>,
}

impl ArpPattern {
    pub fn new() -> Self {
        Self {
            notes: vec![Frequency::default()],
            mode: ArpMode::Up,
            octave_range: 2,
            index: 0,
            rng: NoiseGenerator::new(RANDOM_MODE_SEED),
            shuffled_order: Vec::new(),
        }
    }

    // getters
    pub fn notes(&self) -> &[Frequency] {
        &self.notes
    }

    pub fn mode(&self) -> ArpMode {
        self.mode
    }

    pub fn octave_range(&self) -> u8 {
        self.octave_range
    }

    // setters -- changing any of these resets the walk back to the start

    fn reset_index(&mut self) {
        self.index = 0;
    }

    pub fn set_notes(&mut self, notes: Vec<Frequency>) {
        self.reset_index();
        self.notes = notes;
    }

    pub fn set_mode(&mut self, mode: ArpMode) {
        self.reset_index();
        self.mode = mode;
    }

    pub fn set_octave_range(&mut self, octaves: u8) {
        self.reset_index();
        // 0 would mean repeating the chord zero times -- permanent
        // silence -- so floor it at the base octave
        self.octave_range = octaves.max(1);
    }

    // next note seection. if number of additional octaves is longer than 0
    // that is the next node
    pub fn next_note(&mut self) -> Option<Frequency> {
        if self.notes.is_empty() {
            return None;
        }

        let expanded = self.expand_across_octaves();

        let order = match self.mode {
            ArpMode::Up => expanded,
            ArpMode::Down => expanded.into_iter().rev().collect(),
            ArpMode::UpDown => Self::up_down(expanded),
            ArpMode::Random => {
                // only reshuffle when a new pass starts -- every other
                // call just reuses whatever order we already picked
                if self.index == 0 {
                    self.shuffled_order = Self::shuffle(expanded, &mut self.rng);
                }

                self.shuffled_order.clone()
            }
        };

        if order.is_empty() {
            return None;
        }

        let note = order[self.index % order.len()];
        self.index = (self.index + 1) % order.len();

        Some(note)
    }

    // math helper to just have quick access to next octaves
    fn expand_across_octaves(&self) -> Vec<Frequency> {
        (0..self.octave_range)
            .flat_map(|octave| {
                let multiplier = 2f32.powi(octave as i32);
                self.notes.iter().map(move |&note| note * multiplier)
            })
            .collect()
    }

    //sort the chord down to up and appen dup to down
    fn up_down(ascending: Vec<Frequency>) -> Vec<Frequency> {
        let mut sequence = ascending.clone();

        if ascending.len() > 2 {
            let descending_middle = ascending[1..ascending.len() - 1].iter().rev().copied();
            sequence.extend(descending_middle);
        }

        sequence
    }

    // Standard Fisher-Yates shuffle, just fed by NoiseGenerator's
    // -1..1 float output instead of a normal integer RNG.
    fn shuffle(mut notes: Vec<Frequency>, rng: &mut NoiseGenerator) -> Vec<Frequency> {
        for i in (1..notes.len()).rev() {
            let unipolar = (rng.next_sample() + 1.0) / 2.0;
            let j = ((unipolar * (i + 1) as f32) as usize).min(i);

            notes.swap(i, j);
        }

        notes
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
    fn new_has_defaults() {
        let pattern = ArpPattern::new();

        assert_eq!(pattern.mode(), ArpMode::Up);
        assert_eq!(pattern.octave_range(), 2);
    }

    #[test]
    fn set_octave_range_clamps_to_at_least_one() {
        let mut pattern = ArpPattern::new();

        pattern.set_octave_range(0);

        assert_eq!(pattern.octave_range(), 1);
    }

    #[test]
    fn next_note_returns_none_when_no_notes_are_held() {
        let mut pattern = ArpPattern::new();
        pattern.set_notes(vec![]);

        assert_eq!(pattern.next_note(), None);
    }

    #[test]
    fn set_notes_restarts_the_walk_from_the_beginning() {
        let mut pattern = ArpPattern::new();
        pattern.set_octave_range(1);
        pattern.set_notes(vec![100.0, 200.0, 300.0]);

        assert_approx_eq(pattern.next_note().unwrap(), 100.0);
        assert_approx_eq(pattern.next_note().unwrap(), 200.0);

        // mid-walk -- the next call, without set_notes, would have been 300.0
        pattern.set_notes(vec![9.0, 8.0]);

        assert_approx_eq(pattern.next_note().unwrap(), 9.0);
    }

    #[test]
    fn up_mode_walks_ascending_then_wraps() {
        let mut pattern = ArpPattern::new();
        pattern.set_octave_range(1);
        pattern.set_notes(vec![100.0, 200.0, 300.0]);

        let walked: Vec<f32> = (0..6).map(|_| pattern.next_note().unwrap()).collect();

        assert_eq!(walked, vec![100.0, 200.0, 300.0, 100.0, 200.0, 300.0]);
    }

    #[test]
    fn up_mode_expands_across_octaves() {
        let mut pattern = ArpPattern::new();
        pattern.set_octave_range(2);
        pattern.set_notes(vec![100.0, 150.0]);

        // second octave's frequencies are doubled, not "+12"
        let walked: Vec<f32> = (0..4).map(|_| pattern.next_note().unwrap()).collect();

        assert_eq!(walked, vec![100.0, 150.0, 200.0, 300.0]);
    }

    #[test]
    fn down_mode_walks_descending_then_wraps() {
        let mut pattern = ArpPattern::new();
        pattern.set_octave_range(1);
        pattern.set_notes(vec![100.0, 200.0, 300.0]);
        pattern.set_mode(ArpMode::Down);

        let walked: Vec<f32> = (0..4).map(|_| pattern.next_note().unwrap()).collect();

        assert_eq!(walked, vec![300.0, 200.0, 100.0, 300.0]);
    }

    #[test]
    fn up_down_mode_bounces_without_repeating_the_turnaround_notes() {
        let mut pattern = ArpPattern::new();
        pattern.set_octave_range(1);
        pattern.set_notes(vec![100.0, 200.0, 300.0, 400.0]);
        pattern.set_mode(ArpMode::UpDown);

        let walked: Vec<f32> = (0..7).map(|_| pattern.next_note().unwrap()).collect();

        assert_eq!(
            walked,
            vec![100.0, 200.0, 300.0, 400.0, 300.0, 200.0, 100.0]
        );
    }

    #[test]
    fn up_down_mode_has_no_middle_to_descend_through_with_two_notes() {
        let mut pattern = ArpPattern::new();
        pattern.set_octave_range(1);
        pattern.set_notes(vec![100.0, 200.0]);
        pattern.set_mode(ArpMode::UpDown);

        let walked: Vec<f32> = (0..4).map(|_| pattern.next_note().unwrap()).collect();

        assert_eq!(walked, vec![100.0, 200.0, 100.0, 200.0]);
    }

    #[test]
    fn random_mode_plays_every_note_once_per_pass_before_repeating() {
        let mut pattern = ArpPattern::new();
        pattern.set_octave_range(1);
        pattern.set_notes(vec![100.0, 200.0, 300.0, 400.0]);
        pattern.set_mode(ArpMode::Random);

        let mut first_pass: Vec<f32> = (0..4).map(|_| pattern.next_note().unwrap()).collect();
        let mut second_pass: Vec<f32> = (0..4).map(|_| pattern.next_note().unwrap()).collect();

        first_pass.sort_by(|a, b| a.partial_cmp(b).unwrap());
        second_pass.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let expected = vec![100.0, 200.0, 300.0, 400.0];
        assert_eq!(
            first_pass, expected,
            "every note should appear exactly once in a pass"
        );
        assert_eq!(
            second_pass, expected,
            "every note should appear exactly once in the next pass too"
        );
    }
}
