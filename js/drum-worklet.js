import "./text-decoder-polyfill.js";

import {
    initSync,
    DrumMachineEngine
} from "../pkg/versasha_com.js";

class DrumMachineProcessor extends AudioWorkletProcessor {
    constructor(options) {
        super();

        const { wasmModule, samples } =
            options.processorOptions ?? {};

        if (!wasmModule || !samples) {
            throw new Error(
                "Missing wasmModule or samples in processorOptions"
            );
        }

        initSync({
            module: wasmModule
        });

        this.engine = new DrumMachineEngine(
            sampleRate,
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

        this.port.onmessage = event => {
            const message = event.data;

            try {
                switch (message.type) {
                    case "play":
                        this.engine.play();
                        break;

                    case "pause":
                        this.engine.pause();
                        break;

                    case "stop":
                        this.engine.stop();
                        break;

                    case "set-bpm":
                        this.engine.set_bpm(Number(message.value));
                        break;

                    case "set-master-volume":
                        this.engine.set_master_volume(Number(message.value));
                        break;

                    case "set-step":
                        this.engine.set_step(
                            Number(message.step),
                            Number(message.track),
                            Boolean(message.active)
                        );
                        break;

                    case "clear-track-pattern":
                        this.engine.clear_track_pattern(Number(message.track));
                        break;

                    case "clear-all-patterns":
                        this.engine.clear_all_patterns();
                        break;

                    case "trigger-track":
                        this.engine.trigger_track(Number(message.track));
                        break;

                    case "set-track-volume":
                        this.engine.set_track_volume(
                            Number(message.track),
                            Number(message.value)
                        );
                        break;

                    case "set-track-status":
                        this.engine.set_track_status(
                            Number(message.track),
                            Number(message.value)
                        );
                        break;

                    case "set-track-tune":
                        this.engine.set_track_tune(
                            Number(message.track),
                            Number(message.value)
                        );
                        break;

                    case "set-track-attack":
                        this.engine.set_track_attack(
                            Number(message.track),
                            Number(message.value)
                        );
                        break;

                    case "set-track-decay":
                        this.engine.set_track_decay(
                            Number(message.track),
                            Number(message.value)
                        );
                        break;

                    case "set-track-snappy":
                        this.engine.set_track_snappy(
                            Number(message.track),
                            Number(message.value)
                        );
                        break;

                    default:
                        this.port.postMessage({
                            type: "warning",
                            message: `Unknown message: ${message.type}`
                        });
                }
            } catch (error) {
                this.port.postMessage({
                    type: "error",
                    message: String(error)
                });
            }
        };
    }

    process(inputs, outputs) {
        const output = outputs[0];

        if (!output || output.length === 0) {
            return true;
        }

        const left = output[0];

        if (!left) {
            return true;
        }

        for (let index = 0; index < left.length; index++) {
            const sample = this.engine.next_sample();

            left[index] = Number.isFinite(sample) ? sample : 0.0;
        }

        for (let channel = 1; channel < output.length; channel++) {
            output[channel].set(left);
        }

        // report the current step only when it actually changes, so the
        // UI can highlight a playhead without spamming the main thread
        // with a message on every single sample
        const currentStep = this.engine.current_step();

        if (currentStep !== this.lastReportedStep) {
            this.lastReportedStep = currentStep;

            this.port.postMessage({
                type: "step",
                step: currentStep
            });
        }

        return true;
    }
}

registerProcessor(
    "drum-machine-processor",
    DrumMachineProcessor
);
