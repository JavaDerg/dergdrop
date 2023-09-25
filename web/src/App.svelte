<script lang="ts">
    import { preventDefault } from "./lib/util";
    import FileDrop from "./lib/FileDrop.svelte";
    import Uploader from "./lib/Uploader.svelte";

    let file: File | null = null;
</script>

<main class="hero min-h-screen p-10" on:dragover={preventDefault}>
    {#if file === null}
        <FileDrop bind:file />
    {:else}
        <Uploader bind:file />
    {/if}
</main>
