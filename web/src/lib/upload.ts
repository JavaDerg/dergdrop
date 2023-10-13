import type { ByteStream } from "./streams";

export type UploadMode = "REQ" | "WSC" | "WSF";

export function upload(meta: Uint8Array, stream: ByteStream, stream_len: number, progress: (p: number) => void): Promise<string> {
    // await req_upload(meta, stream, stream_len, progress);
    return ws_upload(meta, stream, stream_len, progress);
}

function ws_upload(meta: Uint8Array, stream: ByteStream, stream_len: number, progress: (p: number) => void): Promise<string> {
    let url = "ws";
    if (location.protocol.endsWith("s:")) {
        url += "s";
    }
    url += `://${location.host}/api/upload/ws`;

    return new Promise((resolve, reject) => {
        const reader = stream.getReader();

        const ws = new WebSocket(url);
        ws.onerror = reject;

        ws.onopen = async () => {
            ws.onmessage = (msg: MessageEvent<string>) => {
                console.log(msg);
                if (typeof msg.data !== "string") {
                    console.error("???");
                    return;
                }
                ws.close();

                resolve(msg.data);
            };

            ws.send(meta);

            for(let idx = 1;; idx++) {
                const { value, done } = await reader.read();
                if (done) {
                    ws.send(new Uint8Array());
                    return;
                };

                ws.send(value);

                progress(idx / stream_len);

                while(ws.bufferedAmount > 1024*1024) await timeout(1);
            }
        }
    });
}

async function req_upload(meta: Uint8Array, stream: ByteStream, stream_len: number, progress: (p: number) => void): Promise<string> {
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

    return upload_id;
}

function timeout(ms: number): Promise<void> {
    return new Promise((resolve, _) => setTimeout(resolve, ms));
}

async function instrument<T>(name: string, fun: () => Promise<T>): Promise<T> {
    const start = Date.now();
    const res = await fun();
    const end = Date.now();

    console.log(name, (end - start) / 1000)

    return res;
}
