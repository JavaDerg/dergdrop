<script lang="ts">
    import { fly } from "svelte/transition";
    import * as tus from "tus-js-client";
    import sodium from "libsodium-wrappers";

    export let file: File | null;

    $: if(file) uploadFile(file);

    let uploading = false;

    const uploadFile = async (file: File) => {
        if (uploading) return;
        uploading = true;

        await sodium.ready;

        const key = await sodium.crypto_secretstream_xchacha20poly1305_keygen();
        const { state, header } = sodium.crypto_secretstream_xchacha20poly1305_init_push(key);

        const fr = file.stream().getReader();

        const stream = new ReadableStream({
            start(controller) {
                controller.enqueue(header);
                // TODO FIXME HELP ASASDA
                
                let ct = sodium.crypto_secretstream_xchacha20poly1305_push(state, file.name, null, sodium.crypto_secretstream_xchacha20poly1305_TAG_MESSAGE);

                throw "FIXME LIKE RN";
            },
            async pull(controller) {
                const { value, done } = await fr.read();

                if (done) {
                    let ct = sodium.crypto_secretstream_xchacha20poly1305_push(state, new Uint8Array(), null, sodium.crypto_secretstream_xchacha20poly1305_TAG_FINAL);
                    controller.enqueue(ct);
                    controller.close();
                    return;
                }

                let ct = sodium.crypto_secretstream_xchacha20poly1305_push(state, value, null, sodium.crypto_secretstream_xchacha20poly1305_TAG_MESSAGE);
                controller.enqueue(ct);
            }
        })

        console.log(state, header);
    };
</script>

<div
    class="hero-content text-center min-w-full min-h-full
            rounded-2xl 
            border-dashed"
            in:fly={{ y: -100 }}
    >
        <div class="max-w-md">
        <span class="loading loading-spinner loading-lg"></span>
    </div>
</div>
