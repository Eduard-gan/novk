# NOVK

NOVK — небольшой server-rendered музыкальный сервис на Rust/Loco.rs. Это
переписывание старого Django-приложения без сохранения старой схемы один в
один.

Приложение поддерживает вход, плейлисты, загрузку MP3 с редактированием тегов
и воспроизведение музыки из SQLite и файлового хранилища.

## Текущее состояние

Готово:

- cookie-based login через `GET /login` и `POST /login`;
- JWT Loco в HttpOnly cookie, срок по конфигурации, выход;
- защита SSR-страниц через `CurrentUser`;
- SQLite schema для `users`, `songs`, `genres`, `playlists`, `playlist_songs`;
- автоматическая «Медиатека», выбор плейлиста, новые песни сверху;
- создание плейлистов и добавление существующих песен через UI;
- загрузка MP3 до 20 МиБ в два шага, редактирование ID3v2.3 с сохранением обложки;
- импорт старого дампа в текущую локальную `novk.sqlite`;
- Dockerfile и `docker-compose.yml` для запуска приложения без Caddy.

Вне объёма:

- регистрация, API аутентификации стартера, админка;
- удаление песен из плейлистов;
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
- genre_id -> genres.id nullable, ON DELETE RESTRICT
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
- unique(user_id) WHERE is_default = 1

playlist_songs
- id
- playlist_id -> playlists.id
- song_id -> songs.id
- added_at
- unique(playlist_id, song_id)

genres
- id
- name unique
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

Caddy управляется отдельно и в production отдаёт `/uploaded/...` из каталога
`/mnt/novk-data/data/novk/static/uploaded`. Compose монтирует `/mnt/novk-data`
по тому же пути. В development приложение отдаёт `/uploaded` само.

Корень медиа задаётся `settings.uploaded_dir` из `NOVK_UPLOADED_DIR`. В development
значение по умолчанию `assets/static/uploaded`. В production переменная обязательна.
Корень должен существовать и быть каталогом. Приложение создаёт только `audio/`.
Лимит файла `settings.upload_file_limit` равен 20 971 520 байт. Лимит multipart
равен лимиту файла плюс 1 048 576 байт и действует только на POST загрузки.

Незавершённая загрузка хранится в памяти, одна на пользователя, 30 минут.
Новая успешная загрузка заменяет прежнюю. Перезапуск очищает все слоты.
На диск файл попадает при подтверждении, как `audio/<uuid>.mp3`. Жанры
нормализуются по Unicode в нижний регистр с одним пробелом между словами.
Песня, жанр и связь с «Медиатекой» сохраняются в одной транзакции.

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
NOVK_UPLOADED_DIR=/mnt/novk-data/data/novk/static/uploaded
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
GET  /logout
GET  /music
GET  /music/upload
POST /music/upload
GET  /music/upload/confirm
POST /music/upload/confirm
GET  /music/create-playlist
POST /music/create-playlist
POST /music/ajax/playlist-operations
```

`GET /` перенаправляет на `/music`. Защищённые страницы без действующей cookie
перенаправляют на `/login`. Вход по email выдаёт JWT Loco, проверяется подпись
и срок действия. Имя cookie хранится только в `auth.jwt.location`, тип `Cookie`.
Атрибуты cookie `HttpOnly`, `SameSite=Lax`, `Path=/`, `Secure` при HTTPS.
Выход удаляет cookie, но не отзывает JWT на сервере.

Отказы обработчиков имеют код 409 и `application/problem+json` с `type`, `title`,
`status`, `detail`, а при ошибке поля также `field`. Формы требуют JavaScript
и отправляются через fetch. После успешного POST используется 303, шаг загрузки
ведёт на `/music/upload/confirm`. AJAX возвращает текст при успехе и problem
при отказе. Неизвестный маршрут даёт 404, внутренний сбой 500.
В development и test fallback Loco выключен. CSRF-защита не используется.

## Проверки

CODER выполняет `cargo check --manifest-path rs/Cargo.toml` и
`cargo clippy --manifest-path rs/Cargo.toml --all-targets` из корня репозитория.
TS-01–25 находятся в `tests/requests/parity.rs` и unit-модулях `genres`, `uploads`.
Для запуска тестов QA использует `cargo test --manifest-path rs/Cargo.toml`.
HTTP-тесты создают изолированные SQLite и каталоги медиа. Не используйте
пользовательскую БД для проверки миграций. Перед миграцией существующих данных
нужна резервная копия.

## Production Notes

- `JWT_SECRET` должен быть стабильным между рестартами. При смене секрета все
  auth-cookie станут недействительными.
- `DOMAIN` задается без схемы, например `novk.example.com`.
- Caddy должен проксировать приложение на `127.0.0.1:5150`.
- Caddy должен отдельно отдавать `/uploaded/...` из каталога с MP3.
- SQLite-файл и uploaded-каталог должны жить на persistent storage.
