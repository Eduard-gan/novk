from django.conf.urls import url

from audio.views import music, upload, CreatePlaylist
from audio.ajax import playlist_handler
urlpatterns = [
    url(r'^upload/+$', upload, name='upload'),
    url(r'^create-playlist/$', CreatePlaylist.as_view(), name='create-playlist'),
    url(r'^$', music, name='music'),
    url(r'^ajax/playlist_operations/$', playlist_handler),
]
