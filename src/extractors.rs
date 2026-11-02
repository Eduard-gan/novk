use crate::{cookies, models::users};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
};
use loco_rs::prelude::*;

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

    async fn from_request_parts(parts: &mut Parts, state: &S) -> std::result::Result<Self, Self::Rejection> {
        let ctx = AppContext::from_ref(state);
        let jwt_config = ctx.config.get_jwt_config().map_err(|e| e.into_response())?;
        let cookie_name = cookies::name(&ctx.config).map_err(|e| e.into_response())?;
        let jar = cookie::CookieJar::from_headers(&parts.headers);
        let Some(cookie) = jar.get(cookie_name) else {
            return Err(Redirect::to("/login").into_response());
        };
        let Ok(token) = loco_rs::auth::jwt::JWT::new(&jwt_config.secret).validate(cookie.value()) else {
            return Err(Redirect::to("/login").into_response());
        };
        let user = match users::Model::find_by_pid(&ctx.db, &token.claims.pid).await {
            Ok(user) => user,
            Err(ModelError::EntityNotFound) => return Err(Redirect::to("/login").into_response()),
            Err(e) => return Err(crate::errors::LibraryError::Internal(e.into()).into_response()),
        };
        Ok(Self {
            id: user.id,
            pid: user.pid,
        })
    }
}
