use cookie::{Cookie, CookieJar, Key, SameSite};
use sha2::{Digest, Sha512};
use uuid::Uuid;

pub fn auth_cookie(
    cookie_name: &str,
    pid: Uuid,
    server_host: &str,
    secret: &str,
) -> Cookie<'static> {
    let secure = server_host.starts_with("https://");
    let mut cookie = Cookie::build((cookie_name.to_string(), pid.to_string()))
        .http_only(true)
        .secure(secure)
        .max_age(::cookie::time::Duration::days(30))
        .same_site(SameSite::Lax)
        .path("/");

    if let Some(domain) = cookie_domain(server_host) {
        cookie = cookie.domain(domain);
    }

    let digest = Sha512::digest(secret);
    let key = Key::from(&digest);
    let mut jar = CookieJar::new();
    jar.signed_mut(&key).add(cookie.build());
    jar.get(cookie_name)
        .expect("signed auth cookie should be present")
        .clone()
}

pub fn verify_auth_cookie(cookie_name: &str, value: &str, secret: &str) -> Option<String> {
    let digest = Sha512::digest(secret);
    let key = Key::from(&digest);
    let mut jar = CookieJar::new();
    jar.add_original(Cookie::new(cookie_name.to_string(), value.to_string()));
    jar.signed(&key)
        .get(cookie_name)
        .map(|cookie| cookie.value().to_string())
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
