use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};
use tera::{Context, Tera};

use crate::{
    repositories::{CommitLog, CommitLogReq},
    Args,
};

const HOSTNAME: &str = "localhost"; // todo: fix - make this actually a proper config'd item.
const PORT: i32 = 4000; // todo: fix - make this actually a proper config'd item.

pub struct Frontend {
    args: Arc<Args>,
    tera: Tera,
}

impl Frontend {
    pub fn new(args: Arc<Args>) -> Self {
        Self {
            args,
            tera: Tera::new("templates/**/*.html")
                .expect("Failed to create Tera instance from templates/"),
        }
    }
    pub async fn index(&self) -> impl IntoResponse {
        axum::response::Redirect::temporary(&format!("http://{HOSTNAME}:{PORT}/entities"))
        // future when we have a homepage, I guess.
        // Html(self.tera.render("index.html", &Context::new()).unwrap())
    }
    pub async fn entities(&self) -> Html<String> {
        let mut c = Context::new();
        let entities = crate::entities::entities(&self.args).await;
        c.insert("entities", &entities.entities);
        Html(self.tera.render("entities.html", &c).unwrap())
    }
    pub async fn repositories(&self, name: &str) -> Html<String> {
        let mut c = Context::new();
        let repos = crate::entities::Entity::repos(&self.args, name).await;
        c.insert("repositories", &repos.repos);
        c.insert("entity_name", name);
        Html(self.tera.render("repositories.html", &c).unwrap())
    }
    pub async fn repository(
        &self,
        entity: &str,
        repo: &str,
        req: &CommitLogReq,
    ) -> Result<Html<String>, StatusCode> {
        let mut c = Context::new();
        c.insert("entity_name", entity);
        c.insert("repository_name", repo);
        c.insert("commit_id", &req.rev);
        c.insert("increment", &req.increment);
        let commits = CommitLog::commit_log(&self.args, entity, repo, req).await;
        c.insert("commits", &commits?.commits);
        Ok(Html(self.tera.render("repository.html", &c).unwrap()))
    }
}
