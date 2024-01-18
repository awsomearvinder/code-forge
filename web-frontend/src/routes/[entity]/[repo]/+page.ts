import type { PageLoad } from "./$types";

export type Commits = {
    message_header: string,
    message_body: string,
    commit_id: string,
};


export const load: PageLoad = async ({url}):
  Promise<{ commits: Commits[]; ref: string; increment: number; }> => {
      let response = await fetch(`http://localhost:4000/api/${url.pathname.substring(1)}/commits?` + url.searchParams);

      let log: { commits: Commits[]} = await response.json();
      // skip the first since we've already seen it
      return {
        ref: url.searchParams.get("rev") ?? log.commits[0].commit_id,
        commits: log.commits.slice(1),
        increment: parseInt(url.searchParams.get("increment") ?? "0"),
      };
};
