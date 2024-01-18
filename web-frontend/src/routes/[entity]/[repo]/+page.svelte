<script lang="ts">
    import { page } from "$app/stores";
    import type { PageData } from "./$types";
    export let data: PageData;
</script>

<h1> Commit Log: </h1>
<ul>
    {#each data.commits as commit (commit.commit_id)}
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
<a href="{$page.url.pathname}?rev={data.ref}&increment={data.increment - 10 < 0 ? 0 : data.increment  - 10}"> Prev </a>
<a href="{$page.url.pathname}?rev={data.ref}&increment={data.increment + 10}"> Next </a>


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
        margin: 0.5rem 0;
        padding: 0rem;
        border: 0.2rem solid black;
    }
    .commit-description {
        white-space: pre-line;
        box-decoration-break: clone;
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
