use crate::dsp::types::Frequency;

// One step of a monophonic acid pattern
#[derive(Clone, Copy, Debug)]
pub struct AcidStep {
    note: Frequency,
    gate: bool,
    accent: bool,
    slide: bool,
}

impl AcidStep {
    pub fn new() -> Self {
        Self {
            gate: false,
            accent: false,
            slide: false,
            note: 440.0,
        }
    }

    // getters
    pub fn note(&self) -> Frequency {
        self.note
    }

    pub fn gate(&self) -> bool {
        self.gate
    }

    pub fn accent(&self) -> bool {
        self.accent
    }

    pub fn slide(&self) -> bool {
        self.slide
    }

    // setters
    pub fn set_note(&mut self, note: Frequency) {
        self.note = note;
    }

    pub fn set_gate(&mut self, gate: bool) {
        self.gate = gate;
    }

    pub fn set_accent(&mut self, accent: bool) {
        self.accent = accent;
    }

    pub fn set_slide(&mut self, slide: bool) {
        self.slide = slide;
    }
}

impl Default for AcidStep {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults_to_a_rest_with_no_flags() {
        let step = AcidStep::new();

        assert!(!step.gate());
        assert!(!step.accent());
        assert!(!step.slide());
    }

    #[test]
    fn new_has_a_sane_default_note() {
        let step = AcidStep::new();

        assert_eq!(step.note(), 440.0);
    }

    #[test]
    fn default_trait_matches_new() {
        let step = AcidStep::default();

        assert_eq!(step.note(), 440.0);
        assert!(!step.gate());
        assert!(!step.accent());
        assert!(!step.slide());
    }

    #[test]
    fn set_note_updates_note() {
        let mut step = AcidStep::new();

        step.set_note(220.0);

        assert_eq!(step.note(), 220.0);
    }

    #[test]
    fn set_gate_updates_gate() {
        let mut step = AcidStep::new();

        step.set_gate(true);
        assert!(step.gate());

        step.set_gate(false);
        assert!(!step.gate());
    }

    #[test]
    fn set_accent_updates_accent() {
        let mut step = AcidStep::new();

        step.set_accent(true);
        assert!(step.accent());

        step.set_accent(false);
        assert!(!step.accent());
    }

    #[test]
    fn set_slide_updates_slide() {
        let mut step = AcidStep::new();

        step.set_slide(true);
        assert!(step.slide());

        step.set_slide(false);
        assert!(!step.slide());
    }

    #[test]
    fn setters_are_independent_of_each_other() {
        // regression-style test: each field's setter should only ever
        // touch its own field, not accidentally clobber a sibling one
        let mut step = AcidStep::new();

        step.set_gate(true);
        step.set_accent(true);
        step.set_slide(true);
        step.set_note(110.0);

        assert!(step.gate());
        assert!(step.accent());
        assert!(step.slide());
        assert_eq!(step.note(), 110.0);
    }
}
