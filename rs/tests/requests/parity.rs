use axum::{
    body::Bytes,
    http::{
        header::{CONTENT_TYPE, COOKIE, LOCATION, SET_COOKIE},
        StatusCode,
    },
};
use id3::{Tag, TagLike, Version};
use loco_rs::{app::AppContext, testing::request::request_with_create_db};
use novk::{
    app::App,
    cookies,
    models::{genres, playlist_songs, playlists, songs, users},
    settings::Settings,
    uploads::PendingUploads,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ConnectionTrait, EntityTrait, PaginatorTrait, QueryOrder,
};
use serde_json::{json, Value};
use serial_test::serial;
use std::path::PathBuf;
use uuid::Uuid;

const PASSWORD: &str = "parity-local-test-password";

struct Media {
    path: PathBuf,
    previous: Option<std::ffi::OsString>,
}
impl Media {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("novk-parity-{}", Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        let previous = std::env::var_os("NOVK_TEST_UPLOADED_DIR");
        std::env::set_var("NOVK_TEST_UPLOADED_DIR", &path);
        Self { path, previous }
    }
}
impl Drop for Media {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var("NOVK_TEST_UPLOADED_DIR", value),
            None => std::env::remove_var("NOVK_TEST_UPLOADED_DIR"),
        }
        std::fs::remove_dir_all(&self.path).unwrap();
    }
}

async fn user(ctx: &AppContext, email: &str) -> users::Model {
    users::Model::create_with_password(
        &ctx.db,
        &users::RegisterParams {
            email: email.to_string(),
            password: PASSWORD.to_string(),
            name: "Test User".to_string(),
        },
    )
    .await
    .unwrap()
}

fn cookie(ctx: &AppContext, user: &users::Model) -> String {
    let jwt = ctx.config.get_jwt_config().unwrap();
    format!(
        "{}={}",
        cookies::name(&ctx.config).unwrap(),
        user.generate_jwt(&jwt.secret, jwt.expiration).unwrap()
    )
}

async fn song(ctx: &AppContext, id: i32, title: &str, artist: &str) -> songs::Model {
    songs::ActiveModel {
        id: Set(id),
        title: Set(title.to_string()),
        artist: Set(artist.to_string()),
        file_path: Set(format!("audio/test-{id}.mp3")),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await
    .unwrap()
}

fn check_problem(value: Value, kind: &str, detail: &str) {
    assert_eq!(value["type"], kind);
    assert_eq!(value["status"], 409);
    assert_eq!(value["detail"], detail);
    assert!(!value["title"].as_str().unwrap().is_empty());
}

fn mp3(tagged: bool, cover: bool) -> Vec<u8> {
    let path = std::env::temp_dir().join(format!("novk-fixture-{}.mp3", Uuid::new_v4()));
    std::fs::copy("assets/static/uploaded/audio/demo.mp3", &path).unwrap();
    Tag::remove_from_path(&path).unwrap();
    if tagged {
        let mut tag = Tag::new();
        tag.set_artist("AC/DC");
        tag.set_title("Back in Black");
        tag.set_album("Back in Black");
        tag.set_year(1980);
        tag.set_text("TRCK", "6/10");
        tag.set_genre("Hard Rock");
        if cover {
            tag.add_frame(id3::frame::Picture {
                mime_type: "image/png".to_string(),
                picture_type: id3::frame::PictureType::CoverFront,
                description: String::new(),
                data: std::fs::read("assets/static/image.png").unwrap(),
            });
        }
        tag.write_to_path(&path, Version::Id3v23).unwrap();
    }
    let bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    bytes
}

fn multipart(bytes: &[u8], padding: usize) -> Bytes {
    let mut body =
        b"--novk-boundary\r\nContent-Disposition: form-data; name=\"padding\"\r\n\r\n".to_vec();
    body.resize(body.len() + padding, b'x');
    body.extend_from_slice(b"\r\n--novk-boundary\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.mp3\"\r\nContent-Type: audio/mpeg\r\n\r\n");
    body.extend_from_slice(bytes);
    body.extend_from_slice(b"\r\n--novk-boundary--\r\n");
    body.into()
}

fn fields(uuid: Uuid) -> Value {
    json!({"uuid": uuid.to_string(), "title": "Back in Black", "artist": "AC/DC", "album": "", "year": "1980", "track": "6", "genre": "Hard Rock", "comment": "test"})
}

/// Removed starter routes must not create users.
///
/// Case: TS-01 sends four requests to removed and unknown routes in an empty database.
///
/// Expected: All responses are 404 and the user count remains zero.
#[tokio::test]
#[serial]
async fn ts01_removed_routes() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        request
            .post("/api/auth/register")
            .json(&json!({"email":"new@test.local", "name":"New", "password": PASSWORD}))
            .await
            .assert_status(StatusCode::NOT_FOUND);
        request
            .post("/api/auth/login")
            .await
            .assert_status(StatusCode::NOT_FOUND);
        request
            .get("/api/auth/current")
            .await
            .assert_status(StatusCode::NOT_FOUND);
        request
            .get("/no-such-page")
            .await
            .assert_status(StatusCode::NOT_FOUND);
        assert_eq!(users::Entity::find().count(&ctx.db).await.unwrap(), 0);
    })
    .await;
}

