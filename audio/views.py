from django.shortcuts import render
from django.http import HttpResponse
from .forms import *
from django.core.files.storage import FileSystemStorage
from mutagen import id3
import random
from django.conf import settings
import os
from django.contrib.auth.decorators import login_required
from django.core.files import File
from django.shortcuts import HttpResponseRedirect
from mutagen.id3._frames import TPE1, TIT2, TALB, TDRC, TRCK, TCON
import binascii
from audio.models import Playlist


def get_filetype(file):

    signatures = {'mp3 with id3': b'4944',
                  'mp3 wthout id3': b'fffb'
                  }
    filetype = 'Not supported filetype'

    with open(file, 'rb') as data:
        byte = data.read(1)
        if byte != b'\x00':
            data.seek(0)
        else:
            while byte == b'\x00':
                byte = data.read(1)
            data.seek((data.tell()-1))
        chunk = data.read(3)

        for sig in signatures.items():
            if sig[1] in binascii.hexlify(chunk):
                filetype = sig[0]
    return filetype


def get_random_name():
    return "".join(
        [random.choice(list('123456789qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM')) for _ in range(12)]
    )


def convert_to_unicode(possible_cp1251_string):
    converted_unicode_string = ''
    for letter in possible_cp1251_string:
        temp = letter.encode('raw_unicode_escape')
        testnumber = int.from_bytes(temp, 'big')
        if testnumber > 127 < 256:
            converted_unicode_string += ((temp.decode('cp1251')).encode('UTF-8')).decode()
        else:
            converted_unicode_string += temp.decode()
    return converted_unicode_string


@login_required
def music(request):
    songs = Song.objects.filter(playlist__user=request.user, playlist__number=0).values()
    context = {}
    context.update({'songs': songs})
    return render(request, 'music.html', context)


def upload(request):
    if request.method == 'GET':
        form = Upload()
        return render(request, 'upload.html', {'form': form})
    if request.method == 'POST':
        if request.FILES.get('file'):
            # Сохранение файла во временной папке и отдача в сессию сведений о временном файле
            fs = FileSystemStorage(location=settings.MEDIA_ROOT + '/temp/',
                                   base_url=settings.MEDIA_URL + 'temp/')
            filename = fs.save(get_random_name(), request.FILES['file'])
            request.session['TempFileName'] = filename
            request.session['TempFileURL'] = fs.url(filename)
            request.session['TempFilePath'] = fs.path(filename)

            # Анализ типа файла
            filetype = get_filetype(fs.path(filename))
            if filetype != 'Not supported filetype':
                # Попытка считывания и передача тэгов в форму
                # для передачи пользователю где он вносит правки и подтверждает.
                try:
                    tags = id3.ID3(fs.path(filename))
                    initial = {
                        'artist': convert_to_unicode(tags.get('TPE1', [''])[0]).title(),
                        'title': convert_to_unicode(tags.get('TIT2', [''])[0]).title(),
                        'album': convert_to_unicode(tags.get('TALB', [''])[0]),
                        'year': tags.get('TDRC', [''])[0],
                        'track': tags.get('TIT2', [''])[0],
                        'genre': '1'
                    }
                    form = SongCommit(initial)
                except id3.ID3NoHeaderError:
                    form = SongCommit()
                    return render(request, 'upload.html',
                                  {'form': form, 'uploaded_file_url': request.session.get('TempFileURL')})
            else:
                os.remove(request.session.get('TempFilePath'))
                return HttpResponse('No processing code for mime type {}'.format(filetype))
            return render(request, 'upload.html',
                          {'form': form, 'uploaded_file_url': request.session.get('TempFileURL')})
        elif request.FILES.get('file') is None:
            # Связываем принятые данные с новой формой для валидации
            form = SongCommit(request.POST)
            if form.is_valid():
                model = form.save(commit=False)
                try:  # Обновление тэгов файла
                    tags = id3.ID3(request.session.get('TempFilePath'))
                except id3.ID3NoHeaderError:
                    tags = id3.ID3()
                tags.update_to_v23()
                tags.add(TPE1(encoding=3, text=form.cleaned_data['artist']))
                tags.add(TIT2(encoding=3, text=form.cleaned_data['title']))
                tags.add(TALB(encoding=3, text=form.cleaned_data['album']))
                tags.add(TDRC(encoding=3, text=str(form.cleaned_data['album'])))
                tags.add(TRCK(encoding=3, text=str(form.cleaned_data['track'])))
                tags.add(TCON(encoding=3, text=str(form.cleaned_data['genre'])))
                tags.save(request.session.get('TempFilePath')) if not tags.filename else tags.save()
            # Создание файлового объекта Джанго из временного файла для последующего крепления в модель.
                with open(request.session.get('TempFilePath'), 'rb') as TempFile:
                    django_file_object = File(TempFile)
                    model.file = django_file_object
                    model.file.name = request.session.get('TempFileName') + '.mp3'
                    model.artist = form.cleaned_data['artist']
                    model.title = form.cleaned_data['title']
                    model.album = form.cleaned_data['album']
                    model.year = form.cleaned_data['year']
                    model.track = form.cleaned_data['track']
                    model.genre = form.cleaned_data['genre']
                    model.save()
                    # Добавление в дефолтный плэйлист
                    Playlist.objects.create(user=request.user, number=0, song=model)
                # Удаление временного файла
                os.remove(request.session.get('TempFilePath'))
            else:   # В случае невалидной формы показываем её ещё
                return render(request, 'upload.html', {'form': form})
        return HttpResponseRedirect('/music/')
    return HttpResponse(
        "<p>No actions found in views.upload for request's HTTP-method {}</p>".format(
            request.method))
