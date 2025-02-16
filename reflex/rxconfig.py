import os

import reflex as rx


config = rx.Config(
    app_name="novk_reflex",
    db_url=f"postgresql://{os.getenv('POSTGRES_USER')}:{os.getenv('POSTGRES_PASSWORD')}@{os.getenv('POSTGRES_HOST')}:5432/{os.getenv('POSTGRES_DB')}",
    audio_dir=os.getenv('AUDIO_DIR'),
    secret_key=os.getenv('SECRET_KEY'),
    access_token_expires_in_minutes=60,
    access_token_cookie_name="novk_api_access_token",
    access_token_algorithm="HS256",
    secure_access_token=os.getenv('SECURE_ACCESS_TOKEN'),
)
