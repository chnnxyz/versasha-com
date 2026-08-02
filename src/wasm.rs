use wasm_bindgen::prelude::*;

use crate::dsp::waveform::Waveform;
use crate::engine::synth::Synth;

#[wasm_bindgen]
pub struct SynthEngine {
    synth: Synth,
}

#[wasm_bindgen]
impl SynthEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32) -> Self {
        Self {
            synth: Synth::new(sample_rate, 8),
        }
    }

    pub fn note_on(&mut self, frequency: f32) {
        self.synth.note_on(frequency);
    }

    pub fn note_off(&mut self, frequency: f32) {
        self.synth.note_off(frequency);
    }

    pub fn next_sample(&mut self) -> f32 {
        self.synth.next_sample()
    }

    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.synth.next_sample();
        }
    }

    pub fn set_master_volume(&mut self, value: f32) {
        self.synth.set_master_volume(value);
    }

    pub fn set_osc1_level(&mut self, value: f32) {
        self.synth.set_osc1_level(value);
    }

    pub fn set_osc2_level(&mut self, value: f32) {
        self.synth.set_osc2_level(value);
    }

    pub fn set_osc2_detune(&mut self, cents: f32) {
        self.synth.set_osc2_detune(cents);
    }

    pub fn set_filter_cutoff(&mut self, cutoff: f32) {
        self.synth.set_filter_cutoff(cutoff);
    }

    pub fn set_lfo_frequency(&mut self, freq: f32) {
        self.synth.set_lfo_frequency(freq);
    }

    pub fn set_lfo_amount(&mut self, amount: f32) {
        self.synth.set_lfo_amount(amount);
    }

    pub fn set_lfo_target(&mut self, target: u32) {
        let target = match target {
            0 => crate::dsp::modulation::ModulationTarget::Pitch,
            1 => crate::dsp::modulation::ModulationTarget::Vibrato,
            2 => crate::dsp::modulation::ModulationTarget::Volume,
            3 => crate::dsp::modulation::ModulationTarget::FilterCutoff,
            _ => crate::dsp::modulation::ModulationTarget::Pitch,
        };

        self.synth.set_lfo_target(target);
    }

    pub fn set_osc1_waveform(&mut self, waveform: u32) {
        self.synth.set_osc1_waveform(match waveform {
            0 => Waveform::Sine,
            1 => Waveform::Square,
            2 => Waveform::Saw,
            3 => Waveform::Triangle,
            _ => Waveform::Sine,
        });
    }

    pub fn set_osc2_waveform(&mut self, waveform: u32) {
        self.synth.set_osc2_waveform(match waveform {
            0 => Waveform::Sine,
            1 => Waveform::Square,
            2 => Waveform::Saw,
            3 => Waveform::Triangle,
            _ => Waveform::Sine,
        });
    }
}
