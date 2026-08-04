use wasm_bindgen::prelude::*;

use crate::dsp::filter::FilterType;
use crate::dsp::fx::FxRoute;
use crate::dsp::waveform::Waveform;
use crate::drums::sample_track::SampleTrackStatus;
use crate::engine::drum_machine::{DrumMachine, SequencerStatus};
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

    pub fn set_filter_resonance(&mut self, resonance: f32) {
        self.synth.set_filter_resonance(resonance);
    }

    pub fn set_filter_type(&mut self, filter_type: u32) {
        self.synth.set_filter_type(match filter_type {
            0 => FilterType::LowPass,
            1 => FilterType::HighPass,
            2 => FilterType::BandPass,
            _ => FilterType::LowPass,
        });
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

    pub fn set_delay_route(&mut self, route: u32) {
        self.synth.set_delay_route(match route {
            0 => FxRoute::Osc1,
            1 => FxRoute::Osc2,
            2 => FxRoute::Master,
            _ => FxRoute::Master,
        });
    }

    pub fn set_delay_time(&mut self, seconds: f32) {
        self.synth.set_delay_time(seconds);
    }

    pub fn set_delay_feedback(&mut self, feedback: f32) {
        self.synth.set_delay_feedback(feedback);
    }

    pub fn set_delay_mix(&mut self, mix: f32) {
        self.synth.set_delay_mix(mix);
    }
}

#[wasm_bindgen]
pub struct DrumMachineEngine {
    machine: DrumMachine,
}

#[wasm_bindgen]
impl DrumMachineEngine {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sample_rate: f32,
        kick: Vec<f32>,
        snare: Vec<f32>,
        clap: Vec<f32>,
        rimshot: Vec<f32>,
        tom_low: Vec<f32>,
        tom_mid: Vec<f32>,
        tom_hi: Vec<f32>,
        hihat_closed: Vec<f32>,
        hihat_open: Vec<f32>,
        crash: Vec<f32>,
        ride: Vec<f32>,
    ) -> Self {
        Self {
            machine: DrumMachine::new(
                sample_rate,
                kick,
                snare,
                clap,
                rimshot,
                tom_low,
                tom_mid,
                tom_hi,
                hihat_closed,
                hihat_open,
                crash,
                ride,
            ),
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.machine.next_sample()
    }

    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.machine.next_sample();
        }
    }

    pub fn play(&mut self) {
        self.machine.play();
    }

    pub fn pause(&mut self) {
        self.machine.pause();
    }

    pub fn stop(&mut self) {
        self.machine.stop();
    }

    pub fn sequencer_status(&self) -> u32 {
        match self.machine.sequencer_status() {
            SequencerStatus::Play => 0,
            SequencerStatus::Pause => 1,
            SequencerStatus::Stop => 2,
        }
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.machine.set_bpm(bpm);
    }

    pub fn bpm(&self) -> f32 {
        self.machine.bpm()
    }

    pub fn current_step(&self) -> usize {
        self.machine.current_step()
    }

    pub fn track_count(&self) -> usize {
        self.machine.track_count()
    }

    // encoded as 0/1 per track rather than returning Vec<bool> directly,
    // to keep the boundary type simple and unambiguous on the JS side
    pub fn active_step_tracks(&self) -> Vec<u8> {
        self.machine
            .active_step_tracks()
            .iter()
            .map(|&active| active as u8)
            .collect()
    }

    pub fn set_step(&mut self, step: usize, track: usize, active: bool) {
        self.machine.set_step(step, track, active);
    }

    pub fn clear_track_pattern(&mut self, track: usize) {
        self.machine.clear_track_pattern(track);
    }

    pub fn clear_all_patterns(&mut self) {
        self.machine.clear_all_patterns();
    }

    pub fn trigger_track(&mut self, index: usize) {
        self.machine.trigger_track(index);
    }

    pub fn set_active_track(&mut self, index: Option<usize>) {
        self.machine.set_active_track(index);
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.machine.set_master_volume(volume);
    }

    pub fn master_volume(&self) -> f32 {
        self.machine.master_volume()
    }

    pub fn set_track_volume(&mut self, index: usize, volume: f32) {
        self.machine.set_track_volume(index, volume);
    }

    pub fn track_volume(&self, index: usize) -> f32 {
        self.machine.track_volume(index).unwrap_or(0.0)
    }

    pub fn set_track_status(&mut self, index: usize, status: u32) {
        let status = match status {
            0 => SampleTrackStatus::Active,
            1 => SampleTrackStatus::Solo,
            2 => SampleTrackStatus::Muted,
            _ => SampleTrackStatus::Active,
        };

        self.machine.set_track_status(index, status);
    }

    pub fn track_status(&self, index: usize) -> u32 {
        match self.machine.track_status(index) {
            Some(SampleTrackStatus::Active) | None => 0,
            Some(SampleTrackStatus::Solo) => 1,
            Some(SampleTrackStatus::Muted) => 2,
        }
    }

    // 0.0 on a bad index or an unsupported param, same fallback
    // convention as track_volume above -- the UI only ever renders these
    // knobs for track types that actually support them, so 0.0 is never
    // seen as a "real" value by anything that isn't misusing the API
    pub fn set_track_tune(&mut self, index: usize, semitones: f32) {
        self.machine.set_track_tune(index, semitones);
    }

    pub fn track_tune(&self, index: usize) -> f32 {
        self.machine.track_tune(index).unwrap_or(0.0)
    }

    pub fn set_track_attack(&mut self, index: usize, time: f32) {
        self.machine.set_track_attack(index, time);
    }

    pub fn track_attack(&self, index: usize) -> f32 {
        self.machine.track_attack(index).unwrap_or(0.0)
    }

    pub fn set_track_decay(&mut self, index: usize, time: f32) {
        self.machine.set_track_decay(index, time);
    }

    pub fn track_decay(&self, index: usize) -> f32 {
        self.machine.track_decay(index).unwrap_or(0.0)
    }

    pub fn set_track_snappy(&mut self, index: usize, amount: f32) {
        self.machine.set_track_snappy(index, amount);
    }

    pub fn track_snappy(&self, index: usize) -> f32 {
        self.machine.track_snappy(index).unwrap_or(0.0)
    }
}
