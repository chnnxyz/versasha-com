use crate::dsp::modulation::{Modulation, ModulationTarget};
use crate::dsp::types::Sample;

pub struct ModulationMatrix {
    values: Vec<Modulation>,
}

impl ModulationMatrix {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn push(&mut self, modulation: Modulation) {
        self.values.push(modulation);
    }

    pub fn value(&self, target: ModulationTarget) -> Sample {
        self.values
            .iter()
            .filter(|m| m.target() == target)
            .map(|m| m.value())
            .sum()
    }

    pub fn reset(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::modulation::{Modulation, ModulationSource};

    #[test]
    fn new_matrix_starts_empty() {
        let matrix = ModulationMatrix::new();

        assert_eq!(matrix.value(ModulationTarget::Pitch), 0.0);
    }

    #[test]
    fn push_adds_modulation() {
        let mut matrix = ModulationMatrix::new();

        matrix.push(Modulation::new(
            ModulationSource::Lfo,
            ModulationTarget::Pitch,
            2.0,
        ));

        assert_eq!(matrix.value(ModulationTarget::Pitch), 2.0);
    }

    #[test]
    fn values_are_summed() {
        let mut matrix = ModulationMatrix::new();

        matrix.push(Modulation::new(
            ModulationSource::Lfo,
            ModulationTarget::Pitch,
            2.0,
        ));

        matrix.push(Modulation::new(
            ModulationSource::Envelope,
            ModulationTarget::Pitch,
            3.0,
        ));

        assert_eq!(matrix.value(ModulationTarget::Pitch), 5.0);
    }

    #[test]
    fn different_targets_are_independent() {
        let mut matrix = ModulationMatrix::new();

        matrix.push(Modulation::new(
            ModulationSource::Lfo,
            ModulationTarget::Pitch,
            5.0,
        ));

        matrix.push(Modulation::new(
            ModulationSource::Lfo,
            ModulationTarget::Volume,
            0.5,
        ));

        assert_eq!(matrix.value(ModulationTarget::Pitch), 5.0);
        assert_eq!(matrix.value(ModulationTarget::Volume), 0.5);
        assert_eq!(matrix.value(ModulationTarget::FilterCutoff), 0.0);
    }

    #[test]
    fn clear_removes_all_modulation() {
        let mut matrix = ModulationMatrix::new();

        matrix.push(Modulation::new(
            ModulationSource::Lfo,
            ModulationTarget::Pitch,
            10.0,
        ));

        matrix.clear();

        assert_eq!(matrix.value(ModulationTarget::Pitch), 0.0);
    }

    #[test]
    fn reset_clears_matrix() {
        let mut matrix = ModulationMatrix::new();

        matrix.push(Modulation::new(
            ModulationSource::Lfo,
            ModulationTarget::Pitch,
            10.0,
        ));

        matrix.reset();

        assert_eq!(matrix.value(ModulationTarget::Pitch), 0.0);
    }
}
