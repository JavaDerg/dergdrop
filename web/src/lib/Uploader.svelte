<script lang="ts">
    import { fly } from "svelte/transition";
    import sodium from "libsodium-wrappers";
    import { MetaPacker } from "./meta";
    import { newEncryptingStream, newCompressorStream as newRechunkingStream } from "./streams";

    let progress: number | null = null;

    export let file: File | null;

    $: if(file) uploadFile(file);

    let uploading = false;

    const uploadFile = async (file: File) => {
        console.log(file);

        if (uploading) return;
        uploading = true;


        const chunked = newRechunkingStream(file.stream(), 1024 * 1024);
        const { stream: encrypted, meta, key } = newEncryptingStream(chunked, {
            filename: file.name,
            filesize: file.size,
        });

        // TODO: Upload
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
            <span class="loading loading-spinner loading-lg"></span>
        {:else}
            <span class="flex flex-col gap-5">
                <h1 class="text-3xl">Uploading...</h1>
                <progress class="progress w-56" value={progress} max="100"></progress>
            </span>
        {/if}
    </div>
</div>
