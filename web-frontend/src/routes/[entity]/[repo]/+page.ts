import type * as Kit from "@sveltejs/kit";
import { writable } from "svelte/store";

export type Commits = {
    message_header: string,
    message_body: string,
    commit_id: string,
};


export const load: Kit.Load<{}> = async ({url}): Promise<{ commits: Commits[]; }> => {
      const request_params = new URLSearchParams({
      });
      if(url.searchParams.get("rev")) request_params.append("rev", url.searchParams.get("rev") ?? (() => {throw "Failed to get rev"})());

      let response = await fetch(`http://localhost:4000/api/${url.pathname.substring(1)}/commits?` + request_params);

      let log: { commits: Commits[]} = await response.json();
      // skip the first since we've already seen it
      return { commits: log.commits.slice(1) };
};
