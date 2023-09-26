export function newCompressorStream(stream: ReadableStream<Uint8Array>, size: number): ReadableStream<Uint8Array> {
    const reader = stream.getReader();

    let leftover: Uint8Array | null = null;

    return new ReadableStream({
        async pull(controller) {
            const buffer = new Uint8Array(size);
            let filled = 0;

            if (leftover !== null) {
                buffer.set(leftover);
                filled += leftover.length;
                leftover = null;                
            }

            while (filled < buffer.length) {
                const {value, done} = await reader.read();
                if (done) break;

                const needed = Math.min(buffer.length - filled, value.length);
                if (value.length === needed) {
                    buffer.set(value, filled);
                    filled += needed;           
                    continue;
                }

                buffer.set(value.subarray(0, needed), filled);
                filled += needed;
                leftover = value.subarray(needed);
                break;
            }

            if (filled === 0) {
                controller.close();
                return;
            }

            controller.enqueue(buffer.subarray(0, filled));
        }
    });
}