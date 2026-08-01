use crate::dsp::types::Sample;
use crate::dsp::waveform::Waveform;

#[derive(Debug, Clone, Copy)]
pub struct OscillatorParams {
    pub waveform: Waveform,

    pub level: Sample,
}

impl Default for OscillatorParams {
    fn default() -> Self {
        Self {
            waveform: Waveform::Sine,
            level: 0.5,
        }
    }
}
