use crate::dsp::types::Frequency;

#[derive(Debug, Clone, Copy)]
pub struct FilterParams {
    pub cutoff: Frequency,

    pub resonance: f32,
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            cutoff: 2000.0,

            resonance: 0.0,
        }
    }
}
