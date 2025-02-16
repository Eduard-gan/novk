from __future__ import annotations
from datetime import date

import reflex as rx
from sqlalchemy.exc import NoResultFound
from sqlmodel import Field, select


class Genre(rx.Model, table=True):
    __tablename__ = "audio_genre"

    id: int = Field(primary_key=True)
    name: str = Field(max_length=32)


class Song(rx.Model, table=True):
    __tablename__ = "audio_song"

    id: int = Field(primary_key=True)
    title: str = Field(max_length=256)
    artist: str = Field(max_length=256, nullable=False, default="")
    album: str = Field(max_length=256, nullable=False, default="")
    comment: str = Field(max_length=32, nullable=False, default="")
    genre_id: int | None = Field(default=None, foreign_key="audio_genre.id")
    track: int = Field(nullable=True)
    year: int = Field(nullable=True)
    file: str = Field(max_length=512)
    uploaded: date = Field(default_factory=date.today)


class Playlist(rx.Model, table=True):
    __tablename__ = "audio_playlist"

    id: int = Field(primary_key=True)
    name: str = Field(default='Медиатека', max_length=32)
    number: int = Field(nullable=False)
    user_id: int = Field(nullable=False, foreign_key="auth_user.id")
    song_id: int | None = Field(nullable=True, foreign_key="audio_song.id")


class User(rx.Model, table=True):
    __tablename__ = "auth_user"

    id: int = Field(primary_key=True)
    username: str
    password: str

    @classmethod
    def get_user_by_username(cls, session, username: str) -> User:
        user = session.execute(select(cls).where(cls.username == username)).scalar()
        if user:
            return user
        else:
            raise NoResultFound(username)

    @classmethod
    def get_user_by_id(cls, session, user_id: int) -> User:
        user = session.execute(select(cls).where(cls.id == user_id)).scalar()
        if user:
            return user
        else:
            raise NoResultFound(user_id)