/// Token-only user columns must be removed without removing the API key.
///
/// Case: TS-02 inspects the schema after all migrations on a fresh database.
///
/// Expected: Seven obsolete columns are absent and api_key remains.
#[tokio::test]
#[serial]
async fn ts02_user_columns() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|_request, ctx| async move {
        let rows = ctx
            .db
            .query_all(sea_orm::Statement::from_string(
                sea_orm::DbBackend::Sqlite,
                "PRAGMA table_info(users)",
            ))
            .await
            .unwrap();
        let names = rows
            .iter()
            .map(|row| row.try_get::<String>("", "name").unwrap())
            .collect::<Vec<_>>();
        for name in [
            "reset_token",
            "reset_sent_at",
            "email_verification_token",
            "email_verification_sent_at",
            "email_verified_at",
            "magic_link_token",
            "magic_link_expiration",
        ] {
            assert!(!names.iter().any(|column| column == name));
        }
        assert!(names.iter().any(|column| column == "api_key"));
    })
    .await;
}

/// Valid credentials must create a usable JWT cookie.
///
/// Case: TS-03 logs Anna in and requests her music page with the returned cookie.
///
/// Expected: Login returns 303 with the specified cookie attributes and music returns 200.
#[tokio::test]
#[serial]
async fn ts03_login() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let response = request
            .post("/login")
            .form(&json!({"email": anna.email, "password": PASSWORD}))
            .await;
        response.assert_status(StatusCode::SEE_OTHER);
        assert_eq!(response.header(LOCATION), "/music");
        let header = response.header(SET_COOKIE);
        let cookie_header = header.to_str().unwrap();
        for attribute in [
            "auth=",
            "HttpOnly",
            "SameSite=Lax",
            "Path=/",
            "Max-Age=604800",
        ] {
            assert!(cookie_header.contains(attribute));
        }
        request
            .get("/music")
            .add_header(COOKIE, cookie_header.split(';').next().unwrap())
            .await
            .assert_status(StatusCode::OK);
    })
    .await;
}

/// Invalid credentials must not issue an authentication cookie.
///
/// Case: TS-04 sends a wrong password for Anna and a password for an unknown email.
///
/// Expected: Both requests redirect to login without Set-Cookie.
#[tokio::test]
#[serial]
async fn ts04_invalid_login() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        user(&ctx, "anna@test.local").await;
        for email in ["anna@test.local", "nobody@test.local"] {
            let response = request
                .post("/login")
                .form(&json!({"email":email,"password":"different-test-password"}))
                .await;
            response.assert_status(StatusCode::SEE_OTHER);
            assert_eq!(response.header(LOCATION), "/login");
            assert!(!response.headers().contains_key(SET_COOKIE));
        }
    })
    .await;
}

/// Protected pages must reject missing, invalid, expired, and deleted-user tokens.
///
/// Case: TS-05 requests music with five token states and waits two seconds for a one-second token.
///
/// Expected: Every response is 303 to login.
#[tokio::test]
#[serial]
async fn ts05_authentication_failures() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let jwt = ctx.config.get_jwt_config().unwrap();
        let wrong = anna
            .generate_jwt("ZGlmZmVyZW50LXRlc3Qtc2VjcmV0", 604800)
            .unwrap();
        let expired = anna.generate_jwt(&jwt.secret, 1).unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let boris = user(&ctx, "boris@test.local").await;
        let deleted = cookie(&ctx, &boris);
        users::Entity::delete_by_id(boris.id)
            .exec(&ctx.db)
            .await
            .unwrap();
        request
            .get("/music")
            .await
            .assert_status(StatusCode::SEE_OTHER);
        for value in [
            "auth=garbage".to_string(),
            format!("auth={wrong}"),
            format!("auth={expired}"),
            deleted,
        ] {
            let response = request.get("/music").add_header(COOKIE, value).await;
            response.assert_status(StatusCode::SEE_OTHER);
            assert_eq!(response.header(LOCATION), "/login");
        }
    })
    .await;
}

/// The root and logout must use their specified redirects.
///
/// Case: TS-06 requests root anonymously and logs Anna out with her cookie.
///
/// Expected: Root redirects to music and logout clears the cookie before redirecting to login.
#[tokio::test]
#[serial]
async fn ts06_root_logout() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let response = request.get("/").await;
        response.assert_status(StatusCode::SEE_OTHER);
        assert_eq!(response.header(LOCATION), "/music");
        let response = request
            .get("/logout")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .await;
        response.assert_status(StatusCode::SEE_OTHER);
        assert_eq!(response.header(LOCATION), "/login");
        let header = response.header(SET_COOKIE);
        let value = header.to_str().unwrap();
        for attribute in ["auth=", "Max-Age=0", "Path=/"] {
            assert!(value.contains(attribute));
        }
    })
    .await;
}

/// Music must create exactly one default playlist for a new user.
///
/// Case: TS-07 requests Anna's music page twice with no initial playlists.
///
/// Expected: Both responses are 200, one default exists, and position is absent.
#[tokio::test]
#[serial]
async fn ts07_default_playlist() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let auth = cookie(&ctx, &anna);
        for _ in 0..2 {
            request
                .get("/music")
                .add_header(COOKIE, &auth)
                .await
                .assert_status(StatusCode::OK);
        }
        let rows = playlists::Entity::find().all(&ctx.db).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].user_id, anna.id);
        assert_eq!(rows[0].name, "Медиатека");
        assert!(rows[0].is_default);
        let columns = ctx
            .db
            .query_all(sea_orm::Statement::from_string(
                sea_orm::DbBackend::Sqlite,
                "PRAGMA table_info(playlist_songs)",
            ))
            .await
            .unwrap();
        assert!(!columns
            .iter()
            .any(|column| column.try_get::<String>("", "name").unwrap() == "position"));
    })
    .await;
}

