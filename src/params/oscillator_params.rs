use crate::dsp::oscillators::waveform::Waveform;
use crate::dsp::types::Sample;

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
