<slot />

<style>
    :global(p, h1, h2, h3, h4, h5, h6, hr) {
        color: #eee;
    }

    :global(*) {
        overflow: hidden;
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }

    :global(html, body) {
        height: 100%;
        width: 100%;
        margin: 0;
        padding: 0;
    }

    :global(html) {
        background-color: #070707;
    }
</style>