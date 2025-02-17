import reflex as rx

from novk_reflex import api
# from novk_reflex import pages
from novk_reflex.state import IndexState
import logging
from fastapi import Request
from time import time
import http


logger = None
def setup_logging():
    global logger
    uvicorn_access = logging.getLogger("uvicorn.access")
    uvicorn_access.disabled = False
    logger = logging.getLogger("uvicorn")
    logger.setLevel(logging.getLevelName(logging.DEBUG))


async def log_request_middleware(request: Request, call_next):
    url = f"{request.url.path}?{request.query_params}" if request.query_params else request.url.path
    start_time = time()
    response = await call_next(request)
    process_time = (time() - start_time) * 1000
    formatted_process_time = "{0:.2f}".format(process_time)
    host = getattr(getattr(request, "client", None), "host", None)
    port = getattr(getattr(request, "client", None), "port", None)
    try:
        status_phrase = http.HTTPStatus(response.status_code).phrase
    except ValueError:
        status_phrase = ""
    logger.info(
        f'{host}:{port} - "{request.method} {url}" {response.status_code} {status_phrase} {formatted_process_time}ms')
    return response


app = rx.App()
# app.add_page(pages.index, on_load=IndexState.get_songs)
app.api.add_api_route(path="/api/song", endpoint=api.add_song, methods=["POST"])
app.api.add_api_route(path="/api/auth/token", endpoint=api.get_token, methods=["POST"])
app.api.middleware("http")(log_request_middleware)
setup_logging()
