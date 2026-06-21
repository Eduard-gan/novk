use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};

use crate::extractors::CurrentUser;
use crate::models::{playlist_songs, playlists, songs};

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

#[derive(Debug, Deserialize)]
struct PlaylistOperation {
    operation_type: String,
    song_id: i32,
    playlist_id: i32,
}

#[debug_handler]
async fn page(
    State(ctx): State<AppContext>,
    current_user: CurrentUser,
    ViewEngine(view_engine): ViewEngine<TeraView>,
) -> Result<Response> {
    let user_playlists = playlists::Entity::find()
        .filter(playlists::Column::UserId.eq(current_user.id))
        .order_by_desc(playlists::Column::IsDefault)
        .order_by_asc(playlists::Column::Name)
        .all(&ctx.db)
        .await?;

    let default_playlist = user_playlists
        .iter()
        .find(|playlist| playlist.is_default)
        .or_else(|| user_playlists.first());

    let songs = if let Some(default_playlist) = default_playlist {
        playlist_songs::Entity::find()
            .filter(playlist_songs::Column::PlaylistId.eq(default_playlist.id))
            .find_also_related(songs::Entity)
            .order_by_asc(playlist_songs::Column::Position)
            .order_by_asc(playlist_songs::Column::Id)
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
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let current_playlist = default_playlist
        .map(|playlist| playlist.name.clone())
        .unwrap_or_else(|| "Медиатека".to_string());

    let playlists = user_playlists
        .into_iter()
        .map(|playlist| Playlist {
            id: playlist.id,
            name: playlist.name,
        })
        .collect::<Vec<_>>();

    format::view(
        &view_engine,
        "music/page.html",
        data!({
            "songs": songs,
            "playlists": playlists,
            "current_playlist": current_playlist,
        }),
    )
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

#[debug_handler]
async fn upload_stub(
    State(_ctx): State<AppContext>,
    _current_user: CurrentUser,
) -> Result<Response> {
    format::text("upload stub")
}

#[debug_handler]
async fn create_playlist_stub(
    State(_ctx): State<AppContext>,
    _current_user: CurrentUser,
) -> Result<Response> {
    format::text("create playlist stub")
}

#[debug_handler]
async fn playlist_operations_stub(
    State(_ctx): State<AppContext>,
    _current_user: CurrentUser,
    Form(params): Form<PlaylistOperation>,
) -> Result<Response> {
    tracing::info!(
        operation_type = params.operation_type,
        song_id = params.song_id,
        playlist_id = params.playlist_id,
        "playlist operation stub"
    );
    format::text("Операция принята")
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/music", get(page))
        .add("/music/upload", get(upload_stub))
        .add("/music/create-playlist", get(create_playlist_stub))
        .add(
            "/music/ajax/playlist-operations",
            post(playlist_operations_stub),
        )
}
