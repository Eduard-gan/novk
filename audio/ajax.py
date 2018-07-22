from django.contrib.auth.decorators import login_required
from django.http import HttpResponse


from audio.models import Playlist, Song


@login_required
def playlist_handler(request):

    if str(request.POST['operation_type']) == 'add':

        playlist = Playlist.objects.filter(number=int(request.POST['playlist_id']),
                                           user=request.user)
        result = Playlist.objects.get_or_create(user=request.user,
                                                number=int(request.POST['playlist_id']),
                                                song=Song.objects.get(id=int(request.POST['song_id'])),
                                                name=playlist[0].name)
        if result[1]:
            return(HttpResponse('Песня добавлена в плэйлист'))
        else:
            return (HttpResponse('Эта песня уже есть в этом плэйлисте'))