/// Playlist selection must control the songs and newest songs must appear first.
///
/// Case: TS-08 puts songs 1 and 2 in Anna's default and song 1 in Rock.
///
/// Expected: Default shows New before Old and Rock shows only Old with its own label.
#[tokio::test]
#[serial]
async fn ts08_order_and_selection() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let default = playlists::Model::default_for(&ctx.db, anna.id)
            .await
            .unwrap();
        let rock = playlists::Model::create(&ctx.db, anna.id, "Rock")
            .await
            .unwrap();
        song(&ctx, 1, "Old", "Old Artist").await;
        song(&ctx, 2, "New", "New Artist").await;
        for id in [1, 2] {
            playlist_songs::Model::add(&ctx.db, anna.id, default.id, id)
                .await
                .unwrap();
        }
        playlist_songs::Model::add(&ctx.db, anna.id, rock.id, 1)
            .await
            .unwrap();
        let response = request
            .get("/music")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .await;
        response.assert_status(StatusCode::OK);
        let html = response.text();
        assert!(html.find("New Artist - New").unwrap() < html.find("Old Artist - Old").unwrap());
        assert!(html.contains("text-bg-warning\">Медиатека"));
        let response = request
            .get(&format!("/music?playlist={}", rock.id))
            .add_header(COOKIE, cookie(&ctx, &anna))
            .await;
        response.assert_status(StatusCode::OK);
        let html = response.text();
        assert!(html.contains("Old Artist - Old"));
        assert!(!html.contains("New Artist - New"));
        assert!(html.contains("text-bg-warning\">Rock"));
    })
    .await;
}

/// Playlist selection must reject inaccessible or invalid identifiers.
///
/// Case: TS-09 has Anna select Boris's playlist, id 999999, and abc.
///
/// Expected: Three 409 problems identify missing playlists or an invalid request.
#[tokio::test]
#[serial]
async fn ts09_playlist_rejections() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let boris = user(&ctx, "boris@test.local").await;
        let other = playlists::Model::create(&ctx.db, boris.id, "B")
            .await
            .unwrap();
        for (id, kind, detail) in [
            (
                other.id.to_string(),
                "playlist-not-found",
                "Плэйлист не найден.",
            ),
            (
                "999999".to_string(),
                "playlist-not-found",
                "Плэйлист не найден.",
            ),
            (
                "abc".to_string(),
                "request-invalid",
                "Запрос заполнен неверно.",
            ),
        ] {
            let response = request
                .get(&format!("/music?playlist={id}"))
                .add_header(COOKIE, cookie(&ctx, &anna))
                .await;
            response.assert_status(StatusCode::CONFLICT);
            assert_eq!(response.header(CONTENT_TYPE), "application/problem+json");
            check_problem(response.json(), kind, detail);
        }
    })
    .await;
}

/// Creating a playlist must trim its name and redirect to music.
///
/// Case: TS-10 opens the form and submits the name with two spaces on each side of Rock.
///
/// Expected: Form is available and one trimmed Rock playlist is stored for Anna.
#[tokio::test]
#[serial]
async fn ts10_create_playlist() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let response = request
            .get("/music/create-playlist")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .await;
        response.assert_status(StatusCode::OK);
        assert!(response.text().contains("data-fetch-form"));
        let response = request
            .post("/music/create-playlist")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .form(&json!({"name":"  Rock  "}))
            .await;
        response.assert_status(StatusCode::SEE_OTHER);
        assert_eq!(response.header(LOCATION), "/music");
        let rows = playlists::Entity::find().all(&ctx.db).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "Rock");
        assert_eq!(rows[0].user_id, anna.id);
    })
    .await;
}

/// Playlist names must be unique within one user's collection.
///
/// Case: TS-11 submits Rock for Anna who already has Rock and for Boris who does not.
///
/// Expected: Anna gets a precise 409 problem and Boris gets one new playlist.
#[tokio::test]
#[serial]
async fn ts11_duplicate_playlist() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let boris = user(&ctx, "boris@test.local").await;
        playlists::Model::create(&ctx.db, anna.id, "Rock")
            .await
            .unwrap();
        let response = request
            .post("/music/create-playlist")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .form(&json!({"name":"Rock"}))
            .await;
        response.assert_status(StatusCode::CONFLICT);
        check_problem(
            response.json(),
            "playlist-name-taken",
            "Плэйлист с таким названием уже есть, выберите другое.",
        );
        assert_eq!(playlists::Entity::find().count(&ctx.db).await.unwrap(), 1);
        request
            .post("/music/create-playlist")
            .add_header(COOKIE, cookie(&ctx, &boris))
            .form(&json!({"name":"Rock"}))
            .await
            .assert_status(StatusCode::SEE_OTHER);
        assert_eq!(playlists::Entity::find().count(&ctx.db).await.unwrap(), 2);
    })
    .await;
}

/// Playlist names must contain between one and 32 Unicode characters after trimming.
///
/// Case: TS-12 submits three spaces, 33 Cyrillic characters, and 32 Cyrillic characters.
///
/// Expected: The first two get distinct problems and only the 32-character name is stored.
#[tokio::test]
#[serial]
async fn ts12_playlist_length() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        for (name, kind, detail) in [
            (
                "   ".to_string(),
                "playlist-name-empty",
                "Введите название плэйлиста.",
            ),
            (
                "я".repeat(33),
                "playlist-name-too-long",
                "Название плэйлиста длиннее 32 символов.",
            ),
        ] {
            let response = request
                .post("/music/create-playlist")
                .add_header(COOKIE, cookie(&ctx, &anna))
                .form(&json!({"name":name}))
                .await;
            response.assert_status(StatusCode::CONFLICT);
            check_problem(response.json(), kind, detail);
        }
        request
            .post("/music/create-playlist")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .form(&json!({"name":"я".repeat(32)}))
            .await
            .assert_status(StatusCode::SEE_OTHER);
        assert_eq!(playlists::Entity::find().count(&ctx.db).await.unwrap(), 1);
    })
    .await;
}

