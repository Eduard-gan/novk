use crate::{
    errors::{LibraryError, LibraryResult},
    extractors::CurrentUser,
    models::{playlist_songs, playlists, songs},
    settings::Settings,
    uploads::PendingUploads,
};
use axum::{
    extract::DefaultBodyLimit,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::WithRejection;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Song {
    id: i32,
    artist: String,
    title: String,
    file: String,
}

#[derive(Serialize)]
struct Playlist {
    id: i32,
    name: String,
}

#[derive(Deserialize)]
struct PlaylistQuery {
    playlist: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Operation {
    Add,
}

#[derive(Deserialize)]
struct PlaylistOperation {
    operation_type: Operation,
    song_id: i32,
    playlist_id: i32,
}

#[derive(Deserialize)]
struct PlaylistForm {
    name: String,
}

#[derive(Deserialize)]
struct CommitForm {
    uuid: String,
    #[serde(flatten)]
    song: songs::SongInput,
}

async fn page(
    State(ctx): State<AppContext>,
    current_user: CurrentUser,
    WithRejection(Query(query), _): WithRejection<Query<PlaylistQuery>, LibraryError>,
    ViewEngine(view_engine): ViewEngine<TeraView>,
) -> LibraryResult<Response> {
    let default = playlists::Model::default_for(&ctx.db, current_user.id).await?;
    let selected = match query.playlist {
        Some(id) => playlists::Model::owned(&ctx.db, current_user.id, id).await?,
        None => default,
    };
    let user_playlists = playlists::Entity::find()
        .filter(playlists::Column::UserId.eq(current_user.id))
        .order_by_desc(playlists::Column::IsDefault)
        .order_by_asc(playlists::Column::Name)
        .all(&ctx.db)
        .await?;
    let songs = playlist_songs::Entity::find()
        .filter(playlist_songs::Column::PlaylistId.eq(selected.id))
        .find_also_related(songs::Entity)
        .order_by_desc(songs::Column::Id)
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter_map(|(_, song)| song)
        .map(|song| Song {
            id: song.id,
            artist: song.artist,
            title: song.title,
            file: uploaded_url(&song.file_path),
        })
        .collect::<Vec<_>>();
    let playlists = user_playlists
        .into_iter()
        .map(|playlist| Playlist {
            id: playlist.id,
            name: playlist.name,
        })
        .collect::<Vec<_>>();
    Ok(format::view(
        &view_engine,
        "music/page.html",
        data!({"songs": songs, "playlists": playlists, "current_playlist": selected.name}),
    )?)
}

fn uploaded_url(file_path: &str) -> String {
    if file_path.starts_with('/')
        || file_path.starts_with("http://")
        || file_path.starts_with("https://")
    {
        return file_path.to_string();
    }
    format!("/uploaded/{}", file_path.trim_start_matches('/'))
}

async fn playlist_page(
    _current_user: CurrentUser,
    ViewEngine(view_engine): ViewEngine<TeraView>,
) -> LibraryResult<Response> {
    Ok(format::view(
        &view_engine,
        "music/create_playlist.html",
        data!({}),
    )?)
}

async fn create_playlist(
    State(ctx): State<AppContext>,
    current_user: CurrentUser,
    WithRejection(Form(form), _): WithRejection<Form<PlaylistForm>, LibraryError>,
) -> LibraryResult<Response> {
    playlists::Model::create(&ctx.db, current_user.id, &form.name).await?;
    Ok(Redirect::to("/music").into_response())
}

async fn playlist_operation(
    State(ctx): State<AppContext>,
    current_user: CurrentUser,
    WithRejection(Form(form), _): WithRejection<Form<PlaylistOperation>, LibraryError>,
) -> LibraryResult<Response> {
    match form.operation_type {
        Operation::Add => {
            playlist_songs::Model::add(&ctx.db, current_user.id, form.playlist_id, form.song_id)
                .await?;
        }
    }
    Ok(format::text("Песня добавлена в плэйлист")?)
}

async fn upload_page(
    State(ctx): State<AppContext>,
    _current_user: CurrentUser,
    ViewEngine(view_engine): ViewEngine<TeraView>,
) -> LibraryResult<Response> {
    let settings = Settings::from_config(&ctx.config)?;
    Ok(format::view(
        &view_engine,
        "music/upload.html",
        data!({"file_limit": settings.upload_file_limit}),
    )?)
}

async fn upload(
    State(ctx): State<AppContext>,
    current_user: CurrentUser,
    WithRejection(mut multipart, _): WithRejection<Multipart, LibraryError>,
) -> LibraryResult<Response> {
    let settings = Settings::from_config(&ctx.config)?;
    let mut uploaded = None;
    while let Some(mut field) = multipart.next_field().await? {
        if field.name() == Some("file") && uploaded.is_none() {
            let filename = field.file_name().unwrap_or_default().to_string();
            let mut bytes = Vec::new();
            while let Some(chunk) = field.chunk().await? {
                if chunk.len() > settings.upload_file_limit.saturating_sub(bytes.len()) {
                    return Err(LibraryError::UploadFileTooLarge);
                }
                bytes.extend_from_slice(&chunk);
            }
            if bytes.is_empty() || filename.is_empty() {
                return Err(LibraryError::UploadFileMissing);
            }
            if infer::get(&bytes).is_none_or(|kind| kind.mime_type() != "audio/mpeg") {
                return Err(LibraryError::UploadNotMp3);
            }
            uploaded = Some((bytes, filename));
        } else {
            while field.chunk().await?.is_some() {}
        }
    }
    let (bytes, filename) = uploaded.ok_or(LibraryError::UploadFileMissing)?;
    ctx.shared_store
        .get_ref::<PendingUploads>()
        .expect("pending upload store missing")
        .insert(current_user.id, bytes, filename);
    Ok(Redirect::to("/music/upload/confirm").into_response())
}

async fn confirm(
    State(ctx): State<AppContext>,
    current_user: CurrentUser,
    ViewEngine(view_engine): ViewEngine<TeraView>,
) -> LibraryResult<Response> {
    let store = ctx
        .shared_store
        .get_ref::<PendingUploads>()
        .expect("pending upload store missing");
    let pending = store.get(current_user.id, None)?;
    let song = songs::SongInput::from_tag(&pending.tag);
    Ok(format::view(
        &view_engine,
        "music/confirm.html",
        data!({"uuid": pending.uuid.to_string(), "song": song}),
    )?)
}

async fn commit(
    State(ctx): State<AppContext>,
    current_user: CurrentUser,
    WithRejection(Form(form), _): WithRejection<Form<CommitForm>, LibraryError>,
) -> LibraryResult<Response> {
    let uuid = Uuid::parse_str(&form.uuid).map_err(|_e| LibraryError::UploadExpired)?;
    let store = ctx
        .shared_store
        .get_ref::<PendingUploads>()
        .expect("pending upload store missing");
    store.get(current_user.id, Some(uuid))?;
    let fields = form.song.validate()?;
    let pending = store.take(current_user.id, uuid)?;
    let settings = Settings::from_config(&ctx.config)?;
    songs::Model::save_upload(&ctx.db, &settings, current_user.id, pending, fields).await?;
    Ok(Redirect::to("/music").into_response())
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/music", get(page))
        .add("/music/create-playlist", get(playlist_page))
        .add("/music/create-playlist", post(create_playlist))
        .add("/music/ajax/playlist-operations", post(playlist_operation))
        .add("/music/upload", get(upload_page))
        .add("/music/upload/confirm", get(confirm))
        .add("/music/upload/confirm", post(commit))
}

pub fn upload_routes(ctx: &AppContext) -> Routes {
    let settings = Settings::from_config(&ctx.config)
        .expect("media settings validated during context creation");
    Routes::new()
        .add("/music/upload", post(upload))
        .layer(DefaultBodyLimit::max(settings.request_limit()))
}
