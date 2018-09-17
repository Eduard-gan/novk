FROM pypy:3

RUN pip install pipenv
WORKDIR /root
COPY . novk
WORKDIR novk
RUN pipenv install

EXPOSE 8000


ENTRYPOINT ["/usr/local/bin/pipenv"]
CMD ["run", "gunicorn", "novk.wsgi", "-b", "0.0.0.0:8000", "--log-level", "debug", "--log-file", "/var/log/gunicorn.log"]
