use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    bgworker::Queue,
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    db::{self, truncate_table},
    environment::Environment,
    task::Tasks,
    Result,
};
use migration::Migrator;
use std::path::Path;

#[allow(unused_imports)]
use crate::{
    controllers, initializers,
    models::_entities::{genres, playlist_songs, playlists, songs, users},
    tasks,
};

pub struct App;
#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(
            initializers::view_engine::ViewEngineInitializer,
        )])
    }

    async fn after_context(ctx: AppContext) -> Result<AppContext> {
        crate::cookies::name(&ctx.config)?;
        let settings = crate::settings::Settings::from_config(&ctx.config)?;
        settings.prepare_media()?;
        ctx.shared_store
            .insert(crate::uploads::PendingUploads::default());
        Ok(ctx)
    }

    async fn before_routes(ctx: &AppContext) -> Result<axum::Router<AppContext>> {
        let router = axum::Router::new();
        if ctx.environment == Environment::Production {
            return Ok(router);
        }
        let settings = crate::settings::Settings::from_config(&ctx.config)?;
        Ok(router.nest_service(
            "/uploaded",
            tower_http::services::ServeDir::new(settings.uploaded_dir)
                .append_index_html_on_directories(false),
        ))
    }

    fn routes(ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes() // controller routes below
            .add_route(controllers::login::routes())
            .add_route(controllers::music::routes())
            .add_route(controllers::music::upload_routes(ctx))
    }
    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn register_tasks(tasks: &mut Tasks) {
        tasks.register(tasks::users::CreateUser);
        // tasks-inject (do not remove)
    }
    async fn truncate(ctx: &AppContext) -> Result<()> {
        truncate_table(&ctx.db, playlist_songs::Entity).await?;
        truncate_table(&ctx.db, playlists::Entity).await?;
        truncate_table(&ctx.db, songs::Entity).await?;
        truncate_table(&ctx.db, genres::Entity).await?;
        truncate_table(&ctx.db, users::Entity).await?;
        Ok(())
    }
    async fn seed(ctx: &AppContext, base: &Path) -> Result<()> {
        db::seed::<users::ActiveModel>(&ctx.db, &base.join("users.yaml").display().to_string())
            .await?;
        Ok(())
    }
}
