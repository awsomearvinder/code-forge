<script>
    import { page } from "$app/stores";
    import { onMount } from "svelte";
    import { writable } from "svelte/store";

    const commits = writable([]);
    onMount(async () =>{
        let response = await fetch(`http://localhost:4000/api/${$page.url.pathname.substring(1)}/commits`);
        commits.set((await response.json()).commits);
    });
</script>

<h1> Repositories: </h1>
<ul>
    {#each $commits as commit (commit.message_header)}
      <h3> {commit.message_header} </h3>
      <p> {commit.message_body} </p>
    {/each}
</ul>

