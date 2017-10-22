from django import forms

from .models import Song


class Upload(forms.Form):
    file = forms.FileField()

class SongCommit(forms.ModelForm):
    class Meta:
        model = Song
        fields = [  'title',
                    'artist',
                    'album',
                    'year',
                    'comment',
                    'track',
                    'genre',
        ]

class SongCommit2(forms.ModelForm):
    class Meta:
        model = Song
        fields = '__all__'
