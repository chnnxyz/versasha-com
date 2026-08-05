use wasm_bindgen::prelude::*;

use crate::acid_bass::acid_step::AcidStep;
use crate::dsp::filters::FilterType;
use crate::dsp::fx::FxRoute;
use crate::dsp::oscillators::waveform::Waveform;
use crate::drums::sample_track::SampleTrackStatus;
use crate::engine::acid_synth::AcidSynth;
use crate::engine::drum_machine::DrumMachine;
use crate::engine::session::{Session, ACID_CHANNEL, DRUM_CHANNEL, SYNTH_CHANNEL};
use crate::engine::synth::Synth;
use crate::mixer::instrument_channel::InstrumentChannelStatus;
use crate::sequencing::transport::SequencerStatus;

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

#[wasm_bindgen]
pub struct AcidSynthEngine {
    synth: AcidSynth,
}

#[wasm_bindgen]
impl AcidSynthEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32, num_steps: usize, bpm: f32, steps_per_beat: Option<usize>) -> Self {
        Self {
            synth: AcidSynth::new(sample_rate, num_steps, bpm, steps_per_beat),
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.synth.next_sample()
    }

    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.synth.next_sample();
        }
    }

    pub fn play(&mut self) {
        self.synth.play();
    }

    pub fn pause(&mut self) {
        self.synth.pause();
    }

    pub fn stop(&mut self) {
        self.synth.stop();
    }

    pub fn sequencer_status(&self) -> u32 {
        match self.synth.sequencer_status() {
            SequencerStatus::Play => 0,
            SequencerStatus::Pause => 1,
            SequencerStatus::Stop => 2,
        }
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.synth.set_bpm(bpm);
    }

    pub fn bpm(&self) -> f32 {
        self.synth.bpm()
    }

    pub fn current_step(&self) -> usize {
        self.synth.current_step()
    }

    pub fn step_count(&self) -> usize {
        self.synth.step_count()
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.synth.set_master_volume(volume);
    }

    pub fn master_volume(&self) -> f32 {
        self.synth.master_volume()
    }

    // primitives in, since AcidStep itself isn't exposed across the wasm
    // boundary -- build one internally and hand it to AcidSynth, same
    // "no exposed Rust structs across the boundary" convention the rest
    // of this file follows
    pub fn set_step(&mut self, index: usize, note: f32, gate: bool, accent: bool, slide: bool) {
        let mut step = AcidStep::new();
        step.set_note(note);
        step.set_gate(gate);
        step.set_accent(accent);
        step.set_slide(slide);

        self.synth.set_step(index, step);
    }

    pub fn clear_all_steps(&mut self) {
        self.synth.clear_all_steps();
    }

    pub fn set_waveform(&mut self, waveform: u32) {
        self.synth.set_waveform(match waveform {
            0 => Waveform::Sine,
            1 => Waveform::Square,
            2 => Waveform::Saw,
            3 => Waveform::Triangle,
            _ => Waveform::Sine,
        });
    }

    pub fn set_tuning(&mut self, semitones: f32) {
        self.synth.set_tuning(semitones);
    }

    pub fn set_cutoff(&mut self, cutoff: f32) {
        self.synth.set_cutoff(cutoff);
    }

    pub fn set_resonance(&mut self, resonance: f32) {
        self.synth.set_resonance(resonance);
    }

    pub fn set_env_mod(&mut self, amount: f32) {
        self.synth.set_env_mod(amount);
    }

    pub fn set_decay(&mut self, time: f32) {
        self.synth.set_decay(time);
    }

    pub fn set_accent_amount(&mut self, amount: f32) {
        self.synth.set_accent_amount(amount);
    }

    pub fn set_glide_time(&mut self, time: f32) {
        self.synth.set_glide_time(time);
    }
}

