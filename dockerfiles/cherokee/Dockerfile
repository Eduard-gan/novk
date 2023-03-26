FROM ubuntu
RUN apt-get update && apt-get -qq install git autoconf automake libtool libtool-bin python2 libssl-dev gettext make
RUN git clone https://github.com/cherokee/webserver.git
WORKDIR webserver
RUN ./autogen.sh --disable-dependency-tracking
RUN make && make install
RUN echo "/usr/local/lib/cherokee" > /etc/ld.so.conf.d/cherokee.conf
RUN ldconfig

COPY cherokee.conf /etc/cherokee.conf

EXPOSE 80 443 9090

CMD ["cherokee", "-C", "/etc/cherokee.conf"]
#CMD ["tail", "-f", "/dev/null"]
