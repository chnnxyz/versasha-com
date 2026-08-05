use crate::dsp::types::Sample;

pub mod lfo;
pub mod modulation_matrix;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModulationTarget {
    Pitch,
    Vibrato,
    Volume,
    FilterCutoff,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModulationSource {
    Lfo,
    Envelope,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Modulation {
    source: ModulationSource,
    target: ModulationTarget,
    value: Sample,
}

impl Modulation {
    pub fn new(source: ModulationSource, target: ModulationTarget, value: Sample) -> Self {
        Self {
            source,
            target,
            value,
        }
    }

    pub fn source(&self) -> ModulationSource {
        self.source
    }

    pub fn target(&self) -> ModulationTarget {
        self.target
    }

    pub fn value(&self) -> Sample {
        self.value
    }
}

pub trait ModulationGenerator {
    fn next_modulation(&mut self) -> Modulation;
    fn reset(&mut self);
}
