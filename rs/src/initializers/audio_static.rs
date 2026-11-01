use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};
use std::env;
use std::path::PathBuf;
use tower_http::services::ServeDir;
use tracing::info;

pub struct AudioStaticInitializer;

#[async_trait]
impl Initializer for AudioStaticInitializer {
    fn name(&self) -> String {
        "audio-static".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        let audio_dir = env::var("AUDIO_DIR").unwrap_or_else(|_| "audio".to_string());
        let path = PathBuf::from(&audio_dir);
        info!(path = %path.display(), "serving /uploaded/audio from audio directory");
        Ok(router.nest_service("/uploaded/audio", ServeDir::new(path)))
    }
}