/// Adding a song twice must preserve exactly one playlist link.
///
/// Case: TS-13 posts song 7 to Anna's Rock playlist twice.
///
/// Expected: Success text then a duplicate problem, with exactly one link.
#[tokio::test]
#[serial]
async fn ts13_add_song() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let rock = playlists::Model::create(&ctx.db, anna.id, "Rock")
            .await
            .unwrap();
        song(&ctx, 7, "Song", "Artist").await;
        let data = json!({"operation_type":"add","song_id":7,"playlist_id":rock.id});
        let response = request
            .post("/music/ajax/playlist-operations")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .form(&data)
            .await;
        response.assert_status(StatusCode::OK);
        assert_eq!(response.text(), "Песня добавлена в плэйлист");
        let response = request
            .post("/music/ajax/playlist-operations")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .form(&data)
            .await;
        response.assert_status(StatusCode::CONFLICT);
        check_problem(
            response.json(),
            "song-already-in-playlist",
            "Эта песня уже есть в этом плэйлисте",
        );
        assert_eq!(
            playlist_songs::Entity::find().count(&ctx.db).await.unwrap(),
            1
        );
    })
    .await;
}

/// Invalid playlist operations must not create links.
///
/// Case: TS-14 uses Boris's playlist, song 999999, song abc, and operation remove.
///
/// Expected: Four typed 409 problems and zero links.
#[tokio::test]
#[serial]
async fn ts14_invalid_operations() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await; let boris = user(&ctx, "boris@test.local").await;
        let own = playlists::Model::create(&ctx.db, anna.id, "Rock").await.unwrap();
        let other = playlists::Model::create(&ctx.db, boris.id, "B").await.unwrap();
        song(&ctx, 7, "Song", "Artist").await;
        for (operation, song_id, playlist_id, kind, detail) in [
            ("add", "7", other.id, "playlist-not-found", "Плэйлист не найден."),
            ("add", "999999", own.id, "song-not-found", "Песня не найдена."),
            ("add", "abc", own.id, "request-invalid", "Запрос заполнен неверно."),
            ("remove", "7", own.id, "request-invalid", "Запрос заполнен неверно.")
        ] {
            let response = request.post("/music/ajax/playlist-operations").add_header(COOKIE, cookie(&ctx, &anna)).form(&json!({"operation_type":operation,"song_id":song_id,"playlist_id":playlist_id})).await;
            response.assert_status(StatusCode::CONFLICT); check_problem(response.json(), kind, detail);
        }
        assert_eq!(playlist_songs::Entity::find().count(&ctx.db).await.unwrap(), 0);
    }).await;
}

/// A normalized genre must be reused and protected while referenced by a song.
///
/// Case: TS-16 creates Rock and rock, Шансон and шансон, then deletes the referenced rock.
///
/// Expected: Two genres, matching ids for each pair, and a rejected delete with rows intact.
#[tokio::test]
#[serial]
async fn ts16_genre_reuse_and_protection() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|_request, ctx| async move {
        let rock = genres::Model::find_or_create(&ctx.db, "Rock")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            genres::Model::find_or_create(&ctx.db, "rock")
                .await
                .unwrap()
                .unwrap()
                .id,
            rock.id
        );
        let chanson = genres::Model::find_or_create(&ctx.db, "Шансон")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            genres::Model::find_or_create(&ctx.db, "шансон")
                .await
                .unwrap()
                .unwrap()
                .id,
            chanson.id
        );
        let track = song(&ctx, 1, "Song", "Artist").await;
        let mut track: songs::ActiveModel = sea_orm::IntoActiveModel::into_active_model(track);
        track.genre_id = Set(Some(rock.id));
        track.update(&ctx.db).await.unwrap();
        assert!(genres::Entity::delete_by_id(rock.id)
            .exec(&ctx.db)
            .await
            .is_err());
        assert_eq!(genres::Entity::find().count(&ctx.db).await.unwrap(), 2);
        assert_eq!(
            songs::Entity::find_by_id(1)
                .one(&ctx.db)
                .await
                .unwrap()
                .unwrap()
                .genre_id,
            Some(rock.id)
        );
    })
    .await;
}

/// Invalid media roots must fail and a valid root must gain its audio directory.
///
/// Case: TS-17 checks missing settings, absent path, file path, and an empty directory.
///
/// Expected: Three distinct errors, path context where applicable, and audio created for the directory.
#[tokio::test]
#[serial]
async fn ts17_media_settings() {
    let media = Media::new();
    request_with_create_db::<App, _, _>(|_request, ctx| async move {
        let mut config = ctx.config.clone();
        config.settings = None;
        let missing = Settings::from_config(&config).unwrap_err().to_string();
        let absent = Settings {
            uploaded_dir: media.path.join("absent"),
            upload_file_limit: 20971520,
        };
        let absent_error = absent.prepare_media().unwrap_err().to_string();
        let file = media.path.join("file");
        std::fs::write(&file, "not a directory").unwrap();
        let invalid = Settings {
            uploaded_dir: file.clone(),
            upload_file_limit: 20971520,
        };
        let file_error = invalid.prepare_media().unwrap_err().to_string();
        assert_ne!(missing, absent_error);
        assert_ne!(absent_error, file_error);
        assert!(absent_error.contains(absent.uploaded_dir.to_str().unwrap()));
        assert!(file_error.contains(file.to_str().unwrap()));
        let root = media.path.join("valid");
        std::fs::create_dir(&root).unwrap();
        Settings {
            uploaded_dir: root.clone(),
            upload_file_limit: 20971520,
        }
        .prepare_media()
        .unwrap();
        assert!(root.join("audio").is_dir());
    })
    .await;
}

