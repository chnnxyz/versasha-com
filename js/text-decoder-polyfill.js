if (typeof globalThis.TextDecoder === "undefined") {
    globalThis.TextDecoder = class TextDecoder {
        decode(input) {
            if (!input) {
                return "";
            }

            const bytes =
                input instanceof Uint8Array
                    ? input
                    : new Uint8Array(input);

            let result = "";
            let index = 0;

            while (index < bytes.length) {
                const first = bytes[index++];

                if (first < 0x80) {
                    result += String.fromCharCode(first);
                    continue;
                }

                if (first < 0xe0) {
                    const second = bytes[index++];

                    result += String.fromCharCode(
                        ((first & 0x1f) << 6) |
                        (second & 0x3f)
                    );

                    continue;
                }

                if (first < 0xf0) {
                    const second = bytes[index++];
                    const third = bytes[index++];

                    result += String.fromCharCode(
                        ((first & 0x0f) << 12) |
                        ((second & 0x3f) << 6) |
                        (third & 0x3f)
                    );

                    continue;
                }

                const second = bytes[index++];
                const third = bytes[index++];
                const fourth = bytes[index++];

                let codePoint =
                    ((first & 0x07) << 18) |
                    ((second & 0x3f) << 12) |
                    ((third & 0x3f) << 6) |
                    (fourth & 0x3f);

                codePoint -= 0x10000;

                result += String.fromCharCode(
                    0xd800 + (codePoint >> 10),
                    0xdc00 + (codePoint & 0x3ff)
                );
            }

            return result;
        }
    };
}

if (typeof globalThis.TextEncoder === "undefined") {
    globalThis.TextEncoder = class TextEncoder {
        encode(input = "") {
            const bytes = [];

            for (const character of String(input)) {
                const codePoint =
                    character.codePointAt(0);

                if (codePoint <= 0x7f) {
                    bytes.push(codePoint);
                    continue;
                }

                if (codePoint <= 0x7ff) {
                    bytes.push(
                        0xc0 | (codePoint >> 6),
                        0x80 | (codePoint & 0x3f)
                    );

                    continue;
                }

                if (codePoint <= 0xffff) {
                    bytes.push(
                        0xe0 | (codePoint >> 12),
                        0x80 | ((codePoint >> 6) & 0x3f),
                        0x80 | (codePoint & 0x3f)
                    );

                    continue;
                }

                bytes.push(
                    0xf0 | (codePoint >> 18),
                    0x80 | ((codePoint >> 12) & 0x3f),
                    0x80 | ((codePoint >> 6) & 0x3f),
                    0x80 | (codePoint & 0x3f)
                );
            }

            return new Uint8Array(bytes);
        }

        encodeInto(input, destination) {
            const encoded =
                this.encode(input);

            const written =
                Math.min(
                    encoded.length,
                    destination.length
                );

            destination.set(
                encoded.subarray(0, written)
            );

            return {
                read: input.length,
                written
            };
        }
    };
}
