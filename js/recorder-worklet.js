// Sits between the sound sources and audio.destination, passing audio
// through unchanged. While recording is active it also copies each block
// into memory; on stop it hands the whole take back to the main thread as
// one Float32Array per channel, ready to be encoded into a WAV file there
// -- encoding is deliberately not real-time work, so it stays off this
// thread entirely.
class RecorderProcessor extends AudioWorkletProcessor {
    constructor() {
        super();

        this.recording = false;
        this.chunks = null; // chunks[channel] = [Float32Array, Float32Array, ...]

        this.port.onmessage = event => {
            const message = event.data;

            if (message.type === "start") {
                this.recording = true;
                this.chunks = [];
            } else if (message.type === "stop") {
                this.recording = false;
                this.flush();
            }
        };
    }

    flush() {
        if (!this.chunks || this.chunks.length === 0) {
            this.port.postMessage({ type: "recording", channels: [] });
            this.chunks = null;
            return;
        }

        const channelBuffers = this.chunks.map(channelChunks => {
            const length = channelChunks.reduce((sum, chunk) => sum + chunk.length, 0);
            const merged = new Float32Array(length);

            let offset = 0;
            for (const chunk of channelChunks) {
                merged.set(chunk, offset);
                offset += chunk.length;
            }

            return merged;
        });

        this.port.postMessage(
            { type: "recording", channels: channelBuffers },
            channelBuffers.map(buffer => buffer.buffer)
        );

        this.chunks = null;
    }

    process(inputs, outputs) {
        const input = inputs[0];
        const output = outputs[0];

        if (input && output) {
            for (let channel = 0; channel < output.length; channel++) {
                if (input[channel]) {
                    output[channel].set(input[channel]);
                }
            }
        }

        if (this.recording && input && input.length > 0) {
            if (this.chunks.length !== input.length) {
                this.chunks = input.map(() => []);
            }

            for (let channel = 0; channel < input.length; channel++) {
                this.chunks[channel].push(input[channel].slice());
            }
        }

        return true;
    }
}

registerProcessor("recorder-processor", RecorderProcessor);
