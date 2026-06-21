.PHONY: models migrate

DATABASE_URL ?= sqlite://novk.sqlite?mode=rwc

models:
	sea-orm-cli generate entity -u "$(DATABASE_URL)" -o src/models/_entities --with-serde both

migrate:
	DATABASE_URL="$(DATABASE_URL)" sea-orm-cli migrate up
