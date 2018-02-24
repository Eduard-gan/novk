from django.db import models


def upload_destination(instance, filename):
    from uuid import uuid4

    return 'audio/{}'.format(uuid4())


class Picture(models.Model):
    title = models.CharField(max_length=128)
    image = models.ImageField(upload_to=upload_destination)
    created = models.DateTimeField(auto_now_add=True)
    description = models.CharField(max_length=4096, blank=True)

    def __str__(self):
        return '{1} - {0}'.format(self.title, self.created.date())