fn uuid_from_html(html: &str) -> Uuid {
    let value = html
        .split("name=\"uuid\" value=\"")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    Uuid::parse_str(value).unwrap()
}

/// Uploading MP3 tags must preserve their text and parse year and track numbers.
///
/// Case: TS-19 sends AC/DC, Back in Black, year 1980, track 6/10, and Hard Rock.
///
/// Expected: A 303 leads to a 200 confirmation form containing the unchanged text and numbers.
#[tokio::test]
#[serial]
async fn ts19_upload_tags() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let response = request
            .post("/music/upload")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .bytes(multipart(&mp3(true, false), 0))
            .content_type("multipart/form-data; boundary=novk-boundary")
            .await;
        response.assert_status(StatusCode::SEE_OTHER);
        assert_eq!(response.header(LOCATION), "/music/upload/confirm");
        let response = request
            .get("/music/upload/confirm")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .await;
        response.assert_status(StatusCode::OK);
        let html = response.text();
        for value in ["AC/DC", "Back in Black", "1980", "value=\"6\"", "Hard Rock"] {
            assert!(html.contains(value));
        }
        uuid_from_html(&html);
    })
    .await;
}

/// MP3 files without tags are valid while non-MP3 and missing files are rejected.
///
/// Case: TS-20 uploads a tagless MP3, hello in fake.mp3 for Boris, and an empty multipart form.
///
/// Expected: Empty confirmation fields, upload-not-mp3 with no slot, and upload-file-missing.
#[tokio::test]
#[serial]
async fn ts20_upload_types() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await; let boris = user(&ctx, "boris@test.local").await;
        request.post("/music/upload").add_header(COOKIE, cookie(&ctx, &anna)).bytes(multipart(&mp3(false, false), 0)).content_type("multipart/form-data; boundary=novk-boundary").await.assert_status(StatusCode::SEE_OTHER);
        let response = request.get("/music/upload/confirm").add_header(COOKIE, cookie(&ctx, &anna)).await;
        response.assert_status(StatusCode::OK);
        assert!(response.text().contains("name=\"title\" value=\"\"")); assert!(response.text().contains("name=\"artist\" value=\"\""));
        let response = request.post("/music/upload").add_header(COOKIE, cookie(&ctx, &boris)).bytes(multipart(b"hello", 0)).content_type("multipart/form-data; boundary=novk-boundary").await;
        response.assert_status(StatusCode::CONFLICT); check_problem(response.json(), "upload-not-mp3", "Файл не является MP3.");
        request.get("/music/upload/confirm").add_header(COOKIE, cookie(&ctx, &boris)).await.assert_status(StatusCode::CONFLICT);
        let response = request.post("/music/upload").add_header(COOKIE, cookie(&ctx, &boris)).bytes(Bytes::from_static(b"--novk-boundary\r\nContent-Disposition: form-data; name=\"other\"\r\n\r\nx\r\n--novk-boundary--\r\n")).content_type("multipart/form-data; boundary=novk-boundary").await;
        response.assert_status(StatusCode::CONFLICT); check_problem(response.json(), "upload-file-missing", "Выберите файл.");
    }).await;
}

/// File and request limits must return typed 409 responses at the defined boundaries.
///
/// Case: TS-21 sends MP3 sizes 20971520, 20971521, 24117248, and padding of 22020097 bytes.
///
/// Expected: Exact limit succeeds, file-only excess has its type, both excesses allow either size type, padding has the request type.
#[tokio::test]
#[serial]
async fn ts21_size_limits() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let boris = user(&ctx, "boris@test.local").await;
        for size in [20971520, 20971521, 24117248] {
            let mut bytes = mp3(false, false);
            bytes.resize(size, 0);
            let account = if size == 20971520 { &anna } else { &boris };
            let response = request
                .post("/music/upload")
                .add_header(COOKIE, cookie(&ctx, account))
                .bytes(multipart(&bytes, 0))
                .content_type("multipart/form-data; boundary=novk-boundary")
                .await;
            if size == 20971520 {
                response.assert_status(StatusCode::SEE_OTHER);
                assert_eq!(response.header(LOCATION), "/music/upload/confirm");
            } else {
                response.assert_status(StatusCode::CONFLICT);
                assert_eq!(response.header(CONTENT_TYPE), "application/problem+json");
                let problem: Value = response.json();
                if size == 20971521 {
                    check_problem(problem, "upload-file-too-large", "Файл больше 20 МБ.");
                } else {
                    assert_eq!(problem["status"], 409);
                    match problem["type"].as_str().unwrap() {
                        "upload-file-too-large" => {
                            assert_eq!(problem["detail"], "Файл больше 20 МБ.")
                        }
                        "upload-request-too-large" => {
                            assert_eq!(problem["detail"], "Запрос больше 21 МБ.")
                        }
                        other => panic!("unexpected size failure type {other}"),
                    }
                }
                request
                    .get("/music/upload/confirm")
                    .add_header(COOKIE, cookie(&ctx, &boris))
                    .await
                    .assert_status(StatusCode::CONFLICT);
            }
        }
        let response = request
            .post("/music/upload")
            .add_header(COOKIE, cookie(&ctx, &boris))
            .bytes(multipart(&mp3(false, false), 22020097))
            .content_type("multipart/form-data; boundary=novk-boundary")
            .await;
        response.assert_status(StatusCode::CONFLICT);
        check_problem(
            response.json(),
            "upload-request-too-large",
            "Запрос больше 21 МБ.",
        );
        assert_eq!(songs::Entity::find().count(&ctx.db).await.unwrap(), 0);
    })
    .await;
}

