FROM alpine

ARG TAG

RUN wget -o /usr/local/bin/ccpmbuild "https://github.com/Deleranax/ccpmbuild/releases/download/$TAG/ccpmbuild-x86_64-unknown-linux-gnu"

ENTRYPOINT [ "ccpmbuild" ]