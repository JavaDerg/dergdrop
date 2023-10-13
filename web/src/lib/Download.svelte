<script lang="ts">
    import sodium from "libsodium-wrappers";
    import { onMount } from "svelte";
    import type { Meta } from "./meta";
    import { filesize } from "filesize";

    let filename = "";
    let size = 0;

    onMount(async () => {
        const id = location.pathname.substring(1);

        const encoded_key = location.hash.substring(1);
        const master_key = sodium.from_base64(encoded_key, sodium.base64_variants.URLSAFE);
 
        if (master_key.length !== sodium.crypto_kdf_KEYBYTES){
            alert("Invalid url");
            return;
        }

        const meta_key = sodium.crypto_kdf_derive_from_key(sodium.crypto_secretbox_KEYBYTES, 0, "meta____", master_key);
        const stream_key = sodium.crypto_kdf_derive_from_key(sodium.crypto_secretstream_xchacha20poly1305_KEYBYTES, 1, "stream__", master_key);

        console.log("master", sodium.to_hex(master_key));
        console.log("meta", sodium.to_hex(meta_key));

        const meta_blob = await (await fetch(`/api/download/${id}/meta`)).blob();
        const meta_ciphertext = new Uint8Array(await meta_blob.arrayBuffer());
        const meta_padded = sodium.crypto_secretbox_open_easy(meta_ciphertext, new Uint8Array(sodium.crypto_secretbox_NONCEBYTES), meta_key);
        const meta_encoded = sodium.unpad(meta_padded, 4096 - sodium.crypto_secretbox_MACBYTES);
        const meta: Meta = JSON.parse(sodium.to_string(meta_encoded));

        filename = meta.filename;
        size = meta.filesize;
    });
</script>

<div
    class="hero-content text-center min-w-full min-h-full
            rounded-2xl
            border-dashed"
>
    <div class="max-w-md">
        <span class="flex flex-col gap-5">
            <h3 class="text-xl">Download <code>{filename}</code>?</h3>
            <input type="button" class="btn btn-primary" value="Download - {filesize(size)}">
        </span>
    </div>
</div>
