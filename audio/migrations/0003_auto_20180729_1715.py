# Generated by Django 2.0.4 on 2018-07-29 17:15

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('audio', '0002_auto_20180501_1520'),
    ]

    operations = [
        migrations.AlterField(
            model_name='song',
            name='title',
            field=models.CharField(max_length=128),
        ),
    ]
