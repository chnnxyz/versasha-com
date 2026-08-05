use crate::dsp::types::Frequency;

// One step of Arp's 16-step pattern -- unlike AcidStep's single note,
// this holds a whole chord, since a step's job here is picking which
// chord ArpPattern arpeggiates, not playing one note directly. No
// separate active flag -- a step with notes is what plays, an empty
// step is a no-op (see Arp::trigger_step).
#[derive(Clone, Debug)]
pub struct ArpStep {
    notes: Vec<Frequency>,
}

impl ArpStep {
    pub fn new() -> Self {
        Self { notes: Vec::new() }
    }

    // getters
    pub fn notes(&self) -> &[Frequency] {
        &self.notes
    }

    // setters
    pub fn set_notes(&mut self, notes: Vec<Frequency>) {
        self.notes = notes;
    }
}

impl Default for ArpStep {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_defaults() {
        let step = ArpStep::new();

        assert!(step.notes().is_empty());
    }

    #[test]
    fn set_notes_updates_value() {
        let mut step = ArpStep::new();

        step.set_notes(vec![100.0, 200.0, 300.0]);

        assert_eq!(step.notes(), &[100.0, 200.0, 300.0]);
    }
}
