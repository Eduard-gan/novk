# -*- coding: utf-8 -*-
# Generated by Django 1.11.5 on 2017-10-08 14:39
from __future__ import unicode_literals

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('audio', '0005_auto_20171007_1238'),
    ]

    operations = [
        migrations.AlterField(
            model_name='song',
            name='file',
            field=models.FileField(blank=True, upload_to=''),
        ),
    ]
