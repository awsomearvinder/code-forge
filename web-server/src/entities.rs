use axum::Json;

use crate::{get_entries, Args};

#[derive(serde::Serialize)]
pub(crate) struct Entities {
    entities: Vec<Entity>,
}

#[derive(serde::Serialize)]
pub(crate) struct Entity {
    name: String,
}

#[derive(serde::Serialize)]
pub(crate) struct Repos {
    repos: Vec<Repo>,
}

#[derive(serde::Serialize)]
struct Repo {
    name: String,
}

pub(crate) async fn entities(args: &Args) -> Json<Entities> {
    let entities: Vec<Entity> = get_entries(&args.data_dir.join("repositories/"))
        .await
        .into_iter()
        .map(|i| Entity {
            name: i.to_str().unwrap().to_owned(),
        })
        .collect();
    Json(Entities { entities })
}

impl Entity {
    pub(crate) async fn repos(args: &Args, entity_name: &str) -> Json<Repos> {
        let repo_entry_links =
            get_entries(&args.data_dir.join(format!("repositories/{entity_name}")))
                .await
                .into_iter()
                .map(|i| Repo {
                    name: i.to_str().unwrap().to_owned(),
                })
                .collect();
        Json(Repos {
            repos: repo_entry_links,
        })
    }
}
