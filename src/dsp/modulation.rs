use crate::dsp::types::Sample;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModulationTarget {
    Pitch,
    Volume,
    Vibrato,
}

#[derive(Debug, Clone, Copy)]
pub struct Modulation {
    target: ModulationTarget,
    value: Sample,
}

impl Modulation {
    pub fn new(target: ModulationTarget, value: Sample) -> Self {
        Self {
            target: target,
            value: value,
        }
    }

    pub fn target(&self) -> ModulationTarget {
        self.target
    }

    pub fn value(&self) -> Sample {
        self.value
    }
}
