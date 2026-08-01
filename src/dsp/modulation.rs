use crate::dsp::types::Sample;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModulationTarget {
    Pitch,
    Volume,
    Vibrato,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Modulation {
    target: ModulationTarget,
    value: u8,
}

impl Modulation {
    pub fn new(target: ModulationTarget, value: u8) -> Self {
        Self {
            target: target,
            value: value,
        }
    }

    pub fn target(&self) -> ModulationTarget {
        self.target
    }

    pub fn value(&self) -> u8 {
        self.value
    }

    pub fn normalized(&self) -> Sample {
        self.value as Sample / 255.0
    }
}
