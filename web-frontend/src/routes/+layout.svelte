<script lang = "ts">
    import Nav from "./Nav.svelte";
    import "./global.css";
    let styling = {
        bg: "white",
    };
</script>

<Nav {styling}/>

<div class = "content" style = "--bg: {styling.bg}">
    <slot />
</div>

<style>
    .content {
        background-color: var(--bg);
    }
</style>
