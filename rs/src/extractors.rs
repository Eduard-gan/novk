use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;

use crate::{cookies, models::users, settings::Settings};

pub struct CurrentUser {
    pub id: i32,
    pub pid: Uuid,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    AppContext: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let ctx = AppContext::from_ref(state);
        let settings = Settings::from_config(&ctx.config);
        let jar = cookie::CookieJar::from_headers(&parts.headers);
        let Ok(jwt_secret) = ctx.config.get_jwt_config() else {
            return Err(unauthorized());
        };

        let Some(cookie) = jar.get(&settings.auth.cookie_name) else {
            return Err(unauthorized());
        };
        let Some(pid) = cookies::verify_auth_cookie(
            &settings.auth.cookie_name,
            cookie.value(),
            &jwt_secret.secret,
        ) else {
            return Err(unauthorized());
        };

        let Ok(user) = users::Model::find_by_pid(&ctx.db, &pid).await else {
            return Err(unauthorized());
        };

        Ok(Self {
            id: user.id,
            pid: user.pid,
        })
    }
}

fn unauthorized() -> Response {
    StatusCode::UNAUTHORIZED.into_response()
}
