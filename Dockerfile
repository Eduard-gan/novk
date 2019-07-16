FROM python:3.7.4

COPY . /root/novk
WORKDIR /root/novk

RUN pip install -r requirements.txt

EXPOSE 8000
ENTRYPOINT ["gunicorn", "novk.wsgi", "-b", "0.0.0.0:8000", "--log-level", "debug", "--log-file", "/var/log/gunicorn.log"]
