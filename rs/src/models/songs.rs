pub use super::_entities::songs::*;
use super::{genres, playlist_songs, playlists};
use crate::{
    errors::{LibraryError, LibraryResult},
    settings::Settings,
    uploads::PendingUpload,
};
use id3::{Tag, TagLike, Version};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, TransactionTrait};
use serde::{Deserialize, Serialize};
use std::{
    io::{Cursor, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct SongInput {
    pub title: String,
    pub artist: String,
    #[serde(default)]
    pub album: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub year: String,
    #[serde(default)]
    pub track: String,
    #[serde(default)]
    pub genre: String,
}

pub struct SongFields {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub comment: Option<String>,
    pub year: Option<i32>,
    pub track: Option<i32>,
    pub genre: String,
}

impl SongInput {
    pub fn validate(&self) -> LibraryResult<SongFields> {
        let title = self.title.trim();
        if title.is_empty() {
            return Err(LibraryError::SongFormInvalid {
                field: "title",
                detail: "Введите название.",
            });
        }
        if title.chars().count() > 256 {
            return Err(LibraryError::SongFormInvalid {
                field: "title",
                detail: "Название длиннее 256 символов.",
            });
        }
        let artist = self.artist.trim();
        if artist.is_empty() {
            return Err(LibraryError::SongFormInvalid {
                field: "artist",
                detail: "Введите исполнителя.",
            });
        }
        if artist.chars().count() > 256 {
            return Err(LibraryError::SongFormInvalid {
                field: "artist",
                detail: "Исполнитель длиннее 256 символов.",
            });
        }
        let album = self.album.trim();
        if album.chars().count() > 256 {
            return Err(LibraryError::SongFormInvalid {
                field: "album",
                detail: "Альбом длиннее 256 символов.",
            });
        }
        let comment = self.comment.trim();
        if comment.chars().count() > 32 {
            return Err(LibraryError::SongFormInvalid {
                field: "comment",
                detail: "Комментарий длиннее 32 символов.",
            });
        }
        let year =
            if self.year.trim().is_empty() {
                None
            } else {
                Some(self.year.trim().parse::<i32>().map_err(|_e| {
                    LibraryError::SongFormInvalid {
                        field: "year",
                        detail: "Год должен быть целым числом.",
                    }
                })?)
            };
        let track =
            if self.track.trim().is_empty() {
                None
            } else {
                Some(self.track.trim().parse::<i32>().map_err(|_e| {
                    LibraryError::SongFormInvalid {
                        field: "track",
                        detail: "Номер трека должен быть целым числом.",
                    }
                })?)
            };
        let genre = genres::normalize(&self.genre);
        if genre.chars().count() > 32 {
            return Err(LibraryError::SongFormInvalid {
                field: "genre",
                detail: "Жанр длиннее 32 символов.",
            });
        }
        Ok(SongFields {
            title: title.to_string(),
            artist: artist.to_string(),
            album: (!album.is_empty()).then(|| album.to_string()),
            comment: (!comment.is_empty()).then(|| comment.to_string()),
            year,
            track,
            genre,
        })
    }

    pub fn from_tag(tag: &Tag) -> Self {
        let year = tag.year().or_else(|| {
            tag.get("TDRC")
                .and_then(|frame| frame.content().text())
                .and_then(|text| text.get(..4))
                .and_then(|text| text.parse::<i32>().ok())
        });
        let track = tag
            .get("TRCK")
            .and_then(|frame| frame.content().text())
            .and_then(|text| text.split('/').next())
            .and_then(|text| text.parse::<i32>().ok());
        Self {
            title: tag.title().unwrap_or_default().to_string(),
            // id3 decodes ID3v2.3 slash separators into text values, including names such as AC/DC.
            artist: if tag.version() == Version::Id3v23 {
                tag.artists().unwrap_or_default().join("/")
            } else {
                tag.artist().unwrap_or_default().to_string()
            },
            album: tag.album().unwrap_or_default().to_string(),
            comment: String::new(),
            year: year.map(|value| value.to_string()).unwrap_or_default(),
            track: track.map(|value| value.to_string()).unwrap_or_default(),
            genre: tag
                .genre_parsed()
                .map(|value| value.into_owned())
                .unwrap_or_default(),
        }
    }
}

pub(crate) fn read_tag(bytes: &[u8]) -> (Tag, Option<&'static str>) {
    match Tag::read_from2(Cursor::new(bytes)) {
        Ok(tag) => (tag, None),
        Err(e) if matches!(e.kind, id3::ErrorKind::NoTag) => (Tag::new(), None),
        Err(e) => {
            let kind = match e.kind {
                id3::ErrorKind::Io(_) => "Io",
                id3::ErrorKind::StringDecoding(_) => "StringDecoding",
                id3::ErrorKind::Parsing => "Parsing",
                id3::ErrorKind::InvalidInput => "InvalidInput",
                id3::ErrorKind::UnsupportedFeature => "UnsupportedFeature",
                id3::ErrorKind::NoTag => "NoTag",
            };
            (e.partial_tag.unwrap_or_default(), Some(kind))
        }
    }
}

fn remove_failed_file(path: &Path) {
    if let Err(e) = std::fs::remove_file(path) {
        tracing::error!(path = %path.display(), error = ?e, "could not remove failed upload");
    }
}

impl Model {
    pub async fn save_upload(
        db: &DatabaseConnection,
        settings: &Settings,
        user_id: i32,
        upload: PendingUpload,
        fields: SongFields,
    ) -> LibraryResult<Self> {
        let mut tag = upload.tag;
        for id in ["TPE1", "TIT2", "TALB", "TYER", "TDRC", "TRCK", "TCON"] {
            tag.remove(id);
        }
        tag.set_artist(&fields.artist);
        tag.set_title(&fields.title);
        if let Some(album) = &fields.album {
            tag.set_album(album);
        }
        if let Some(year) = fields.year {
            tag.set_year(year);
        }
        if let Some(track) = fields.track {
            tag.set_text("TRCK", track.to_string());
        }
        if !fields.genre.is_empty() {
            tag.set_genre(&fields.genre);
        }

        let relative_path = format!("audio/{}.mp3", upload.uuid);
        let path: PathBuf = settings.uploaded_dir.join(&relative_path);
        let output_path = path.clone();
        tokio::task::spawn_blocking(move || -> LibraryResult<()> {
            let mut file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&output_path)
                .map_err(|e| LibraryError::Internal(e.into()))?;
            let written = (|| -> LibraryResult<()> {
                file.write_all(&upload.bytes)
                    .map_err(|e| LibraryError::Internal(e.into()))?;
                // Unreadable tags retain every original byte, including frames after the error.
                if upload.tag_error.is_none() {
                    tag.write_to_file(&mut file, Version::Id3v23)
                        .map_err(|_e| {
                            LibraryError::Internal(loco_rs::Error::string(
                                "MP3 tag encoding failed",
                            ))
                        })?;
                }
                Ok(())
            })();
            drop(file);
            if written.is_err() {
                remove_failed_file(&output_path);
            }
            written
        })
        .await
        .map_err(|e| {
            LibraryError::Internal(loco_rs::Error::string(&format!(
                "upload writer task failed. {e}"
            )))
        })??;

        let result = async {
            let transaction = db.begin().await?;
            let genre = genres::Model::find_or_create(&transaction, &fields.genre).await?;
            let playlist = playlists::Model::default_for(&transaction, user_id).await?;
            let song = ActiveModel {
                title: Set(fields.title),
                artist: Set(fields.artist),
                album: Set(fields.album),
                comment: Set(fields.comment),
                year: Set(fields.year),
                track: Set(fields.track),
                genre_id: Set(genre.map(|genre| genre.id)),
                file_path: Set(relative_path),
                original_filename: Set(Some(upload.original_filename)),
                ..Default::default()
            }
            .insert(&transaction)
            .await?;
            playlist_songs::Model::add(&transaction, user_id, playlist.id, song.id).await?;
            transaction.commit().await?;
            Ok(song)
        }
        .await;
        if result.is_err() {
            remove_failed_file(&path);
        }
        result
    }
}
