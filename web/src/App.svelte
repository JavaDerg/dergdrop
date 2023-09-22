<main class="hero min-h-screen p-10"
      on:drop={onDrop}
      on:dragover={onDragOver}>
    <div class="hero-content text-center min-w-full min-h-full
                rounded-2xl {over && 'bg-base-200 border-3'}
                border-dashed"
         on:dragenter={() => over = true}
         on:dragleave={() => over = false}
    >
        <section class="max-w-md">
            <h1 class="text-5xl font-bold">Drop your files here</h1>
            <p class="py-6">Files uploaded here are encrypted and will be deleted after their download</p>
            <button class="btn">Upload</button>
        </section>
    </div>
</main>

<script lang="ts">
    let over = false;

    const onDragOver = (ev) => {
        ev.preventDefault();
    };

    const onDrop = (ev: DragEvent) => {
        ev.preventDefault();
        over = false;

        console.log(ev);

        console.log(ev.dataTransfer);

        [...ev.dataTransfer.items].forEach((item, i) => {
            const file = item.getAsFile();
            console.log(file);
        });
    };

</script>
