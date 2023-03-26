FROM debian:bullseye
RUN apt-get update
RUN apt-get install -y git autoconf automake libtool libtool-bin python2.7 libssl-dev gettext make wget

# Link to Python2 is required by .autogen.sh
RUN ln -s /usr/bin/python2.7 /usr/local/bin/python2

# Fucking autoconf is required by .autogen.sh
RUN wget http://ftp.gnu.org/gnu/autoconf/autoconf-2.71.tar.gz
RUN tar xzvf autoconf-2.71.tar.gz
WORKDIR autoconf-2.71
RUN ./configure && make && make install
WORKDIR /

RUN git clone https://github.com/cherokee/webserver.git
WORKDIR webserver
RUN ./autogen.sh
RUN make && make install
# WTF is this!!??? Cherokee won't run without this shit
RUN echo "/usr/local/lib/cherokee" > /etc/ld.so.conf.d/cherokee.conf
RUN ldconfig

COPY cherokee.conf /etc/cherokee.conf

EXPOSE 80 443 9090

CMD ["cherokee", "-C", "/etc/cherokee.conf"]
#CMD ["tail", "-f", "/dev/null"]
