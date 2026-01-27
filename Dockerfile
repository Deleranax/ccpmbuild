FROM alpine

# Version tag
ARG VERSION

# Download the dependencies
RUN apk add gcompat libgcc

# Install the program
RUN wget -O ccpmbuild "https://github.com/Deleranax/ccpmbuild/releases/download/$VERSION/ccpmbuild-x86_64-unknown-linux-gnu" \
    && mv ccpmbuild /usr/local/bin/ccpmbuild \
    && chmod +x /usr/local/bin/ccpmbuild

ENTRYPOINT [ "/usr/local/bin/ccpmbuild" ]