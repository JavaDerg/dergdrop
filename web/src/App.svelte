<script lang="ts">
    import { preventDefault } from "./lib/util";
    import FileDrop from "./lib/FileDrop.svelte";
    import Uploader from "./lib/Uploader.svelte";
    import sodium from "libsodium-wrappers";
    import { onMount } from "svelte";

    let ready: boolean = false;

    let file: File | null = null;

    onMount(async () => {
        await sodium.ready;
        ready = true;
    })
</script>

<main class="hero min-h-screen p-10" on:dragover={preventDefault}>
    {#if ready}
        {#if file === null}
            <FileDrop bind:file />
        {:else}
            <Uploader bind:file />
        {/if}
    {:else}
        <div
            class="hero-content text-center min-w-full min-h-full
                    rounded-2xl
                    border-dashed"
        >
            <div class="max-w-md">
                <span class="loading loading-spinner loading-lg" />
            </div>
        </div>
    {/if}
</main>
