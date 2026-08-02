use crate::dsp::types::Sample;

#[derive(Debug, Clone, Copy)]
pub struct EnvelopeParams {
    pub attack: Sample,
    pub decay: Sample,
    pub sustain: Sample,
    pub release: Sample,
    pub filter_amount: Sample,
}

impl Default for EnvelopeParams {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.2,
            sustain: 0.7,
            release: 0.4,

            filter_amount: 0.0,
        }
    }
}
