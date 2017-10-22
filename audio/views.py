from django.shortcuts import render
from django.http import HttpResponse
from .models import Song 
from .forms import *
from django.core.files.storage import FileSystemStorage
from mutagen import id3
import random
from django.conf import settings
import os
from django.contrib.auth.decorators import login_required
import magic
from django.core.files import File
import datetime

def _getRandomName_():
    return "".join([random.choice(list('123456789qwertyuiopasdfghjklzxcvbnmQWERTYUIOPASDFGHJKLZXCVBNM')) for x in range(12)])

def _convertToUnicode_(PossibleCP1251String):
    convertedUnicodeString = ''
    for letter in PossibleCP1251String:
        temp = letter.encode('raw_unicode_escape')
        testnumber = int.from_bytes(temp,'big')
        if testnumber > 127 and testnumber < 256:
            convertedUnicodeString += ((temp.decode('cp1251')).encode('UTF-8')).decode()
        else:
            return(PossibleCP1251String)
    return(convertedUnicodeString)







@login_required
def music(request):
    songs = Song.objects.all().values()
    context = {}
    context.update({'songs':songs})
    return render(request,'music.html',context)

def upload(request):
    if request.method == 'GET':
        form = Upload()
        return render(request, 'upload.html', {'form': form})
    if request.method == 'POST':
        if request.FILES.get('file'):
            # Сохранение файла во временной папке и отдача в сессию сведений о временном файле
            fs = FileSystemStorage( location=settings.MEDIA_ROOT + '/temp/' , 
                                    base_url=settings.MEDIA_URL + 'temp/')
            filename = fs.save(_getRandomName_(), request.FILES['file'])
            request.session['TempFileName'] = filename
            request.session['TempFileURL'] = fs.url(filename)
            request.session['TempFilePath'] = fs.path(filename)
            # Анализ типа файла
            mime = magic.Magic(mime=True)
            FileType = mime.from_file(fs.path(filename))
            if FileType == 'audio/mpeg':
                # Считывание и передача тэгов в форму для правки и подтверждения
                tags = id3.ID3(fs.path(filename))
                form = SongCommit( initial = {  'artist': _convertToUnicode_(tags['TPE1'][0]).title(),
                                                'title': _convertToUnicode_(tags['TIT2'][0]).title(),
                                                'album': _convertToUnicode_(tags['TALB'][0]).title(),
                                                'year': tags['TDRC'][0],
                                                'track': tags['TRCK'][0]}
                                  )
            else:
                return HttpResponse('No processing code for mime type {}'.format(FileType))
            return render(request, 'upload.html', {'form': form , 'uploaded_file_url': request.session.get('TempFileURL')} )
        elif request.FILES.get('file') == None:
            # Связываем принятые данные с новой формой для валидации
            form = SongCommit(request.POST)
            if form.is_valid():
                model = form.save(commit=False)
                # Обновление тэгов файла
                tags = id3.ID3(request.session.get('TempFilePath'))
                tags.update_to_v23()
                tags.add(id3.TPE1(encoding=3 , text=form.cleaned_data['artist']))
                tags.add(id3.TIT2(encoding=3 , text=form.cleaned_data['title']))
                tags.add(id3.TALB(encoding=3 , text=form.cleaned_data['album']))
                tags.add(id3.TDRC(encoding=3 , text=str(form.cleaned_data['album'])))
                tags.add(id3.TRCK(encoding=3 , text=str(form.cleaned_data['track'])))
                tags.add(id3.TCON(encoding=3 , text=str(form.cleaned_data['genre'])))
                tags.save()
            # Создание файлового объекта Джанго из временного файла для последующего крепления в модель.
            with open(request.session.get('TempFilePath'), 'rb') as TempFile:
                DjangFileObject = File(TempFile)
                model.file = DjangFileObject
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
    return HttpResponse("<p>No actions found in views.upload for request's HTTP-method {}   OR FORM IS NOT VALID(NO FALURE PROCESSING CODE YET WRITTEN)</p>".format(request.method))
