import "./text-decoder-polyfill.js";

import {
    initSync,
    AcidSynthEngine
} from "../pkg/versasha_com.js";

const NUM_STEPS = 16;

class AcidSynthProcessor extends AudioWorkletProcessor {
    constructor(options) {
        super();

        const wasmModule = options.processorOptions?.wasmModule;

        if (!wasmModule) {
            throw new Error("Missing WebAssembly.Module in processorOptions");
        }

        initSync({ module: wasmModule });

        this.synth = new AcidSynthEngine(sampleRate, NUM_STEPS, 120, null);

        this.lastReportedStep = -1;

        this.port.onmessage = event => {
            const message = event.data;

            try {
                switch (message.type) {
                    case "play":
                        this.synth.play();
                        break;

                    case "pause":
                        this.synth.pause();
                        break;

                    case "stop":
                        this.synth.stop();
                        break;

                    case "set-bpm":
                        this.synth.set_bpm(Number(message.value));
                        break;

                    case "set-master-volume":
                        this.synth.set_master_volume(Number(message.value));
                        break;

                    case "set-step":
                        this.synth.set_step(
                            Number(message.index),
                            Number(message.note),
                            Boolean(message.gate),
                            Boolean(message.accent),
                            Boolean(message.slide)
                        );
                        break;

                    case "clear-all-steps":
                        this.synth.clear_all_steps();
                        break;

                    case "set-waveform":
                        this.synth.set_waveform(Number(message.value));
                        break;

                    case "set-tuning":
                        this.synth.set_tuning(Number(message.value));
                        break;

                    case "set-cutoff":
                        this.synth.set_cutoff(Number(message.value));
                        break;

                    case "set-resonance":
                        this.synth.set_resonance(Number(message.value));
                        break;

                    case "set-env-mod":
                        this.synth.set_env_mod(Number(message.value));
                        break;

                    case "set-decay":
                        this.synth.set_decay(Number(message.value));
                        break;

                    case "set-accent-amount":
                        this.synth.set_accent_amount(Number(message.value));
                        break;

                    case "set-glide-time":
                        this.synth.set_glide_time(Number(message.value));
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
            const sample = this.synth.next_sample();

            left[index] = Number.isFinite(sample) ? sample : 0.0;
        }

        for (let channel = 1; channel < output.length; channel++) {
            output[channel].set(left);
        }

        const currentStep = this.synth.current_step();

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

registerProcessor("acid-processor", AcidSynthProcessor);
