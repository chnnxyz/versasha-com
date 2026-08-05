import "./text-decoder-polyfill.js";

import {
    initSync,
    SessionEngine
} from "../pkg/versasha_com.js";

// mirrors Session::DRUM_CHANNEL/ACID_CHANNEL/SYNTH_CHANNEL (also
// available from wasm as drum_channel()/acid_channel()/synth_channel())
const MIXER_CHANNEL_INDICES = [0, 1, 2];

// VU meter refresh rate -- every block (128 samples @ 44.1kHz, ~2.9ms)
// would be needlessly chatty for something a human is just looking at;
// every 6th block is close to 60fps and plenty smooth
const LEVEL_REPORT_INTERVAL_BLOCKS = 6;

// The unified processor for the merged synth+drums+bass+mixer page --
// replaces synth-worklet.js/drum-worklet.js/acid-worklet.js's three
// separate processors with one, wrapping SessionEngine so drums and
// bass share a single Transport while the synth stays always-live on
// top, matching Session's own design.
class SessionProcessor extends AudioWorkletProcessor {
    constructor(options) {
        super();

        const {
            wasmModule,
            samples,
            numSteps,
            bpm,
            stepsPerBeat,
            voiceCount
        } = options.processorOptions ?? {};

        if (!wasmModule || !samples) {
            throw new Error(
                "Missing wasmModule or samples in processorOptions"
            );
        }

        initSync({
            module: wasmModule
        });

        this.engine = new SessionEngine(
            sampleRate,
            numSteps,
            bpm,
            stepsPerBeat ?? null,
            voiceCount,
            samples.kick,
            samples.snare,
            samples.clap,
            samples.rimshot,
            samples.tomLow,
            samples.tomMid,
            samples.tomHi,
            samples.hihatClosed,
            samples.hihatOpen,
            samples.crash,
            samples.ride
        );

        this.lastReportedStep = -1;
        this.lastReportedStatus = -1;
        this.levelReportCounter = 0;

        this.port.onmessage = event => {
            const message = event.data;

            try {
                this.handleMessage(message);
            } catch (error) {
                this.port.postMessage({
                    type: "error",
                    message: String(error)
                });
            }
        };
    }

