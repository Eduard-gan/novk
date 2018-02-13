from django.contrib import admin
from .models import Song , Genre, Playlist


class PlaylistAdmin(admin.ModelAdmin):
    list_display = ('name', 'user', 'song', )
    list_filter = ('name', 'user',)

admin.site.register(Song)
admin.site.register(Genre)
admin.site.register(Playlist, PlaylistAdmin)