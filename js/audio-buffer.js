export class AudioBuffer {
    constructor(size) {
        this.buffer = new SharedArrayBuffer(
            Float32Array.BYTES_PER_ELEMENT * size
        );

        this.samples = new Float32Array(
            this.buffer
        );

        this.writeIndex = 0;
        this.readIndex = 0;
        this.size = size;
    }


    write(value) {
        this.samples[this.writeIndex] = value;

        this.writeIndex =
            (this.writeIndex + 1) % this.size;
    }


    read() {
        const value =
            this.samples[this.readIndex];

        this.readIndex =
            (this.readIndex + 1) % this.size;

        return value;
    }
}