/// Saving a song must preserve cover art, write owned ID3 frames, and link the default playlist.
///
/// Case: TS-22 uploads tagged MP3 with APIC and commits an empty album and a database-only comment.
///
/// Expected: One normalized song and link, ID3v2.3 with correct fields and APIC, no TALB or COMM, and cleared pending state.
#[tokio::test]
#[serial]
async fn ts22_save_upload() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let default = playlists::Model::default_for(&ctx.db, anna.id)
            .await
            .unwrap();
        song(&ctx, 1, "Old", "Artist").await;
        playlist_songs::Model::add(&ctx.db, anna.id, default.id, 1)
            .await
            .unwrap();
        request
            .post("/music/upload")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .bytes(multipart(&mp3(true, true), 0))
            .content_type("multipart/form-data; boundary=novk-boundary")
            .await
            .assert_status(StatusCode::SEE_OTHER);
        let response = request
            .get("/music/upload/confirm")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .await;
        let uuid = uuid_from_html(&response.text());
        let response = request
            .post("/music/upload/confirm")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .form(&fields(uuid))
            .await;
        response.assert_status(StatusCode::SEE_OTHER);
        assert_eq!(response.header(LOCATION), "/music");
        let saved = songs::Entity::find()
            .order_by_desc(songs::Column::Id)
            .one(&ctx.db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved.title, "Back in Black");
        assert_eq!(saved.artist, "AC/DC");
        assert_eq!(saved.album, None);
        assert_eq!(saved.year, Some(1980));
        assert_eq!(saved.track, Some(6));
        assert_eq!(saved.comment.as_deref(), Some("test"));
        assert_eq!(saved.original_filename.as_deref(), Some("test.mp3"));
        assert_eq!(saved.file_path, format!("audio/{uuid}.mp3"));
        let genre = genres::Entity::find_by_id(saved.genre_id.unwrap())
            .one(&ctx.db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(genre.name, "hard rock");
        let tag = Tag::read_from_path(
            Settings::from_config(&ctx.config)
                .unwrap()
                .uploaded_dir
                .join(&saved.file_path),
        )
        .unwrap();
        assert_eq!(tag.version(), Version::Id3v23);
        assert_eq!(
            tag.artists().map(|values| values.join("/")).as_deref(),
            Some("AC/DC")
        );
        assert_eq!(tag.title(), Some("Back in Black"));
        assert_eq!(tag.year(), Some(1980));
        assert_eq!(tag.track(), Some(6));
        assert_eq!(tag.genre(), Some("hard rock"));
        assert!(tag.get("TALB").is_none());
        assert!(tag.get("COMM").is_none());
        let pictures = tag.pictures().collect::<Vec<_>>();
        assert_eq!(pictures.len(), 1);
        assert_eq!(
            pictures[0].data,
            std::fs::read("assets/static/image.png").unwrap()
        );
        let links = playlist_songs::Entity::find().all(&ctx.db).await.unwrap();
        assert_eq!(links.len(), 2);
        assert!(links
            .iter()
            .any(|link| link.playlist_id == default.id && link.song_id == saved.id));
        let html = request
            .get("/music")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .await
            .text();
        assert!(html.find("AC/DC - Back in Black").unwrap() < html.find("Artist - Old").unwrap());
        let columns = ctx
            .db
            .query_all(sea_orm::Statement::from_string(
                sea_orm::DbBackend::Sqlite,
                "PRAGMA table_info(songs)",
            ))
            .await
            .unwrap();
        assert!(!columns
            .iter()
            .any(|row| row.try_get::<String>("", "name").unwrap() == "file_size_bytes"));
        request
            .get("/music/upload/confirm")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .await
            .assert_status(StatusCode::CONFLICT);
    })
    .await;
}

/// Saving a song must reuse an existing normalized genre.
///
/// Case: TS-23 starts with rock and commits an upload whose genre is Rock.
///
/// Expected: One genre remains and the saved song references it.
#[tokio::test]
#[serial]
async fn ts23_upload_reuses_genre() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let rock = genres::Model::find_or_create(&ctx.db, "rock")
            .await
            .unwrap()
            .unwrap();
        request
            .post("/music/upload")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .bytes(multipart(&mp3(false, false), 0))
            .content_type("multipart/form-data; boundary=novk-boundary")
            .await
            .assert_status(StatusCode::SEE_OTHER);
        let html = request
            .get("/music/upload/confirm")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .await
            .text();
        let mut input = fields(uuid_from_html(&html));
        input["genre"] = json!("Rock");
        request
            .post("/music/upload/confirm")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .form(&input)
            .await
            .assert_status(StatusCode::SEE_OTHER);
        assert_eq!(genres::Entity::find().count(&ctx.db).await.unwrap(), 1);
        assert_eq!(
            songs::Entity::find()
                .one(&ctx.db)
                .await
                .unwrap()
                .unwrap()
                .genre_id,
            Some(rock.id)
        );
    })
    .await;
}

