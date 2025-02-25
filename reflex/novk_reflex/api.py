import base64
import hashlib
from datetime import timedelta, datetime
from pathlib import Path
from uuid import uuid4

import reflex as rx
from fastapi import File, UploadFile, Form, Request, Response, HTTPException, status, Depends
from sqlalchemy.exc import NoResultFound

from novk_reflex.models import Song, Playlist, User
from pydantic import BaseModel
from time import sleep
from time import time
from jose import jwt, JWTError


config = rx.config.get_config()


class APIKeyRequest(BaseModel):
    username: str
    password: str


def password_is_valid(user: User, password: str) -> bool:
    # Разбираем хеш
    _, iterations, salt, stored_hash = user.password.split('$')  # algorithm has no use here
    iterations = int(iterations)
    salt = salt.encode()
    stored_hash = base64.b64decode(stored_hash)

    # Генерируем хеш для введенного пароля
    password_hash = hashlib.pbkdf2_hmac('sha256', password.encode(), salt, iterations)

    # Сравниваем с сохраненным хешем
    return password_hash == stored_hash


def issue_access_token(user: User) -> str:
    payload = dict(
        sub=str(user.id),
        exp=datetime.now() + timedelta(minutes=config.access_token_expires_in_minutes)
    )
    return jwt.encode(payload, rx.config.get_config().secret_key, algorithm=config.access_token_algorithm)


def get_current_user(request: Request):
    token = request.cookies.get(config.access_token_cookie_name)
    if not token:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Not authenticated")

    try:
        payload = jwt.decode(token, rx.config.get_config().secret_key, algorithms=[config.access_token_algorithm])
        user_id = int(payload.get("sub"))
        if not user_id:
            raise ValueError(str(user_id))
        with rx.session() as session:
            return User.get_user_by_id(session=session, user_id=user_id)
    except (JWTError, ValueError, NoResultFound) as e:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Invalid token")


CRYPTO_OPERATIONS_DURATION = 0.1
def get_token(api_key_request: APIKeyRequest, response: Response) -> None:
    global CRYPTO_OPERATIONS_DURATION

    try:
        try:
            with rx.session() as session:
                user = User.get_user_by_username(session, api_key_request.username)
        except NoResultFound:
            sleep(CRYPTO_OPERATIONS_DURATION)
            raise

        try:
            start = time()
            assert password_is_valid(user, api_key_request.password)
        except AssertionError:
            CRYPTO_OPERATIONS_DURATION = time() - start
            raise
    except (NoResultFound, AssertionError):
        raise HTTPException(status_code=403, detail="Bad username or password")

    response.set_cookie(
        key=config.access_token_cookie_name,
        value=issue_access_token(user),
        httponly=True,
        secure=config.secure_access_token,
        samesite="Strict",
        max_age=config.access_token_expires_in_minutes * 60
    )


async def add_song(
    user: dict = Depends(get_current_user),
    file: UploadFile = File(...),
    title: str = Form(...),
    artist: str = Form(...),
    user_id: int = Form(...),
    playlist_id: int = Form(...),
):
    """
    Сохраняет файл в хранилище на диске.\n
    Создаёт запись в таблице Song\n
    Создаёт запись в Playlist для user_id {user_id} и playlist number {playlist_id} со ссылкой на Song\n
    """

    file_contents = await file.read()
    extension = file.filename.split(".")[1]
    full_file_name = f"{uuid4()}.{extension}"
    path = Path(rx.config.get_config().audio_dir) / Path(full_file_name)

    # 1
    with open(path, "wb+") as f:
        f.write(file_contents)

    # with rx.session() as session:
    #     # 2
    #     song = Song(title=title, artist=artist, genre_id=1, file=str(path))
    #     session.add(song)
    #     session.flush()
    #     # 3
    #     playlist = Playlist(number=playlist_id, song_id=song.id, user_id=user_id, name="Медиатека")
    #     session.add(playlist)
    #     session.commit()
    #
    # return dict(song=song, playlist=playlist)
