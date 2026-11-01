use axum::{
    extract::{
        multipart::{MultipartError, MultipartRejection},
        rejection::{FormRejection, QueryRejection},
    },
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum LibraryError {
    RequestInvalid,
    PlaylistNotFound,
    PlaylistNameTaken,
    PlaylistNameEmpty,
    PlaylistNameTooLong,
    SongNotFound,
    SongAlreadyInPlaylist,
    UploadFileMissing,
    UploadNotMp3,
    UploadFileTooLarge,
    UploadRequestTooLarge,
    UploadExpired,
    SongFormInvalid {
        field: &'static str,
        detail: &'static str,
    },
    Internal(loco_rs::Error),
}

pub type LibraryResult<T> = std::result::Result<T, LibraryError>;

impl LibraryError {
    fn problem(&self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::RequestInvalid => (
                "request-invalid",
                "Invalid request",
                "Запрос заполнен неверно.",
            ),
            Self::PlaylistNotFound => (
                "playlist-not-found",
                "Playlist not found",
                "Плэйлист не найден.",
            ),
            Self::PlaylistNameTaken => (
                "playlist-name-taken",
                "Playlist name taken",
                "Плэйлист с таким названием уже есть, выберите другое.",
            ),
            Self::PlaylistNameEmpty => (
                "playlist-name-empty",
                "Playlist name empty",
                "Введите название плэйлиста.",
            ),
            Self::PlaylistNameTooLong => (
                "playlist-name-too-long",
                "Playlist name too long",
                "Название плэйлиста длиннее 32 символов.",
            ),
            Self::SongNotFound => ("song-not-found", "Song not found", "Песня не найдена."),
            Self::SongAlreadyInPlaylist => (
                "song-already-in-playlist",
                "Song already added",
                "Эта песня уже есть в этом плэйлисте",
            ),
            Self::UploadFileMissing => ("upload-file-missing", "File missing", "Выберите файл."),
            Self::UploadNotMp3 => (
                "upload-not-mp3",
                "Unsupported file",
                "Файл не является MP3.",
            ),
            Self::UploadFileTooLarge => (
                "upload-file-too-large",
                "File too large",
                "Файл больше 20 МБ.",
            ),
            Self::UploadRequestTooLarge => (
                "upload-request-too-large",
                "Request too large",
                "Запрос больше 21 МБ.",
            ),
            Self::UploadExpired => (
                "upload-expired",
                "Upload expired",
                "Загрузка устарела, загрузите файл снова.",
            ),
            Self::SongFormInvalid { detail, .. } => {
                ("song-form-invalid", "Invalid song field", detail)
            }
            Self::Internal(_) => ("internal-error", "Internal error", "Internal error"),
        }
    }
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.problem().2)
    }
}

impl std::error::Error for LibraryError {}

impl IntoResponse for LibraryError {
    fn into_response(self) -> Response {
        if let Self::Internal(e) = self {
            tracing::error!(error = ?e, "music library operation failed");
            return loco_rs::Error::InternalServerError.into_response();
        }
        let (kind, title, detail) = self.problem();
        let mut problem = json!({"type": kind, "title": title, "status": 409, "detail": detail});
        if let Self::SongFormInvalid { field, .. } = self {
            problem["field"] = json!(field);
        }
        (
            StatusCode::CONFLICT,
            [(header::CONTENT_TYPE, "application/problem+json")],
            Json(problem),
        )
            .into_response()
    }
}

impl From<loco_rs::Error> for LibraryError {
    fn from(e: loco_rs::Error) -> Self {
        Self::Internal(e)
    }
}

impl From<sea_orm::DbErr> for LibraryError {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::Internal(e.into())
    }
}

impl From<FormRejection> for LibraryError {
    fn from(_e: FormRejection) -> Self {
        Self::RequestInvalid
    }
}

impl From<QueryRejection> for LibraryError {
    fn from(_e: QueryRejection) -> Self {
        Self::RequestInvalid
    }
}

impl From<MultipartRejection> for LibraryError {
    fn from(_e: MultipartRejection) -> Self {
        Self::RequestInvalid
    }
}

impl From<MultipartError> for LibraryError {
    fn from(e: MultipartError) -> Self {
        if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
            Self::UploadRequestTooLarge
        } else {
            Self::RequestInvalid
        }
    }
}
