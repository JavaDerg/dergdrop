<script lang="ts">
    import { fly } from 'svelte/transition';
    import { showOpenFilePicker } from "native-file-system-adapter";

    let over = false;

    export let file: File | null;

    const onDrop = (ev: DragEvent) => {
        ev.preventDefault();
        over = false;

        if (ev.dataTransfer == null) return;
        if (
            ev.dataTransfer.items.length > 1 ||
            ev.dataTransfer.files.length > 1
        ) {
            alert("please only upload one file");
            return;
        }

        file =
            ev.dataTransfer.items?.[0]?.getAsFile()
            ?? ev.dataTransfer.files?.[0]
            ?? null;
    };

    let ref = 0
    const dInc = () => {
        ref++;
        over = true;
    };
    const dDec = () => {
        if (--ref === 0)
            over = false;
    };

    const upload = async () => {
        let res = await showOpenFilePicker();

        if (res.length !== 1) {
            return;
        }
        
        file = await res[0].getFile();
    };
</script>

<div
    class="hero-content text-center min-w-full min-h-full
            rounded-2xl {over && 'bg-base-200 border-3 shadow'}
            border-dashed"
    on:dragenter={dInc}
    on:dragleave={dDec}
    on:drop={onDrop}
    role="region"
    out:fly
>
    <div class="max-w-md">
        <h1 class="text-5xl font-bold">Drop your files here</h1>
        <p class="py-6">
            Files uploaded here are encrypted and will be deleted after their
            download
        </p>
        <button class="btn btn-primary" on:click={upload}>Upload</button>
    </div>
</div>
