<script>
    import { onMount } from "svelte";
    import { writable } from "svelte/store";

    const entities = writable([]);
    onMount(async () =>{
        let response = await fetch('http://localhost:4000/api/entities');
        entities.set((await response.json()).entities);
    });
</script>

<h1> Repositories: </h1>
<ul>
    {#each $entities as entity (entity.name)}
      <li> <a href = "/{entity.name}"> {entity.name} </a> </li>
    {/each}
</ul>

