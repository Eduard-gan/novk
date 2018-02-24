from django.db import models
from django.contrib.auth.models import User

from audio.models import Song
from pictures.models import Picture


class DiaryPage(models.Model):
    owner = models.ForeignKey(User)
    created = models.DateTimeField(auto_now_add=True)
    text = models.CharField(max_length=4096, blank=True)
    pictures = models.ManyToManyField(Picture, blank=True)
    songs = models.ManyToManyField(Song, blank=True)

    def __str__(self):
        return '{0} - {1}'.format(self.owner, self.created.date())
