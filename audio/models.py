from django.db import models
from django.contrib.auth.models import User

class Genre(models.Model):
    name = models.CharField(max_length=32)

    def __str__(self):
        return self.name


class Song(models.Model):
    title = models.CharField(max_length=64)
    artist = models.CharField(max_length=64)
    album = models.CharField(max_length=64, blank=True)
    year = models.IntegerField(null=True, blank=True)
    comment = models.CharField(max_length=32, blank=True)
    track = models.IntegerField(null=True, blank=True)
    genre = models.ForeignKey(Genre, on_delete=models.PROTECT, blank=True, null=True)
    file = models.FileField(upload_to='audio')

    def __str__(self):
        return self.artist + ' - ' + self.title


class PlaylistQuerySet(models.QuerySet):
    def get_highest_playlist_number(self):
        numbers = self.all().values('number').distinct()
        highest = 0
        for n in numbers:
            if n['number'] > highest:
                highest = n['number']
        return highest

class Playlist(models.Model):
    name = models.CharField(max_length=32, default='Медиатека')
    number = models.IntegerField()
    user = models.ForeignKey(User)
    song = models.ForeignKey(Song, null=True, blank=True)

    objects = PlaylistQuerySet.as_manager()

    def __str__(self):
        return self.name