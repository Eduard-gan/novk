FROM python:3.11.2

RUN python -m pip install pipx
RUN pipx install poetry==1.4.0
RUN /root/.local/pipx/venvs/poetry/bin/poetry config virtualenvs.create false
ENV VIRTUAL_ENV=/opt/venv
RUN python -m venv $VIRTUAL_ENV
ENV PATH="$VIRTUAL_ENV/bin:$PATH"

COPY pyproject.toml .
COPY poetry.lock .
RUN /root/.local/pipx/venvs/poetry/bin/poetry install --only main

COPY . /root/novk
WORKDIR /root/novk

EXPOSE 8000
ENTRYPOINT ["gunicorn", "novk.wsgi", "-b", "0.0.0.0:8000", "--log-file", "/var/log/gunicorn.log"]
#ENTRYPOINT ["gunicorn", "novk.asgi", "-b", "0.0.0.0:8000", "-k", "uvicorn.workers.UvicornWorker", "--log-file", "/var/log/gunicorn.log"]
#CMD ["tail", "-f", "/dev/null"]