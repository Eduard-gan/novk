FROM python:3.11.2

# Устанавливаем pipx и Poetry
RUN python -m pip install pipx
RUN pipx install poetry==1.4.0

# Добавляем путь к pipx в PATH
ENV PATH="/root/.local/bin:$PATH"

# Конфигурируем poetry для работы без создания виртуальных окружений
RUN poetry config virtualenvs.create false

# Настроим виртуальное окружение
ENV VIRTUAL_ENV=/opt/venv
RUN python -m venv $VIRTUAL_ENV
ENV PATH="$VIRTUAL_ENV/bin:$PATH"

# Копируем файлы для установки зависимостей
COPY ./reflex/pyproject.toml .
COPY ./reflex/poetry.lock .
RUN poetry install

# Копируем код приложения
COPY ./reflex /root/reflex
WORKDIR /root/reflex

# Открываем порт 9000
EXPOSE 9000

# Запускаем Reflex
ENTRYPOINT ["reflex", "run", "--backend-port", "9000", "--env", "prod"]
