<script lang="ts">
    import { page } from "$app/stores";
    import { onMount } from "svelte";
    import { writable } from "svelte/store";
    type Commits = {
        message_header: string,
        message_body: string,
        commit_id: string,
    };
    const commits = writable([] as Commits[]);
    onMount(async () =>{
        let response = await fetch(`http://localhost:4000/api/${$page.url.pathname.substring(1)}/commits`);
        commits.set((await response.json()).commits);
    });
</script>

<h1> Commit Log: </h1>
<ul>
    {#each $commits as commit (commit.message_header)}
      <div class = "commit-log">
          <div class = "commit-header">
              <h3 class="commit-title"> {commit.message_header} </h3>
              <code class = "commit-id"> {commit.commit_id.slice(0, 7)} </code>
          </div>
          {#if commit.message_body}
              <hr>
              <code class = "commit-description"> {commit.message_body.trim()} </code>
          {/if}
      </div>
    {/each}
</ul>


<style>
    ul {
        padding: 0;
        max-width: 80rem;
    }
    .commit-header {
        display: flex;
        align-items: baseline;
        justify-content: space-between;
        width: 100%;
    }
    .commit-log {
        margin: 1rem 0;
        padding: 0rem;
        border: 0.2rem solid black;
    }
    .commit-description {
        white-space: pre-line;
    }
    .commit-title, .commit-id, .commit-description {
        margin: 0.3rem 0.7rem;
    }
    hr {
        width: 100%;
        margin: 0;
        padding: 0;
    }
</style>