// mixer channel indices -- exported instead of hardcoded on the JS side,
// so the wasm binding stays the one source of truth for channel order
// (matches Session's own DRUM_CHANNEL/ACID_CHANNEL/SYNTH_CHANNEL)
#[wasm_bindgen]
pub fn drum_channel() -> usize {
    DRUM_CHANNEL
}

#[wasm_bindgen]
pub fn acid_channel() -> usize {
    ACID_CHANNEL
}

#[wasm_bindgen]
pub fn synth_channel() -> usize {
    SYNTH_CHANNEL
}

// The unified engine for the merged synth+drums+bass+mixer page --
// wraps Session, replacing SynthEngine/DrumMachineEngine/AcidSynthEngine
// above (each still kept around for their own standalone pages). Reaches
// instrument-specific knobs through Session's drum_machine_mut()/
// acid_synth_mut()/synth_mut() accessors rather than Session
// re-exposing every one of them itself.
#[wasm_bindgen]
pub struct SessionEngine {
    session: Session,
}

#[wasm_bindgen]
impl SessionEngine {
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sample_rate: f32,
        num_steps: usize,
        bpm: f32,
        steps_per_beat: Option<usize>,
        voice_count: usize,
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
            session: Session::new(
                sample_rate,
                num_steps,
                bpm,
                steps_per_beat,
                voice_count,
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

    // --- output ------------------------------------------------------

    // fills two separate buffers (rather than one interleaved one or a
    // returned tuple) so the worklet can hand each straight to its own
    // Web Audio output channel with no reshaping on the JS side
    pub fn fill_buffer(&mut self, left: &mut [f32], right: &mut [f32]) {
        let len = left.len().min(right.len());

        for i in 0..len {
            let (l, r) = self.session.next_sample();
            left[i] = l;
            right[i] = r;
        }
    }

    // --- transport (shared by drums + bass) ---------------------------

    pub fn play(&mut self) {
        self.session.play();
    }

    pub fn pause(&mut self) {
        self.session.pause();
    }

    pub fn stop(&mut self) {
        self.session.stop();
    }

    pub fn sequencer_status(&self) -> u32 {
        match self.session.sequencer_status() {
            SequencerStatus::Play => 0,
            SequencerStatus::Pause => 1,
            SequencerStatus::Stop => 2,
        }
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.session.set_bpm(bpm);
    }

    pub fn bpm(&self) -> f32 {
        self.session.bpm()
    }

    pub fn current_step(&self) -> usize {
        self.session.current_step()
    }

    // --- synth (live, always playing regardless of transport) --------

    pub fn note_on(&mut self, frequency: f32) {
        self.session.note_on(frequency);
    }

    pub fn note_off(&mut self, frequency: f32) {
        self.session.note_off(frequency);
    }

    pub fn set_synth_master_volume(&mut self, value: f32) {
        self.session.synth_mut().set_master_volume(value);
    }

    pub fn set_osc1_level(&mut self, value: f32) {
        self.session.synth_mut().set_osc1_level(value);
    }

    pub fn set_osc2_level(&mut self, value: f32) {
        self.session.synth_mut().set_osc2_level(value);
    }

    pub fn set_osc2_detune(&mut self, cents: f32) {
        self.session.synth_mut().set_osc2_detune(cents);
    }

    pub fn set_filter_cutoff(&mut self, cutoff: f32) {
        self.session.synth_mut().set_filter_cutoff(cutoff);
    }

    pub fn set_filter_resonance(&mut self, resonance: f32) {
        self.session.synth_mut().set_filter_resonance(resonance);
    }

    pub fn set_filter_type(&mut self, filter_type: u32) {
        self.session.synth_mut().set_filter_type(match filter_type {
            0 => FilterType::LowPass,
            1 => FilterType::HighPass,
            2 => FilterType::BandPass,
            _ => FilterType::LowPass,
        });
    }

    pub fn set_lfo_frequency(&mut self, freq: f32) {
        self.session.synth_mut().set_lfo_frequency(freq);
    }

    pub fn set_lfo_amount(&mut self, amount: f32) {
        self.session.synth_mut().set_lfo_amount(amount);
    }

    pub fn set_lfo_target(&mut self, target: u32) {
        let target = match target {
            0 => crate::dsp::modulation::ModulationTarget::Pitch,
            1 => crate::dsp::modulation::ModulationTarget::Vibrato,
            2 => crate::dsp::modulation::ModulationTarget::Volume,
            3 => crate::dsp::modulation::ModulationTarget::FilterCutoff,
            _ => crate::dsp::modulation::ModulationTarget::Pitch,
        };

        self.session.synth_mut().set_lfo_target(target);
    }

    pub fn set_osc1_waveform(&mut self, waveform: u32) {
        self.session.synth_mut().set_osc1_waveform(match waveform {
            0 => Waveform::Sine,
            1 => Waveform::Square,
            2 => Waveform::Saw,
            3 => Waveform::Triangle,
            _ => Waveform::Sine,
        });
    }

    pub fn set_osc2_waveform(&mut self, waveform: u32) {
        self.session.synth_mut().set_osc2_waveform(match waveform {
            0 => Waveform::Sine,
            1 => Waveform::Square,
            2 => Waveform::Saw,
            3 => Waveform::Triangle,
            _ => Waveform::Sine,
        });
    }

    pub fn set_delay_route(&mut self, route: u32) {
        self.session.synth_mut().set_delay_route(match route {
            0 => FxRoute::Osc1,
            1 => FxRoute::Osc2,
            2 => FxRoute::Master,
            _ => FxRoute::Master,
        });
    }

    pub fn set_delay_time(&mut self, seconds: f32) {
        self.session.synth_mut().set_delay_time(seconds);
    }

    pub fn set_delay_feedback(&mut self, feedback: f32) {
        self.session.synth_mut().set_delay_feedback(feedback);
    }

    pub fn set_delay_mix(&mut self, mix: f32) {
        self.session.synth_mut().set_delay_mix(mix);
    }

    // --- drums ---------------------------------------------------------

    pub fn track_count(&self) -> usize {
        self.session.drum_machine().track_count()
    }

    pub fn set_drum_step(&mut self, step: usize, track: usize, active: bool) {
        self.session.set_drum_step(step, track, active);
    }

    pub fn trigger_drum_track(&mut self, index: usize) {
        self.session.trigger_drum_track(index);
    }

    pub fn clear_drum_track_pattern(&mut self, track: usize) {
        self.session.drum_machine_mut().clear_track_pattern(track);
    }

    pub fn clear_all_drum_patterns(&mut self) {
        self.session.drum_machine_mut().clear_all_patterns();
    }

    pub fn set_drum_master_volume(&mut self, volume: f32) {
        self.session.drum_machine_mut().set_master_volume(volume);
    }

    pub fn drum_master_volume(&mut self) -> f32 {
        self.session.drum_machine().master_volume()
    }

    pub fn set_drum_track_volume(&mut self, index: usize, volume: f32) {
        self.session.drum_machine_mut().set_track_volume(index, volume);
    }

    pub fn drum_track_volume(&mut self, index: usize) -> f32 {
        self.session.drum_machine().track_volume(index).unwrap_or(0.0)
    }

    pub fn set_drum_track_status(&mut self, index: usize, status: u32) {
        let status = match status {
            0 => SampleTrackStatus::Active,
            1 => SampleTrackStatus::Solo,
            2 => SampleTrackStatus::Muted,
            _ => SampleTrackStatus::Active,
        };

        self.session.drum_machine_mut().set_track_status(index, status);
    }

    pub fn drum_track_status(&mut self, index: usize) -> u32 {
        match self.session.drum_machine().track_status(index) {
            Some(SampleTrackStatus::Active) | None => 0,
            Some(SampleTrackStatus::Solo) => 1,
            Some(SampleTrackStatus::Muted) => 2,
        }
    }

    pub fn set_drum_track_tune(&mut self, index: usize, semitones: f32) {
        self.session.drum_machine_mut().set_track_tune(index, semitones);
    }

    pub fn drum_track_tune(&mut self, index: usize) -> f32 {
        self.session.drum_machine().track_tune(index).unwrap_or(0.0)
    }

    pub fn set_drum_track_attack(&mut self, index: usize, time: f32) {
        self.session.drum_machine_mut().set_track_attack(index, time);
    }

    pub fn drum_track_attack(&mut self, index: usize) -> f32 {
        self.session.drum_machine().track_attack(index).unwrap_or(0.0)
    }

    pub fn set_drum_track_decay(&mut self, index: usize, time: f32) {
        self.session.drum_machine_mut().set_track_decay(index, time);
    }

    pub fn drum_track_decay(&mut self, index: usize) -> f32 {
        self.session.drum_machine().track_decay(index).unwrap_or(0.0)
    }

    pub fn set_drum_track_snappy(&mut self, index: usize, amount: f32) {
        self.session.drum_machine_mut().set_track_snappy(index, amount);
    }

    pub fn drum_track_snappy(&mut self, index: usize) -> f32 {
        self.session.drum_machine().track_snappy(index).unwrap_or(0.0)
    }

    // --- bass (acid) -----------------------------------------------------

    // primitives in, since AcidStep itself isn't exposed across the wasm
    // boundary -- same convention AcidSynthEngine::set_step above follows
    pub fn set_acid_step(&mut self, index: usize, note: f32, gate: bool, accent: bool, slide: bool) {
        let mut step = AcidStep::new();
        step.set_note(note);
        step.set_gate(gate);
        step.set_accent(accent);
        step.set_slide(slide);

        self.session.set_acid_step(index, step);
    }

    pub fn clear_all_acid_steps(&mut self) {
        self.session.acid_synth_mut().clear_all_steps();
    }

    pub fn acid_step_count(&mut self) -> usize {
        self.session.acid_synth_mut().step_count()
    }

    pub fn set_acid_waveform(&mut self, waveform: u32) {
        self.session.acid_synth_mut().set_waveform(match waveform {
            0 => Waveform::Sine,
            1 => Waveform::Square,
            2 => Waveform::Saw,
            3 => Waveform::Triangle,
            _ => Waveform::Sine,
        });
    }

    pub fn set_acid_tuning(&mut self, semitones: f32) {
        self.session.acid_synth_mut().set_tuning(semitones);
    }

    pub fn set_acid_cutoff(&mut self, cutoff: f32) {
        self.session.acid_synth_mut().set_cutoff(cutoff);
    }

    pub fn set_acid_resonance(&mut self, resonance: f32) {
        self.session.acid_synth_mut().set_resonance(resonance);
    }

    pub fn set_acid_env_mod(&mut self, amount: f32) {
        self.session.acid_synth_mut().set_env_mod(amount);
    }

    pub fn set_acid_decay(&mut self, time: f32) {
        self.session.acid_synth_mut().set_decay(time);
    }

    pub fn set_acid_accent_amount(&mut self, amount: f32) {
        self.session.acid_synth_mut().set_accent_amount(amount);
    }

    pub fn set_acid_glide_time(&mut self, time: f32) {
        self.session.acid_synth_mut().set_glide_time(time);
    }

    pub fn set_acid_master_volume(&mut self, volume: f32) {
        self.session.acid_synth_mut().set_master_volume(volume);
    }

    pub fn acid_master_volume(&mut self) -> f32 {
        self.session.acid_synth().master_volume()
    }

    // --- mixer -----------------------------------------------------------
    //
    // `channel` is one of drum_channel()/acid_channel()/synth_channel()
    // above. Every setter here is a silent no-op on an out-of-range
    // channel, every getter falls back to a default -- same "bad index
    // from outside can't panic the audio thread" convention as the
    // per-track drum accessors above.

    pub fn set_channel_status(&mut self, channel: usize, status: u32) {
        let status = match status {
            0 => InstrumentChannelStatus::Active,
            1 => InstrumentChannelStatus::Solo,
            2 => InstrumentChannelStatus::Muted,
            _ => InstrumentChannelStatus::Active,
        };

        if let Some(ch) = self.session.mixer_channel_mut(channel) {
            ch.set_status(status);
        }
    }

    pub fn channel_status(&mut self, channel: usize) -> u32 {
        match self.session.mixer_channel_mut(channel).map(|ch| ch.status()) {
            Some(InstrumentChannelStatus::Active) | None => 0,
            Some(InstrumentChannelStatus::Solo) => 1,
            Some(InstrumentChannelStatus::Muted) => 2,
        }
    }

    pub fn set_channel_volume(&mut self, channel: usize, volume: f32) {
        if let Some(ch) = self.session.mixer_channel_mut(channel) {
            ch.set_volume(volume);
        }
    }

    pub fn channel_volume(&mut self, channel: usize) -> f32 {
        self.session
            .mixer_channel_mut(channel)
            .map(|ch| ch.volume())
            .unwrap_or(0.0)
    }

    pub fn set_channel_pan(&mut self, channel: usize, pan: f32) {
        if let Some(ch) = self.session.mixer_channel_mut(channel) {
            ch.set_pan(pan);
        }
    }

    pub fn channel_pan(&mut self, channel: usize) -> f32 {
        self.session
            .mixer_channel_mut(channel)
            .map(|ch| ch.pan())
            .unwrap_or(0.0)
    }

    pub fn set_channel_eq_low_gain(&mut self, channel: usize, gain: f32) {
        if let Some(ch) = self.session.mixer_channel_mut(channel) {
            ch.eq_mut().set_low_gain(gain);
        }
    }

    pub fn channel_eq_low_gain(&mut self, channel: usize) -> f32 {
        self.session
            .mixer_channel_mut(channel)
            .map(|ch| ch.eq_mut().low_gain())
            .unwrap_or(1.0)
    }

    pub fn set_channel_eq_mid_gain(&mut self, channel: usize, gain: f32) {
        if let Some(ch) = self.session.mixer_channel_mut(channel) {
            ch.eq_mut().set_mid_gain(gain);
        }
    }

    pub fn channel_eq_mid_gain(&mut self, channel: usize) -> f32 {
        self.session
            .mixer_channel_mut(channel)
            .map(|ch| ch.eq_mut().mid_gain())
            .unwrap_or(1.0)
    }

    pub fn set_channel_eq_high_gain(&mut self, channel: usize, gain: f32) {
        if let Some(ch) = self.session.mixer_channel_mut(channel) {
            ch.eq_mut().set_high_gain(gain);
        }
    }

    pub fn channel_eq_high_gain(&mut self, channel: usize) -> f32 {
        self.session
            .mixer_channel_mut(channel)
            .map(|ch| ch.eq_mut().high_gain())
            .unwrap_or(1.0)
    }

    // per-channel VU meter level -- see InstrumentChannel::peak() for the
    // ballistics (instant rise, exponential decay)
    pub fn channel_peak(&mut self, channel: usize) -> f32 {
        self.session
            .mixer_channel_mut(channel)
            .map(|ch| ch.peak())
            .unwrap_or(0.0)
    }

    // --- mixer master section --------------------------------------------

    pub fn set_mixer_master_volume(&mut self, volume: f32) {
        self.session.mixer_mut().set_master_volume(volume);
    }

    pub fn mixer_master_volume(&self) -> f32 {
        self.session.mixer().master_volume()
    }

    pub fn master_peak_left(&self) -> f32 {
        self.session.mixer().master_peak_left()
    }

    pub fn master_peak_right(&self) -> f32 {
        self.session.mixer().master_peak_right()
    }
}
