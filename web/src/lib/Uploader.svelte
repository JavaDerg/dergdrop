<script lang="ts">
    import { fly } from "svelte/transition";
    import sodium from "libsodium-wrappers";
    import { MetaPacker } from "./meta";

    let progress: number | null = null;

    export let file: File | null;

    $: if(file) uploadFile(file);

    let uploading = false;

    const uploadFile = async (file: File) => {
        console.log(file);

        if (uploading) return;
        uploading = true;

        await sodium.ready;

        const key = sodium.crypto_secretstream_xchacha20poly1305_keygen();
        const { state, header } = sodium.crypto_secretstream_xchacha20poly1305_init_push(key);

        const fr = file.stream().getReader();

        let pos = 0;

        const stream = new ReadableStream({
            start(controller) {
                controller.enqueue(header);

                const packer = new MetaPacker();
                packer.push_var_int(file.name.length);
                packer.push_str(file.name);
                packer.push_var_int(file.size);

                const meta_header = packer.build();
                
                const ct = sodium.crypto_secretstream_xchacha20poly1305_push(state, meta_header, null, sodium.crypto_secretstream_xchacha20poly1305_TAG_MESSAGE);
                controller.enqueue(ct);
            },
            async pull(controller) {
                const { value, done } = await fr.read();

                if (done) {
                    const ct = sodium.crypto_secretstream_xchacha20poly1305_push(state, new Uint8Array(), null, sodium.crypto_secretstream_xchacha20poly1305_TAG_FINAL);
                    controller.enqueue(ct);
                    controller.close();
                    return;
                }

                let ct = sodium.crypto_secretstream_xchacha20poly1305_push(state, value, null, sodium.crypto_secretstream_xchacha20poly1305_TAG_MESSAGE);
                controller.enqueue(ct);

                pos += value.length;

                progress = Math.min(pos / file.size * 100, 100);
            }
        });

        const id = await (await fetch("http://localhost:8008/api/upload", {
            method: "POST",
        })).text();

        let reader = stream.getReader();
        while(true) {
            let {value, done} = (await reader.read());

            if (done) break;

            return await fetch(`http://localhost:8008/api/upload/${id}`, {
                body: value,
                method: "PATCH",
            });
        }

        await fetch(`http://localhost:8008/api/upload/${id}`, {
            method: "PATCH",
        });
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
            <progress class="progress w-56" value={progress} max="100"></progress>
        {/if}
    </div>
</div>