from django.urls import re_path

from audio.views import music, upload, CreatePlaylist
from audio.ajax import playlist_handler
urlpatterns = [
    re_path(r'^upload/+$', upload, name='upload'),
    re_path(r'^create-playlist/$', CreatePlaylist.as_view(), name='create-playlist'),
    re_path(r'^$', music, name='music'),
    re_path(r'^ajax/playlist_operations/$', playlist_handler),
]