    handleMessage(message) {
        const engine = this.engine;

        switch (message.type) {
            // --- transport -----------------------------------------------
            case "play":
                engine.play();
                break;

            case "pause":
                engine.pause();
                break;

            case "stop":
                engine.stop();
                break;

            case "set-bpm":
                engine.set_bpm(Number(message.value));
                break;

            // --- synth (live) ----------------------------------------------
            case "note-on":
                engine.note_on(Number(message.frequency));
                break;

            case "note-off":
                engine.note_off(Number(message.frequency));
                break;

            case "synth-master-volume":
                engine.set_synth_master_volume(Number(message.value));
                break;

            case "osc1-level":
                engine.set_osc1_level(Number(message.value));
                break;

            case "osc2-level":
                engine.set_osc2_level(Number(message.value));
                break;

            case "osc2-detune":
                engine.set_osc2_detune(Number(message.value));
                break;

            case "osc1-waveform":
                engine.set_osc1_waveform(Number(message.value));
                break;

            case "osc2-waveform":
                engine.set_osc2_waveform(Number(message.value));
                break;

            case "filter-cutoff":
                engine.set_filter_cutoff(Number(message.value));
                break;

            case "filter-resonance":
                engine.set_filter_resonance(Number(message.value));
                break;

            case "filter-type":
                engine.set_filter_type(Number(message.value));
                break;

            case "lfo-frequency":
                engine.set_lfo_frequency(Number(message.value));
                break;

            case "lfo-amount":
                engine.set_lfo_amount(Number(message.value));
                break;

            case "lfo-target":
                engine.set_lfo_target(Number(message.value));
                break;

            case "delay-route":
                engine.set_delay_route(Number(message.value));
                break;

            case "delay-time":
                engine.set_delay_time(Number(message.value));
                break;

            case "delay-feedback":
                engine.set_delay_feedback(Number(message.value));
                break;

            case "delay-mix":
                engine.set_delay_mix(Number(message.value));
                break;

            // --- drums -----------------------------------------------------
            case "set-drum-step":
                engine.set_drum_step(
                    Number(message.step),
                    Number(message.track),
                    Boolean(message.active)
                );
                break;

            case "trigger-drum-track":
                engine.trigger_drum_track(Number(message.track));
                break;

            case "clear-drum-track-pattern":
                engine.clear_drum_track_pattern(Number(message.track));
                break;

            case "clear-all-drum-patterns":
                engine.clear_all_drum_patterns();
                break;

            case "set-drum-master-volume":
                engine.set_drum_master_volume(Number(message.value));
                break;

            case "set-drum-track-volume":
                engine.set_drum_track_volume(Number(message.track), Number(message.value));
                break;

            case "set-drum-track-status":
                engine.set_drum_track_status(Number(message.track), Number(message.value));
                break;

            case "set-drum-track-tune":
                engine.set_drum_track_tune(Number(message.track), Number(message.value));
                break;

            case "set-drum-track-attack":
                engine.set_drum_track_attack(Number(message.track), Number(message.value));
                break;

            case "set-drum-track-decay":
                engine.set_drum_track_decay(Number(message.track), Number(message.value));
                break;

            case "set-drum-track-snappy":
                engine.set_drum_track_snappy(Number(message.track), Number(message.value));
                break;

            // --- bass (acid) -------------------------------------------------
            case "set-acid-step":
                engine.set_acid_step(
                    Number(message.index),
                    Number(message.note),
                    Boolean(message.gate),
                    Boolean(message.accent),
                    Boolean(message.slide)
                );
                break;

            case "clear-all-acid-steps":
                engine.clear_all_acid_steps();
                break;

            case "set-acid-waveform":
                engine.set_acid_waveform(Number(message.value));
                break;

            case "set-acid-tuning":
                engine.set_acid_tuning(Number(message.value));
                break;

            case "set-acid-cutoff":
                engine.set_acid_cutoff(Number(message.value));
                break;

            case "set-acid-resonance":
                engine.set_acid_resonance(Number(message.value));
                break;

            case "set-acid-env-mod":
                engine.set_acid_env_mod(Number(message.value));
                break;

            case "set-acid-decay":
                engine.set_acid_decay(Number(message.value));
                break;

            case "set-acid-accent-amount":
                engine.set_acid_accent_amount(Number(message.value));
                break;

            case "set-acid-glide-time":
                engine.set_acid_glide_time(Number(message.value));
                break;

            case "set-acid-master-volume":
                engine.set_acid_master_volume(Number(message.value));
                break;

            // --- mixer -----------------------------------------------------
            case "set-channel-status":
                engine.set_channel_status(Number(message.channel), Number(message.value));
                break;

            case "set-channel-volume":
                engine.set_channel_volume(Number(message.channel), Number(message.value));
                break;

            case "set-channel-pan":
                engine.set_channel_pan(Number(message.channel), Number(message.value));
                break;

            case "set-channel-eq-low-gain":
                engine.set_channel_eq_low_gain(Number(message.channel), Number(message.value));
                break;

            case "set-channel-eq-mid-gain":
                engine.set_channel_eq_mid_gain(Number(message.channel), Number(message.value));
                break;

            case "set-channel-eq-high-gain":
                engine.set_channel_eq_high_gain(Number(message.channel), Number(message.value));
                break;

            case "set-mixer-master-volume":
                engine.set_mixer_master_volume(Number(message.value));
                break;

            default:
                this.port.postMessage({
                    type: "warning",
                    message: `Unknown message: ${message.type}`
                });
        }
    }

    process(inputs, outputs) {
        const output = outputs[0];

        if (!output || output.length < 2) {
            return true;
        }

        const left = output[0];
        const right = output[1];

        this.engine.fill_buffer(left, right);

        for (let index = 0; index < left.length; index++) {
            if (!Number.isFinite(left[index])) left[index] = 0.0;
            if (!Number.isFinite(right[index])) right[index] = 0.0;
        }

        // report the current step only when it actually changes, so the
        // UI can highlight a playhead (drum grid and bass sequencer
        // share this one step, since both ride the same Transport) without
        // spamming the main thread with a message on every single sample
        const currentStep = this.engine.current_step();

        if (currentStep !== this.lastReportedStep) {
            this.lastReportedStep = currentStep;

            this.port.postMessage({
                type: "step",
                step: currentStep
            });
        }

        const status = this.engine.sequencer_status();

        if (status !== this.lastReportedStatus) {
            this.lastReportedStatus = status;

            this.port.postMessage({
                type: "status",
                status
            });
        }

        this.levelReportCounter++;

        if (this.levelReportCounter >= LEVEL_REPORT_INTERVAL_BLOCKS) {
            this.levelReportCounter = 0;

            this.port.postMessage({
                type: "levels",
                channels: MIXER_CHANNEL_INDICES.map(index => this.engine.channel_peak(index)),
                masterLeft: this.engine.master_peak_left(),
                masterRight: this.engine.master_peak_right()
            });
        }

        return true;
    }
}

registerProcessor("session-processor", SessionProcessor);
