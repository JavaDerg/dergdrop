<script lang="ts">
    import { fly } from "svelte/transition";
    
    export let url: string | null;

    let timeout: number | null = null;

    let link_box: HTMLInputElement;

    const select = () => {
        if (typeof link_box !== "object") return;

        link_box.focus();
        link_box.select();
        link_box.setSelectionRange(0, 99999);

        navigator.clipboard.writeText(link_box.value);

        if (timeout !== null) clearTimeout(timeout);

        timeout = setTimeout(() => {
            timeout = null;
        }, 1400);
    };
</script>

<div
    class="hero-content text-center min-w-full min-h-full
            rounded-2xl
            border-dashed"
    in:fly={{ y: -100 }}
>
    <div class="max-w-md">
        <span class="flex flex-col gap-5">
            <h1 class="text-3xl">Done!</h1>
            
            <div class="{typeof timeout === "number" ? "tooltip" : ""} tooltip-bottom tooltip-open" data-tip="Link copied to clipboard!">
                <div class="join">
                    <div class="input input-success whitespace-nowrap flex items-center bg-base-200 w-full join-item">
                        <input bind:this={link_box} id="link-box" class="bg-transparent input border-0 p-0 w-full" value={url}>
                    </div>
                    <input type="button" class="btn btn-secondary join-item" value="Copy" on:click={select}>
                </div>
            </div>

            <p>
                The link can be used to download the uploaded file <b>once</b>.
            </p>
            <a href="/" class="link link-primary">Upload another file</a>
        </span>
    </div>
</div>
