use super::oscillator_params::OscillatorParams;

#[derive(Debug, Clone, Copy)]
pub struct VoiceParams {
    pub osc1: OscillatorParams,

    pub osc2: OscillatorParams,

    // cents
    pub osc2_detune: f32,
}

impl Default for VoiceParams {
    fn default() -> Self {
        Self {
            osc1: OscillatorParams::default(),

            osc2: OscillatorParams::default(),

            osc2_detune: 0.0,
        }
    }
}
