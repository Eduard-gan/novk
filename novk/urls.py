
from django.conf.urls import url
from django.contrib import admin
from django.conf.urls import include
from django.contrib.auth import views as auth_views
from django.views.generic.base import RedirectView
from django.urls import reverse_lazy
from novk import views as core_views 

import debug_toolbar

urlpatterns = [
    url(r'^login/', auth_views.LoginView.as_view() , name='login'),
    url(r'^logout/', auth_views.LogoutView.as_view(next_page=reverse_lazy('login')), name='logout'),
    url(r'^signup/', core_views.signup, name='signup'),
    url(r'^admin/', admin.site.urls),
    url(r'^music/', include('audio.urls')),
    url(r'^__debug__/', include(debug_toolbar.urls)),
    url(r'^$', RedirectView.as_view(url='login/', permanent=False), name='index')
]