/// Invalid song fields must preserve the pending upload for a valid retry.
///
/// Case: TS-24 sends empty title, 257-character artist, abc year, 33-character comment, and 33-character genre.
///
/// Expected: Each response names its field and exact detail, creates no song, and a valid retry succeeds.
#[tokio::test]
#[serial]
async fn ts24_song_validation() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        request
            .post("/music/upload")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .bytes(multipart(&mp3(false, false), 0))
            .content_type("multipart/form-data; boundary=novk-boundary")
            .await
            .assert_status(StatusCode::SEE_OTHER);
        let html = request
            .get("/music/upload/confirm")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .await
            .text();
        let uuid = uuid_from_html(&html);
        for (field, value, detail) in [
            ("title", "".to_string(), "Введите название."),
            (
                "artist",
                "я".repeat(257),
                "Исполнитель длиннее 256 символов.",
            ),
            ("year", "abc".to_string(), "Год должен быть целым числом."),
            (
                "comment",
                "я".repeat(33),
                "Комментарий длиннее 32 символов.",
            ),
            ("genre", "я".repeat(33), "Жанр длиннее 32 символов."),
        ] {
            let mut data = fields(uuid);
            data[field] = json!(value);
            let response = request
                .post("/music/upload/confirm")
                .add_header(COOKIE, cookie(&ctx, &anna))
                .form(&data)
                .await;
            response.assert_status(StatusCode::CONFLICT);
            let problem: Value = response.json();
            assert_eq!(problem["field"], field);
            check_problem(problem, "song-form-invalid", detail);
            assert_eq!(songs::Entity::find().count(&ctx.db).await.unwrap(), 0);
            request
                .get("/music/upload/confirm")
                .add_header(COOKIE, cookie(&ctx, &anna))
                .await
                .assert_status(StatusCode::OK);
        }
        request
            .post("/music/upload/confirm")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .form(&fields(uuid))
            .await
            .assert_status(StatusCode::SEE_OTHER);
        assert_eq!(songs::Entity::find().count(&ctx.db).await.unwrap(), 1);
    })
    .await;
}

/// Missing, foreign, replaced, and consumed pending upload identifiers must be rejected.
///
/// Case: TS-25 uses a random uuid, Boris's uuid, replaced A, consumed B, and confirmation without a slot.
///
/// Expected: Every invalid request gets upload-expired and only the valid save creates one song.
#[tokio::test]
#[serial]
async fn ts25_upload_identifiers() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let boris = user(&ctx, "boris@test.local").await;
        let store = ctx.shared_store.get_ref::<PendingUploads>().unwrap();
        let foreign = store.insert(boris.id, mp3(false, false), "b.mp3".to_string());
        let a = store.insert(anna.id, mp3(false, false), "a.mp3".to_string());
        let b = store.insert(anna.id, mp3(false, false), "b.mp3".to_string());
        for uuid in [Uuid::new_v4(), foreign, a] {
            let response = request
                .post("/music/upload/confirm")
                .add_header(COOKIE, cookie(&ctx, &anna))
                .form(&fields(uuid))
                .await;
            response.assert_status(StatusCode::CONFLICT);
            check_problem(
                response.json(),
                "upload-expired",
                "Загрузка устарела, загрузите файл снова.",
            );
            assert_eq!(songs::Entity::find().count(&ctx.db).await.unwrap(), 0);
        }
        request
            .post("/music/upload/confirm")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .form(&fields(b))
            .await
            .assert_status(StatusCode::SEE_OTHER);
        let response = request
            .post("/music/upload/confirm")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .form(&fields(b))
            .await;
        response.assert_status(StatusCode::CONFLICT);
        check_problem(
            response.json(),
            "upload-expired",
            "Загрузка устарела, загрузите файл снова.",
        );
        let response = request
            .get("/music/upload/confirm")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .await;
        response.assert_status(StatusCode::CONFLICT);
        check_problem(
            response.json(),
            "upload-expired",
            "Загрузка устарела, загрузите файл снова.",
        );
        assert_eq!(songs::Entity::find().count(&ctx.db).await.unwrap(), 1);
    })
    .await;
}

#[derive(Clone, Copy)]
enum TagDamage {
    Header,
    FrameSize,
    Encryption,
}

fn unreadable_mp3(damage: TagDamage) -> Vec<u8> {
    let mut tag = Tag::new();
    tag.set_title("Back in Black");
    tag.set_artist("AC/DC");
    let mut bytes = Vec::new();
    tag.write_to(&mut bytes, Version::Id3v23).unwrap();
    assert_eq!(&bytes[10..14], b"TIT2");
    let title_size = u32::from_be_bytes(bytes[14..18].try_into().unwrap()) as usize;
    let artist_offset = 20 + title_size;
    assert_eq!(&bytes[artist_offset..artist_offset + 4], b"TPE1");
    let complete = Tag::read_from2(std::io::Cursor::new(&bytes)).unwrap();
    assert_eq!(complete.title(), Some("Back in Black"));
    assert_eq!(
        complete.artists().map(|values| values.join("/")).as_deref(),
        Some("AC/DC")
    );
    bytes.extend_from_slice(&mp3(false, false));
    match damage {
        TagDamage::Header => bytes[5] = 0x01,
        TagDamage::FrameSize => {
            // Exceed the whole input so decoding fails even if the parser reads into audio data.
            let invalid_size = u32::try_from(bytes.len() + 1).unwrap().to_be_bytes();
            bytes[artist_offset + 4..artist_offset + 8].copy_from_slice(&invalid_size);
        }
        TagDamage::Encryption => bytes[artist_offset + 9] |= 0x40,
    }
    let error = Tag::read_from2(std::io::Cursor::new(&bytes)).unwrap_err();
    match damage {
        TagDamage::Header => {
            assert!(matches!(error.kind, id3::ErrorKind::Parsing));
            assert!(error.partial_tag.is_none());
        }
        TagDamage::FrameSize | TagDamage::Encryption => {
            if matches!(damage, TagDamage::Encryption) {
                assert!(matches!(error.kind, id3::ErrorKind::UnsupportedFeature));
            }
            let partial = error.partial_tag.unwrap();
            assert_eq!(partial.title(), Some("Back in Black"));
            assert_eq!(partial.artist(), None);
        }
    }
    bytes
}

