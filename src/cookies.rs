use cookie::{Cookie, SameSite};
use loco_rs::{
    config::{Config, JWTLocation, JWTLocationConfig},
    Error, Result,
};

pub fn name(config: &Config) -> Result<&str> {
    match config.get_jwt_config()?.location.as_ref() {
        Some(JWTLocationConfig::Single(JWTLocation::Cookie { name })) => Ok(name),
        _ => Err(Error::string(
            "JWT location must name exactly one authentication cookie",
        )),
    }
}

pub fn auth_cookie(config: &Config, token: String) -> Result<Cookie<'static>> {
    let mut cookie = Cookie::build((name(config)?.to_string(), token))
        .http_only(true)
        .secure(config.server.host.starts_with("https://"))
        .max_age(cookie::time::Duration::seconds(
            config
                .get_jwt_config()?
                .expiration
                .try_into()
                .map_err(|e| Error::string(&format!("JWT expiration exceeds cookie duration. {e}")))?,
        ))
        .same_site(SameSite::Lax)
        .path("/");
    if let Some(domain) = cookie_domain(&config.server.host) {
        cookie = cookie.domain(domain);
    }
    Ok(cookie.build())
}

pub fn removal_cookie(config: &Config) -> Result<Cookie<'static>> {
    let mut cookie = auth_cookie(config, String::new())?;
    cookie.make_removal();
    Ok(cookie)
}

fn cookie_domain(server_host: &str) -> Option<String> {
    let host = server_host
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or(server_host)
        .split(':')
        .next()
        .unwrap_or(server_host);
    if host == "localhost" || host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }
    Some(host.to_string())
}
