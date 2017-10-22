# -*- coding: utf-8 -*-
# Generated by Django 1.11.5 on 2017-10-07 11:24
from __future__ import unicode_literals

from django.db import migrations, models
import django.db.models.deletion


class Migration(migrations.Migration):

    dependencies = [
        ('audio', '0003_auto_20170924_1246'),
    ]

    operations = [
        migrations.CreateModel(
            name='TestSong',
            fields=[
                ('id', models.AutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('testtitle', models.CharField(max_length=30)),
                ('testartist', models.CharField(max_length=30)),
                ('testalbum', models.CharField(max_length=30)),
                ('testyear', models.IntegerField(blank=True, null=True)),
                ('testcomment', models.CharField(blank=True, max_length=28)),
                ('testtrack', models.IntegerField(blank=True, null=True)),
                ('testfile', models.FileField(upload_to='')),
                ('testtime', models.DateTimeField(auto_now_add=True)),
                ('testgenre', models.ForeignKey(default=1, on_delete=django.db.models.deletion.PROTECT, to='audio.Genre')),
            ],
        ),
    ]
