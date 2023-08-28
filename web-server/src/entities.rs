use axum::Json;

use crate::{get_entries, Args};

#[derive(serde::Serialize)]
pub(crate) struct Entities {
    entities: Vec<Entity>,
}

#[derive(serde::Serialize)]
struct Entity {
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
