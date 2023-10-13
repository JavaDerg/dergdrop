import sodium, { to_base64 } from 'libsodium-wrappers';
import type { Meta } from './meta';

export type ByteStream = ReadableStream<Uint8Array>;

export function newCompressorStream(stream: ByteStream, size: number): ByteStream {
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
                const { value, done } = await reader.read();
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

export function newEncryptingStream(stream: ByteStream, meta: Meta): { key: string, meta: Uint8Array, stream: ByteStream } {
    const master_key = sodium.crypto_kdf_keygen();

    // context has to be 8 bytes
    const meta_key = sodium.crypto_kdf_derive_from_key(sodium.crypto_secretbox_KEYBYTES, 0, "meta____", master_key);
    const stream_key = sodium.crypto_kdf_derive_from_key(sodium.crypto_secretstream_xchacha20poly1305_KEYBYTES, 1, "stream__", master_key);

    const { state, header } = sodium.crypto_secretstream_xchacha20poly1305_init_push(stream_key);

    const header_base64 = sodium.to_base64(header);
    meta.header = header_base64;


    const meta_unpadded = sodium.from_string(JSON.stringify(meta));
    const meta_plaintext = sodium.pad(meta_unpadded, 4096 - sodium.crypto_secretbox_MACBYTES);
    const meta_ciphertext =
        sodium.crypto_secretbox_easy(
            meta_plaintext,
            new Uint8Array(sodium.crypto_secretbox_NONCEBYTES),
            meta_key,
        );
 

    const reader = stream.getReader();

    const out_stream = new ReadableStream({
        async pull(controller) {
            const { value, done } = await reader.read();

            if (done) {
                const ciphertext = 
                    sodium.crypto_secretstream_xchacha20poly1305_push(
                        state,
                        new Uint8Array(),
                        null,
                        sodium.crypto_secretstream_xchacha20poly1305_TAG_FINAL,
                    );
                controller.enqueue(ciphertext);
                controller.close();
                return;
            }

            const ciphertext =
                sodium.crypto_secretstream_xchacha20poly1305_push(
                    state,
                    value,
                    null,
                    sodium.crypto_secretstream_xchacha20poly1305_TAG_PUSH,
                );
            controller.enqueue(ciphertext);
        }
    });

    return {
        key: to_base64(master_key, sodium.base64_variants.URLSAFE),
        meta: meta_ciphertext,
        stream: out_stream,
    };
}