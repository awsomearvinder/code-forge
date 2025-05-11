use axum::Json;

use crate::{get_entries, Args};

#[derive(serde::Serialize)]
pub(crate) struct Entities {
    pub entities: Vec<Entity>,
}

#[derive(serde::Serialize)]
pub(crate) struct Entity {
    name: String,
}

#[derive(serde::Serialize)]
pub(crate) struct Repos {
    pub repos: Vec<Repo>,
}

#[derive(serde::Serialize)]
pub(crate) struct Repo {
    name: String,
}

pub(crate) async fn entities(args: &Args) -> Entities {
    let entities: Vec<Entity> = get_entries(&args.data_dir.join("repositories/"))
        .await
        .into_iter()
        .map(|i| Entity {
            name: i.to_str().unwrap().to_owned(),
        })
        .collect();
    Entities { entities }
}

impl Entity {
    pub(crate) async fn repos(args: &Args, entity_name: &str) -> Repos {
        let repo_entry_links =
            get_entries(&args.data_dir.join(format!("repositories/{entity_name}")))
                .await
                .into_iter()
                .map(|i| Repo {
                    name: i.to_str().unwrap().to_owned(),
                })
                .collect();
        Repos {
            repos: repo_entry_links,
        }
    }
}
