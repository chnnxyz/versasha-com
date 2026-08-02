use crate::dsp::lfo::Lfo;
use crate::dsp::modulation::{ModulationGenerator, ModulationTarget};
use crate::dsp::types::{Sample, SampleRate};
use crate::dsp::voice::Voice;
use crate::dsp::waveform::Waveform;

pub struct Synth {
    voices: Vec<Voice>,
    lfos: Vec<Lfo>,
    sample_rate: SampleRate,
    master_volume: Sample,
}

impl Synth {
    pub fn new(rate: SampleRate, voices: usize) -> Self {
        let mut synth = Self {
            voices: Vec::new(),
            lfos: vec![Lfo::new(rate)],
            sample_rate: rate,
            master_volume: 1.0,
        };

        for _ in 0..voices {
            synth.voices.push(Voice::new(rate));
        }

        synth
    }

    pub fn note_on(&mut self, frequency: Sample) {
        let index = self
            .voices
            .iter()
            .position(|voice| !voice.is_active())
            .unwrap_or(0);

        self.voices[index].note_on(frequency);
    }

    pub fn note_off(&mut self, frequency: Sample) {
        if let Some(voice) = self
            .voices
            .iter_mut()
            .find(|voice| voice.is_active() && voice.frequency() == frequency)
        {
            voice.note_off();
        }
    }

    pub fn lfo_mut(&mut self, index: usize) -> Option<&mut Lfo> {
        self.lfos.get_mut(index)
    }

    pub fn add_lfo(&mut self, lfo: Lfo) {
        self.lfos.push(lfo);
    }

    pub fn clear_lfos(&mut self) {
        self.lfos.clear();

        for voice in self.voices.iter_mut() {
            voice.reset_modulation();
        }
    }

    pub fn reset_lfos(&mut self) {
        for lfo in self.lfos.iter_mut() {
            lfo.reset();
        }

        for voice in self.voices.iter_mut() {
            voice.reset_modulation();
        }
    }

    pub fn set_master_volume(&mut self, volume: Sample) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    pub fn voice_count(&self) -> usize {
        self.voices.len()
    }

    pub fn lfo_count(&self) -> usize {
        self.lfos.len()
    }

    pub fn next_sample(&mut self) -> Sample {
        let modulations = self
            .lfos
            .iter_mut()
            .map(|lfo| lfo.next_modulation())
            .collect::<Vec<_>>();

        let mut output = 0.0;

        for voice in self.voices.iter_mut() {
            let matrix = voice.modulation_matrix_mut();

            matrix.clear();

            for modulation in modulations.iter().copied() {
                matrix.push(modulation);
            }

            output += voice.next_sample();
        }

        (output * self.master_volume).clamp(-1.0, 1.0)
    }

    pub fn set_osc1_level(&mut self, level: Sample) {
        let level = level.clamp(0.0, 1.0);

        for voice in self.voices.iter_mut() {
            let mut params = voice.params();

            params.osc1.level = level;

            voice.set_params(params);
        }
    }

    pub fn set_osc2_level(&mut self, level: Sample) {
        let level = level.clamp(0.0, 1.0);

        for voice in self.voices.iter_mut() {
            let mut params = voice.params();

            params.osc2.level = level;

            voice.set_params(params);
        }
    }

    pub fn set_osc2_detune(&mut self, cents: Sample) {
        let cents = cents.clamp(-1200.0, 1200.0);

        for voice in self.voices.iter_mut() {
            let mut params = voice.params();

            params.osc2_detune = cents;

            voice.set_params(params);
        }
    }

    pub fn set_filter_cutoff(&mut self, cutoff: Sample) {
        let maximum = self.sample_rate * 0.45;

        let cutoff = cutoff.clamp(20.0, maximum);

        for voice in self.voices.iter_mut() {
            voice.set_filter_cutoff(cutoff);
        }
    }

    pub fn set_osc1_waveform(&mut self, waveform: Waveform) {
        for voice in self.voices.iter_mut() {
            let mut params = voice.params();

            params.osc1.waveform = waveform;

            voice.set_params(params);
        }
    }

