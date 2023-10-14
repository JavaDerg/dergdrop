<script lang="ts">
    import sodium from "libsodium-wrappers";
    import { onMount } from "svelte";
    import type { Meta } from "./meta";
    import { filesize } from "filesize";
    import { showSaveFilePicker } from 'native-file-system-adapter' 
    import { newChunkingStream, newDecryptingStream } from "./streams";

    let meta: Meta;

    let position: number | null = null;

    let stream_key: Uint8Array;

    let id: string;

    onMount(async () => {
        id = location.pathname.substring(1);

        const encoded_key = location.hash.substring(1);
        const master_key = sodium.from_base64(encoded_key, sodium.base64_variants.URLSAFE);

        if (master_key.length !== sodium.crypto_kdf_KEYBYTES){
            alert("Invalid url");
            return;
        }

        const meta_key = sodium.crypto_kdf_derive_from_key(sodium.crypto_secretbox_KEYBYTES, 0, "meta____", master_key);
        stream_key = sodium.crypto_kdf_derive_from_key(sodium.crypto_secretstream_xchacha20poly1305_KEYBYTES, 1, "stream__", master_key);

        console.log("master", sodium.to_hex(master_key));
        console.log("meta", sodium.to_hex(meta_key));

        const meta_blob = await (await fetch(`/api/download/${id}/meta`)).blob();
        const meta_ciphertext = new Uint8Array(await meta_blob.arrayBuffer());
        const meta_padded = sodium.crypto_secretbox_open_easy(meta_ciphertext, new Uint8Array(sodium.crypto_secretbox_NONCEBYTES), meta_key);
        const meta_encoded = sodium.unpad(meta_padded, 4096 - sodium.crypto_secretbox_MACBYTES);
        meta = JSON.parse(sodium.to_string(meta_encoded));

        // FIXME: proper errors
        if (meta.header === undefined) throw "invalid meta";
    });

    const download = async () => {
        if (meta.header === undefined) throw "invalid meta";

        const fileHandle = await showSaveFilePicker({
            suggestedName: meta.filename,
        });

        // await hell :3
        const dl_stream = (await (await fetch(`/api/download/${id}`)).blob()).stream();

        const chunked_stream = newChunkingStream(dl_stream, 1024 * 1024 + sodium.crypto_secretstream_xchacha20poly1305_ABYTES);
        
        const decrypted_stream = newDecryptingStream(chunked_stream, meta.header, stream_key);
    
        await decrypted_stream
            .pipeThrough(new TransformStream({
                async transform(chunk, controller) {
                    if (chunk === null) {
                        controller.terminate();
                        return;
                    }

                    controller.enqueue(chunk);

                    if (position === null) position = 0;
                    position += chunk.length;
                },
            }))
            .pipeTo(await fileHandle.createWritable());

            console.log(fileHandle);
    };
</script>

<div
    class="hero-content text-center min-w-full min-h-full
            rounded-2xl
            border-dashed"
>
    <div class="max-w-md">
        <span class="flex flex-col gap-5">
            <h3 class="text-xl">Download <code>{meta?.filename}</code>?</h3>

            {#if position === null}
                <input type="button" class="btn btn-primary" value="Download - {filesize(meta?.filesize ?? 0, {standard: "jedec"})}" on:click={download}>
            {:else if typeof meta === "object"}
                <progress class="progress" value={position / meta.filesize * 100} max="100" />
                <span class="font-bold text-sm">
                    {filesize(position, {standard: "jedec"})}
                    / 
                    {filesize(meta.filesize, {standard: "jedec"})}
                </span>
            {/if}
            <!--
            {#if position === null}
                <input type="button" class="btn btn-primary" value="Download - {filesize(meta?.filesize ?? 0, {standard: "jedec"})}" on:click={download}>
            {:else}
                <progress class="progress w-56" value={position / meta.filesize * 100} max="100" />
                <span>
                    {filesize(position, {standard: "jedec"})}
                    / 
                    {filesize(meta.filesize, {standard: "jedec"})}
                </span>
            {/if} 
            -->
        </span>
    </div>
</div>
