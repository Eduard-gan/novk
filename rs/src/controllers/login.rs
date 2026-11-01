use crate::{
    cookies,
    errors::{LibraryError, LibraryResult},
    models::users,
};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::WithRejection;
use loco_rs::prelude::*;
use serde::Deserialize;

// Passwords must not be exposed through form Debug output.
#[derive(Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

async fn page(ViewEngine(view_engine): ViewEngine<TeraView>) -> LibraryResult<Response> {
    Ok(format::view(
        &view_engine,
        "login/page.html",
        data!({"app_name": "NOVK"}),
    )?)
}

async fn submit(
    State(ctx): State<AppContext>,
    WithRejection(Form(form), _): WithRejection<Form<LoginForm>, LibraryError>,
) -> LibraryResult<Response> {
    let user = match users::Model::find_by_email(&ctx.db, &form.email).await {
        Ok(user) => Some(user),
        Err(ModelError::EntityNotFound) => None,
        Err(e) => return Err(LibraryError::Internal(e.into())),
    };
    if let Some(user) = user {
        if user.verify_password(&form.password) {
            tracing::info!(email = form.email, result = "success", "login attempt");
            let jwt = ctx.config.get_jwt_config()?;
            let token = user
                .generate_jwt(&jwt.secret, jwt.expiration)
                .map_err(|e| LibraryError::Internal(e.into()))?;
            return Ok(format::render()
                .cookies(&[cookies::auth_cookie(&ctx.config, token)?])?
                .redirect_with_header_key("Location", "/music")?);
        }
    }
    // Both credential failures have the same event and response.
    tracing::info!(email = form.email, result = "failure", "login attempt");
    Ok(Redirect::to("/login").into_response())
}

async fn root() -> Response {
    Redirect::to("/music").into_response()
}

async fn logout(State(ctx): State<AppContext>) -> LibraryResult<Response> {
    Ok(format::render()
        .cookies(&[cookies::removal_cookie(&ctx.config)?])?
        .redirect_with_header_key("Location", "/login")?)
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/", get(root))
        .add("/login", get(page))
        .add("/login", post(submit))
        .add("/logout", get(logout))
}
