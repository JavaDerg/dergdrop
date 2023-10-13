<script lang="ts">
    import { fly } from "svelte/transition";
    import sodium from "libsodium-wrappers";
    import {
        newEncryptingStream,
        newCompressorStream as newRechunkingStream,
    } from "./streams";
    import { upload, type UploadMode } from "./upload";

    let progress: number | null = null;

    export let file: File | null;

    export let url: string | null;

    $: if (file) uploadFile(file);

    let uploading = false;

    const uploadFile = async (file: File) => {
        console.log(file);

        if (uploading) return;
        uploading = true;

        const CHUNK_SIZE = 1024 * 1024;

        const chunked = newRechunkingStream(file.stream(), CHUNK_SIZE);
        const {
            stream: encrypted,
            meta,
            key,
        } = newEncryptingStream(chunked, {
            filename: file.name,
            filesize: file.size,
        });

        const expected_chunks = Math.ceil(file.size / CHUNK_SIZE);

        let id = await upload(
            meta,
            encrypted,
            expected_chunks,
            (p) => (progress = p * 100),
        );

        url = `${location.protocol}//${location.host}/${id}#${key}`;
    };
</script>

<div
    class="hero-content text-center min-w-full min-h-full
            rounded-2xl
            border-dashed"
    in:fly={{ y: -100 }}
>
    <div class="max-w-md">
        {#if progress === null}
            <span class="loading loading-spinner loading-lg" />
        {:else}
            <span class="flex flex-col gap-5">
                <h1 class="text-3xl">Uploading...</h1>
                <progress class="progress w-56" value={progress} max="100" />
            </span>
        {/if}
    </div>
</div>
