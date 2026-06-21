# Руководство NOVK

## Таск создания пользователя

Создает пользователя в текущей базе Loco.
```bash
cargo loco task users.create email:admin@novk.local password:12345678 name:Admin
```
Аргументы:
- `email` - email для входа.
- `password` - пароль, будет сохранен как hash.
- `name` - отображаемое имя.

## Настройка домена

Домен задается через `DOMAIN`.

```bash
DOMAIN=localhost
DOMAIN=novk.localplayer.dev
```

## Ручное добавление музыки

Файлы хранятся вне БД в uploaded-root. В `songs.file_path` сохраняется путь
относительно uploaded-root, например `audio/demo.mp3`. На фронт он отдается как
`/uploaded/audio/demo.mp3`.

Минимальный пример для SQLite:

```sql
insert into playlists (user_id, name, is_default)
select id, 'Медиатека', true
from users
where email = 'root@gmail.com';

insert into songs (title, artist, file_path, original_filename)
values ('Demo Track', 'Demo Artist', 'audio/demo.mp3', 'demo.mp3');

insert into playlist_songs (playlist_id, song_id, position)
select playlists.id, songs.id, 1
from playlists, songs
where playlists.name = 'Медиатека'
  and songs.file_path = 'audio/demo.mp3';
```
