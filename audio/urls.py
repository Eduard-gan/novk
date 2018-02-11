from django.conf.urls import url

from audio.views import music, upload, CreatePlaylist

urlpatterns = [
    url(r'^upload/+$', upload, name='upload'),
    url(r'^create-playlist/$', CreatePlaylist.as_view(), name='create-playlist'),
    url(r'^$', music, name='music')
]
