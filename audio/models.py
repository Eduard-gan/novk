from django.db import models

DEFAULT_GENRE_ID = 1


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
    genre = models.ForeignKey(Genre, on_delete=models.PROTECT, default=DEFAULT_GENRE_ID)
    file = models.FileField(upload_to='audio')

    def __str__(self):
        return self.artist + ' - ' + self.title