    pub fn set_osc2_waveform(&mut self, waveform: Waveform) {
        for voice in self.voices.iter_mut() {
            let mut params = voice.params();

            params.osc2.waveform = waveform;

            voice.set_params(params);
        }
    }

    pub fn set_lfo_frequency(&mut self, frequency: Sample) {
        if let Some(lfo) = self.lfos.first_mut() {
            lfo.set_freq(frequency.clamp(0.01, 20.0));
        }
    }

    pub fn set_lfo_amount(&mut self, amount: Sample) {
        if let Some(lfo) = self.lfos.first_mut() {
            lfo.set_amount(amount.max(0.0));
        }
    }

    pub fn set_lfo_target(&mut self, target: ModulationTarget) {
        if let Some(lfo) = self.lfos.first_mut() {
            lfo.set_target(target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synth_creates_requested_voice_count() {
        let synth = Synth::new(48_000.0, 8);

        assert_eq!(synth.voice_count(), 8);
    }

    #[test]
    fn synth_starts_with_one_lfo() {
        let synth = Synth::new(48_000.0, 4);

        assert_eq!(synth.lfo_count(), 1);
    }

    #[test]
    fn lfo_can_be_added() {
        let mut synth = Synth::new(48_000.0, 4);

        synth.add_lfo(Lfo::new(48_000.0));

        assert_eq!(synth.lfo_count(), 2);
    }

    #[test]
    fn note_on_activates_a_voice() {
        let mut synth = Synth::new(48_000.0, 4);

        synth.note_on(440.0);

        assert!(synth.voices.iter().any(Voice::is_active));
    }

    #[test]
    fn note_off_releases_matching_voice() {
        let mut synth = Synth::new(48_000.0, 4);

        synth.note_on(440.0);
        synth.note_on(660.0);

        synth.note_off(440.0);

        let voice_440 = synth
            .voices
            .iter()
            .find(|voice| voice.frequency() == 440.0)
            .expect("missing 440 Hz voice");

        assert_eq!(
            voice_440.envelope_state(),
            crate::dsp::envelope::EnvelopeState::Release
        );
    }

    #[test]
    fn filter_cutoff_is_limited_by_sample_rate() {
        let mut synth = Synth::new(48_000.0, 1);

        synth.set_filter_cutoff(100_000.0);

        assert_eq!(synth.voices[0].filter_cutoff(), 21_600.0);
    }

    #[test]
    fn filter_cutoff_has_minimum() {
        let mut synth = Synth::new(48_000.0, 1);

        synth.set_filter_cutoff(-100.0);

        assert_eq!(synth.voices[0].filter_cutoff(), 20.0);
    }

    #[test]
    fn master_volume_is_clamped() {
        let mut synth = Synth::new(48_000.0, 1);

        synth.set_master_volume(2.0);

        assert_eq!(synth.master_volume, 1.0);
    }

    #[test]
    fn output_stays_finite() {
        let mut synth = Synth::new(48_000.0, 4);

        synth.note_on(440.0);

        for _ in 0..100_000 {
            assert!(synth.next_sample().is_finite());
        }
    }

    #[test]
    fn output_stays_in_range() {
        let mut synth = Synth::new(48_000.0, 8);

        synth.note_on(261.63);
        synth.note_on(329.63);
        synth.note_on(392.0);

        for _ in 0..100_000 {
            let sample = synth.next_sample();

            assert!((-1.0..=1.0).contains(&sample), "out of range: {}", sample);
        }
    }

    #[test]
    fn reset_lfos_keeps_sources() {
        let mut synth = Synth::new(48_000.0, 1);

        synth.add_lfo(Lfo::new(48_000.0));

        synth.reset_lfos();

        assert_eq!(synth.lfo_count(), 2);
    }

    #[test]
    fn clear_lfos_removes_all_lfos() {
        let mut synth = Synth::new(48_000.0, 1);

        synth.add_lfo(Lfo::new(48_000.0));

        synth.clear_lfos();

        assert_eq!(synth.lfo_count(), 0);
    }
}
