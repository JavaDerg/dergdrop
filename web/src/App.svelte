<script lang="ts">
    import { preventDefault } from "./lib/util";
    import FileDrop from "./lib/FileDrop.svelte";
    import Uploader from "./lib/Uploader.svelte";
    import sodium from "libsodium-wrappers";
    import { onMount } from "svelte";
    import Done from "./lib/Done.svelte";
    import Download from "./lib/Download.svelte";

    let ready: boolean = false;

    let file: File | null = null;
    let final_url: string | null = null;

    onMount(async () => {
        await sodium.ready;
        ready = true;

        window.sodium = sodium;
    });
</script>

<main class="hero min-h-screen p-10" on:dragover={preventDefault}>
    {#if ready}
        {#if location.hash !== ""}
            <Download />
        {:else if final_url !== null}
            <Done bind:url={final_url} />
        {:else if file === null}
            <FileDrop bind:file />
        {:else}
            <Uploader bind:file bind:url={final_url} />
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
