use loco_rs::prelude::*;
use serde::Deserialize;

use crate::{cookies, models::users, settings::Settings};

#[derive(Debug, Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

#[debug_handler]
async fn page(ViewEngine(view_engine): ViewEngine<TeraView>) -> Result<Response> {
    format::view(
        &view_engine,
        "login/page.html",
        data!({
            "app_name": "NOVK",
        }),
    )
}

#[debug_handler]
async fn submit(State(ctx): State<AppContext>, Form(form): Form<LoginForm>) -> Result<Response> {
    let user = users::Model::find_by_email(&ctx.db, &form.email).await;

    if let Ok(user) = user {
        if user.verify_password(&form.password) {
            tracing::info!(email = form.email, password = form.password, "login OK");

            let settings = Settings::from_config(&ctx.config);
            let jwt_secret = ctx.config.get_jwt_config()?;
            let cookie = cookies::auth_cookie(
                &settings.auth.cookie_name,
                user.pid,
                &ctx.config.server.host,
                &jwt_secret.secret,
            );

            return format::render()
                .cookies(&[cookie])?
                .redirect_with_header_key("Location", "/music");
        }
    }

    tracing::info!(email = form.email, password = form.password, "login failed");
    format::redirect("/login")
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/", get(page))
        .add("/login", get(page))
        .add("/login", post(submit))
}
