import "./text-decoder-polyfill.js";

import {
    initSync,
    SynthEngine
} from "../pkg/versasha_com.js";

class SynthProcessor extends AudioWorkletProcessor {
    constructor(options) {
        super();

        const wasmModule =
            options.processorOptions?.wasmModule;

        if (!wasmModule) {
            throw new Error(
                "Missing WebAssembly.Module in processorOptions"
            );
        }

        initSync({
            module: wasmModule
        });

        this.synth =
            new SynthEngine(sampleRate);

        this.port.onmessage = event => {
            const message = event.data;

            try {
                switch (message.type) {
                    case "note-on":
                        this.synth.note_on(
                            Number(message.frequency)
                        );
                        break;

                    case "note-off":
                        this.synth.note_off(
                            Number(message.frequency)
                        );
                        break;

                    case "master-volume":
                        this.synth.set_master_volume(
                            Number(message.value)
                        );
                        break;

                    case "osc1-level":
                        this.synth.set_osc1_level(
                            Number(message.value)
                        );
                        break;

                    case "osc2-level":
                        this.synth.set_osc2_level(
                            Number(message.value)
                        );
                        break;

                    case "osc2-detune":
                        this.synth.set_osc2_detune(
                            Number(message.value)
                        );
                        break;

                    case "osc1-waveform":
                        this.synth.set_osc1_waveform(
                            Number(message.value)
                        );
                        break;

                    case "osc2-waveform":
                        this.synth.set_osc2_waveform(
                            Number(message.value)
                        );
                        break;

                    case "filter-cutoff":
                        this.synth.set_filter_cutoff(
                            Number(message.value)
                        );
                        break;

                    case "lfo-frequency":
                        this.synth.set_lfo_frequency(
                            Number(message.value)
                        );
                        break;

                    case "lfo-amount":
                        this.synth.set_lfo_amount(
                            Number(message.value)
                        );
                        break;

                    case "lfo-target":
                        this.synth.set_lfo_target(
                            Number(message.value)
                        );
                        break;

                    case "delay-route":
                        this.synth.set_delay_route(
                            Number(message.value)
                        );
                        break;

                    case "delay-time":
                        this.synth.set_delay_time(
                            Number(message.value)
                        );
                        break;

                    case "delay-feedback":
                        this.synth.set_delay_feedback(
                            Number(message.value)
                        );
                        break;

                    case "delay-mix":
                        this.synth.set_delay_mix(
                            Number(message.value)
                        );
                        break;

                    default:
                        this.port.postMessage({
                            type: "warning",
                            message:
                                `Unknown message: ${message.type}`
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

        for (
            let index = 0;
            index < left.length;
            index++
        ) {
            const sample =
                this.synth.next_sample();

            left[index] =
                Number.isFinite(sample)
                    ? sample
                    : 0.0;
        }

        for (
            let channel = 1;
            channel < output.length;
            channel++
        ) {
            output[channel].set(left);
        }

        return true;
    }
}

registerProcessor(
    "synth-processor",
    SynthProcessor
);
