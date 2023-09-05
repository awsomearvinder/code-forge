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

<h1> Commit Log: </h1>
<ul>
    {#each $commits as commit (commit.message_header)}
      <div class = "commit-log">
          <h3> {commit.message_header} </h3>
          {#if commit.message_body}
              <hr>
              <p> {commit.message_body} </p>
          {/if}
      </div>
    {/each}
</ul>


<style>
    ul {
        padding: 0;
        max-width: 80rem;
    }
    .commit-log {
        margin: 1rem 0;
        padding: 0rem;
        border: 0.2rem solid black;
    }
    .commit-log p {
        margin: 0.2rem;
    }
    .commit-log h3 {
        margin: 0.2rem;
    }
    hr {
        width: 100%;
        margin: 0;
        padding: 0;
    }
</style>
