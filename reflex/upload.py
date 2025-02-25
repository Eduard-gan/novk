import requests
from pathlib import Path

session = requests.Session()

# Авторизация
session.post(
    url="https://novk.localplayer.dev/api/auth/token",
    headers={"Content-Type": "application/json", "accept": "application/json"},
    json={"username": "root", "password": "ecc14a9t"},
)


files_dir = Path("/home/ed/sources/my/yam_to_novk/pl")


for path in sorted(files_dir.iterdir(), reverse=True):
    _, artist, title = path.name.split(" - ")
    with open(path, "rb") as file:
        response = session.post(
            url="https://novk.localplayer.dev/api/song",
            headers={"accept": "application/json"},
            data={"title": title, "artist": artist, "user_id": "1", "playlist_id": "0"},
            files={"file": (path.name + ".mp3", file, "audio/mpeg")},
        )

    print(path.name, response.status_code, response.text)
