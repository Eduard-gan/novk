# NOVK

NOVK — небольшой server-rendered музыкальный сервис на Rust/Loco.rs. Это
переписывание старого Django-приложения без сохранения старой схемы один в
один.

Цель текущей версии: авторизоваться, открыть `/music`, прочитать дефолтный
плейлист пользователя из SQLite и проигрывать уже существующие MP3-файлы.

## Текущее состояние

Готово:

- cookie-based login через `GET /login` и `POST /login`;
- signed HttpOnly auth-cookie;
- защита SSR-страниц через `CurrentUser`;
- SQLite schema для `users`, `songs`, `playlists`, `playlist_songs`;
- чтение дефолтного плейлиста пользователя на `/music`;
- импорт старого дампа в текущую локальную `novk.sqlite`;
- Dockerfile и `docker-compose.yml` для запуска приложения без Caddy.

Пока не сделано:

- загрузка MP3 через UI;
- ID3 чтение/редактирование;
- создание плейлистов через UI;
- добавление/удаление песен через UI;
- logout endpoint;
- импорт дампа как повторяемый task.

## Данные

SQLite — основная БД.

Минимальная музыкальная схема:

```text
songs
- id
- title
- artist
- album nullable
- year nullable
- comment nullable
- track nullable
- file_path unique
- original_filename nullable
- file_size_bytes nullable
- uploaded_at
- created_at
- updated_at

playlists
- id
- user_id -> users.id
- name
- is_default
- created_at
- updated_at
- unique(user_id, name)

playlist_songs
- id
- playlist_id -> playlists.id
- song_id -> songs.id
- position nullable
- added_at
- unique(playlist_id, song_id)
```

Старый Django `audio_playlist.number` в новую схему не переносится.
Плейлист и связь песни с плейлистом разделены на разные таблицы.

## Файлы

Статика приложения и загруженная музыка разделены:

```text
/static/...    CSS, JavaScript, изображения приложения
/uploaded/...  MP3-файлы медиатеки
```

В БД `songs.file_path` хранится относительный путь внутри uploaded-root:

```text
audio/0001_Buffalo Tom - Summer.mp3
```

На фронт он отдается как:

```text
/uploaded/audio/0001_Buffalo Tom - Summer.mp3
```

Caddy управляется отдельно и должен отдавать `/uploaded/...` из каталога с
музыкой. В compose этот каталог монтируется в контейнер как `/uploaded`, но само
приложение сейчас не раздает его напрямую.

## Локальный запуск

```bash
cargo loco start
```

Создать пользователя:

```bash
cargo loco task users.create email:admin@novk.local password:12345678 name:Admin
```

Применить миграции:

```bash
cargo loco db migrate
```

Проверить сборку:

```bash
cargo check
```

## Модели и миграции

По образцу `erp-backend` есть Makefile:

```bash
make migrate
make models
```

`make models` генерирует SeaORM entities из текущего состояния БД в
`src/models/_entities`.

По умолчанию используется:

```text
sqlite://novk.sqlite?mode=rwc
```

Можно переопределить:

```bash
make models DATABASE_URL='sqlite:///var/lib/novk/novk.sqlite?mode=rwc'
```

## Docker Compose

Caddy в compose не входит. Он управляется внешне.

Собрать и запустить приложение:

```bash
docker compose build novk
docker compose up -d
```

Compose ожидает `.env`.

Пример:

```env
JWT_SECRET=openssl rand -hex 64
DOMAIN=novk.example.com
NOVK_DATA_DIR=./data
NOVK_UPLOADED_DIR=./uploaded
```

Важно: значение `JWT_SECRET` должно быть реальным секретом. Строка в
`.env.example` показывает команду генерации, а не готовое значение.

В compose:

- SQLite берется из `DATABASE_URL`;
- приложение получает доступ к серверному дереву `/mnt/novk-data` по тому же
  абсолютному пути внутри контейнера;
- приложение слушает `127.0.0.1:5150` на хосте;
- внутри контейнера используется
  `DATABASE_URL=sqlite:///mnt/novk-data/data/novk/novk.sqlite?mode=rwc`;
- `LOCO_ENV=production`;
- `auto_migrate=true`.

Для текущей серверной раскладки:

```text
/mnt/novk-data/services/        код и docker-compose.yml
/mnt/novk-data/data/novk/       persistent data приложения
/mnt/novk-data/data/novk/static/uploaded/audio/  старые MP3-файлы
```

`.env` рядом с compose должен указывать:

```env
DATABASE_URL=sqlite:///mnt/novk-data/data/novk/novk.sqlite?mode=rwc
```

При такой раскладке `songs.file_path = 'audio/file.mp3'` превращается в URL
`/uploaded/audio/file.mp3`, а Caddy должен отдавать этот URL из каталога
`/mnt/novk-data/data/novk/static/uploaded`.

## Caddy

Caddy управляется отдельно от приложения. Минимальный фрагмент Caddyfile:

```caddyfile
novk.example.com {
	handle_path /uploaded/* {
		root * /mnt/novk-data/data/novk/static/uploaded
		file_server
	}

	handle {
		reverse_proxy 127.0.0.1:5150
	}
}
```

Для реального домена заменить `novk.example.com` на значение `DOMAIN`.

Важно:

- `/uploaded/*` должен обрабатываться до `reverse_proxy`;
- приложение само не раздает MP3-файлы в production;
- путь `/mnt/novk-data/data/novk/static/uploaded` должен содержать каталог
  `audio/` со старыми файлами;
- `DOMAIN` в `.env` задается без схемы, например `novk.example.com`.

## Импорт старого дампа

Локально был импортирован `/home/ed/dump.sql`:

- `audio_song` -> `songs`;
- только старый плейлист `Медиатека` -> `playlists`;
- строки `audio_playlist` с `name = 'Медиатека'` -> `playlist_songs`;
- старое поле `number` проигнорировано;
- мусорные тестовые плейлисты из старой БД проигнорированы.

Итог локального импорта:

```text
songs: 685
playlists: 1
playlist_songs: 685
```

## Роуты

Сейчас используются:

```text
GET  /
GET  /login
POST /login
GET  /music
GET  /music/upload              stub
GET  /music/create-playlist     stub
POST /music/ajax/playlist_operations  stub
```

API-auth из Loco starter пока остается в коде, но основной пользовательский flow
для NOVK — HTML-форма логина и signed cookie.

## Production Notes

- `JWT_SECRET` должен быть стабильным между рестартами. При смене секрета все
  auth-cookie станут недействительными.
- `DOMAIN` задается без схемы, например `novk.example.com`.
- Caddy должен проксировать приложение на `127.0.0.1:5150`.
- Caddy должен отдельно отдавать `/uploaded/...` из каталога с MP3.
- SQLite-файл и uploaded-каталог должны жить на persistent storage.
