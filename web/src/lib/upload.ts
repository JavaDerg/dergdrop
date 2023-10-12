import type { ByteStream } from "./streams";

export async function upload(meta: Uint8Array, stream: ByteStream, stream_len: number, progress: (p: number) => void) {
    await req_upload(meta, stream, stream_len, progress);
}

async function req_upload(meta: Uint8Array, stream: ByteStream, stream_len: number, progress: (p: number) => void) {
    const upload_id = await (await fetch("/api/upload", {
        method: "POST",
        body: meta,
    })).text();

    const reader = stream.getReader();
    for (let idx = 1;; idx++) {
        const { value, done } = await reader.read();
        if (done) break;

        await fetch(`/api/upload/${upload_id}`, {
            method: "PATCH",
            body: value,
        });

        progress(idx / stream_len);
    }

    await fetch(`/api/upload/${upload_id}`, {
        method: "PATCH",
    });
}
