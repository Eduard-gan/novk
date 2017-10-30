from django.shortcuts import render
from django.http import HttpResponse
from .forms import *
from django.core.files.storage import FileSystemStorage
from mutagen import id3
import random
from django.conf import settings
import os
from django.contrib.auth.decorators import login_required
import magic
from django.core.files import File


def get_random_name():
    return "".join(
        [random.choice(list('123456789qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM')) for x in range(12)])


def convert_to_unicode(possible_cp1251_string):
    converted_unicode_string = ''
    for letter in possible_cp1251_string:
        temp = letter.encode('raw_unicode_escape')
        testnumber = int.from_bytes(temp, 'big')
        if testnumber > 127 < 256:
            converted_unicode_string += ((temp.decode('cp1251')).encode('UTF-8')).decode()
        else:
            return possible_cp1251_string
    return converted_unicode_string


@login_required
def music(request):
    songs = Song.objects.all().values()
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
            mime = magic.Magic(mime=True)
            filetype = mime.from_file(fs.path(filename))
            if filetype == 'audio/mpeg':
                # Попытка считывания и передача тэгов в форму
                # для передачи пользователю где он вносит правки и подтверждает.
                try:
                    tags = id3.ID3(fs.path(filename))
                    initial = {'artist': tags.get('TPE1', [''])[0].title(),
                               'title': tags.get('TIT2', [''])[0].title(),
                               'album': tags.get('TALB', [''])[0],
                               'year': tags.get('TDRC', [''])[0],
                               'track': tags.get('TIT2', [''])[0],
                               'genre': '1'
                               }
                    form = SongCommit(initial)
                except(id3.ID3NoHeaderError):
                    form = SongCommit()
                    return render(request, 'upload.html',
                                  {'form': form, 'uploaded_file_url': request.session.get('TempFileURL')})
            else:
                return HttpResponse('No processing code for mime type {}'.format(filetype))
            return render(request, 'upload.html',
                          {'form': form, 'uploaded_file_url': request.session.get('TempFileURL')})
        elif request.FILES.get('file') is None:
            # Связываем принятые данные с новой формой для валидации
            form = SongCommit(request.POST)
            if form.is_valid():
                model = form.save(commit=False)
                # Обновление тэгов файла
                tags = id3.ID3(request.session.get('TempFilePath'))
                tags.update_to_v23()
                tags.add(id3.TPE1(encoding=3, text=form.cleaned_data['artist']))
                tags.add(id3.TIT2(encoding=3, text=form.cleaned_data['title']))
                tags.add(id3.TALB(encoding=3, text=form.cleaned_data['album']))
                tags.add(id3.TDRC(encoding=3, text=str(form.cleaned_data['album'])))
                tags.add(id3.TRCK(encoding=3, text=str(form.cleaned_data['track'])))
                tags.add(id3.TCON(encoding=3, text=str(form.cleaned_data['genre'])))
                tags.save()
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
            # Удаление временного файла
            os.remove(request.session.get('TempFilePath'))
            return HttpResponse('<p>All Data Saved.</p>')
    return HttpResponse(
        "<p>No actions found in views.upload for request's HTTP-method {}</p>".format(
            request.method))
