<script>
    import { page } from "$app/stores";
    import { onMount } from "svelte";
    import { writable } from "svelte/store";

    const repos = writable([]);
    onMount(async () =>{
        let response = await fetch(`http://localhost:4000/api/${$page.url.pathname.substring(1)}/repos`);
        repos.set((await response.json()).repos);
    });
</script>

<h1> Repositories: </h1>
<ul>
    {#each $repos as repo (repo.name)}
      <li> <a href = "{$page.url.pathname}/{repo.name}"> {repo.name} </a> </li>
    {/each}
</ul>