/// Unreadable ID3 tags must allow confirmation with only fields decoded before the error.
///
/// Case: TS-26 uploads three MP3 copies with header flag 0x01, an oversized second frame, and an encrypted second frame.
///
/// Expected: Each upload redirects to a 200 form, header failure leaves seven fields empty, and frame failures preserve only the title.
#[tokio::test]
#[serial]
async fn ts26_unreadable_tags_show_form() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        for damage in [
            TagDamage::Header,
            TagDamage::FrameSize,
            TagDamage::Encryption,
        ] {
            let bytes = unreadable_mp3(damage);
            let response = request
                .post("/music/upload")
                .add_header(COOKIE, cookie(&ctx, &anna))
                .bytes(multipart(&bytes, 0))
                .content_type("multipart/form-data; boundary=novk-boundary")
                .await;
            response.assert_status(StatusCode::SEE_OTHER);
            assert_eq!(response.header(LOCATION), "/music/upload/confirm");
            for _ in 0..2 {
                let response = request
                    .get("/music/upload/confirm")
                    .add_header(COOKIE, cookie(&ctx, &anna))
                    .await;
                response.assert_status(StatusCode::OK);
                let html = response.text();
                let title = if matches!(damage, TagDamage::Header) {
                    ""
                } else {
                    "Back in Black"
                };
                assert!(html.contains(&format!("name=\"title\" value=\"{title}\"")));
                for field in ["artist", "album", "comment", "year", "track", "genre"] {
                    assert!(html.contains(&format!("name=\"{field}\" value=\"\"")));
                }
                uuid_from_html(&html);
            }
        }
        assert_eq!(songs::Entity::find().count(&ctx.db).await.unwrap(), 0);
        let root = Settings::from_config(&ctx.config).unwrap().uploaded_dir;
        assert_eq!(std::fs::read_dir(root.join("audio")).unwrap().count(), 0);
    })
    .await;
}

/// Saving unreadable ID3 tags must preserve all original bytes and store form values in the library.
///
/// Case: TS-27 saves header and frame failures with title Back in Black, artist AC/DC, year 1980, and genre Rock.
///
/// Expected: Two 303 successes, matching stored bytes, one rock genre, two default-playlist links, and the newest song first.
#[tokio::test]
#[serial]
async fn ts27_unreadable_tags_preserve_original_bytes() {
    let _media = Media::new();
    request_with_create_db::<App, _, _>(|request, ctx| async move {
        let anna = user(&ctx, "anna@test.local").await;
        let mut saved_ids = Vec::new();
        for damage in [TagDamage::Header, TagDamage::FrameSize] {
            let bytes = unreadable_mp3(damage);
            request
                .post("/music/upload")
                .add_header(COOKIE, cookie(&ctx, &anna))
                .bytes(multipart(&bytes, 0))
                .content_type("multipart/form-data; boundary=novk-boundary")
                .await
                .assert_status(StatusCode::SEE_OTHER);
            let response = request
                .get("/music/upload/confirm")
                .add_header(COOKIE, cookie(&ctx, &anna))
                .await;
            response.assert_status(StatusCode::OK);
            let uuid = uuid_from_html(&response.text());
            let mut input = fields(uuid);
            input["genre"] = json!("Rock");
            let response = request
                .post("/music/upload/confirm")
                .add_header(COOKIE, cookie(&ctx, &anna))
                .form(&input)
                .await;
            response.assert_status(StatusCode::SEE_OTHER);
            assert_eq!(response.header(LOCATION), "/music");
            let saved = songs::Entity::find()
                .order_by_desc(songs::Column::Id)
                .one(&ctx.db)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(saved.title, "Back in Black");
            assert_eq!(saved.artist, "AC/DC");
            assert_eq!(saved.year, Some(1980));
            assert_eq!(saved.file_path, format!("audio/{uuid}.mp3"));
            let root = Settings::from_config(&ctx.config).unwrap().uploaded_dir;
            assert_eq!(std::fs::read(root.join(&saved.file_path)).unwrap(), bytes);
            let genre = genres::Entity::find_by_id(saved.genre_id.unwrap())
                .one(&ctx.db)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(genre.name, "rock");
            saved_ids.push(saved.id);
            request
                .get("/music/upload/confirm")
                .add_header(COOKIE, cookie(&ctx, &anna))
                .await
                .assert_status(StatusCode::CONFLICT);
        }
        assert_eq!(songs::Entity::find().count(&ctx.db).await.unwrap(), 2);
        assert_eq!(genres::Entity::find().count(&ctx.db).await.unwrap(), 1);
        let all_playlists = playlists::Entity::find().all(&ctx.db).await.unwrap();
        assert_eq!(all_playlists.len(), 1);
        assert_eq!(all_playlists[0].name, "Медиатека");
        assert!(all_playlists[0].is_default);
        let links = playlist_songs::Entity::find().all(&ctx.db).await.unwrap();
        assert_eq!(links.len(), 2);
        for id in &saved_ids {
            assert!(links
                .iter()
                .any(|link| link.song_id == *id && link.playlist_id == all_playlists[0].id));
        }
        let response = request
            .get("/music")
            .add_header(COOKIE, cookie(&ctx, &anna))
            .await;
        response.assert_status(StatusCode::OK);
        let html = response.text();
        assert_eq!(html.matches("AC/DC - Back in Black").count(), 2);
        let old = format!("id=\"{}\"", saved_ids[0]);
        let new = format!("id=\"{}\"", saved_ids[1]);
        assert!(html.find(&new).unwrap() < html.find(&old).unwrap());
    })
    .await;
}
