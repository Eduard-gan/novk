from django.conf import settings
from django.urls import re_path
from django.conf.urls.static import static
from django.contrib import admin
from django.conf.urls import include
from django.contrib.auth import views as auth_views
from django.views.generic.base import RedirectView
from django.urls import reverse_lazy

import debug_toolbar

from novk import views as core_views

urlpatterns = [
    re_path(r'^login/', auth_views.LoginView.as_view() , name='login'),
    re_path(r'^logout/', auth_views.LogoutView.as_view(next_page=reverse_lazy('login')), name='logout'),
    re_path(r'^signup/', core_views.signup, name='signup'),
    re_path(r'^admin/', admin.site.urls),
    re_path(r'^music/', include('audio.urls')),
    re_path(r'^__debug__/', include(debug_toolbar.urls)),
    re_path(r'^$', RedirectView.as_view(url='music/', permanent=False)),
]

urlpatterns += static(settings.STATIC_URL, document_root=settings.STATIC_ROOT)
urlpatterns += static(settings.MEDIA_URL, document_root=settings.MEDIA_ROOT)
