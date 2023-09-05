<script lang = "ts">
    import Nav from "./Nav.svelte";
    import "./global.css";
    let styling = {
        bg: "white",
    };
</script>

<Nav {styling}/>

<div class = "content" style = "--bg: {styling.bg}">
    <main>
        <slot />
    </main>
</div>

<style>
    .content {
        background-color: var(--bg);
        width: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
    }
    main {
        width: 80rem;
        max-width: 100%;
    }
</style>
