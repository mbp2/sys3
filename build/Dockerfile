# gcc 13.2.0 x86_64-elf cross-compiler container
# with Rust + Cargo cross-compilation setup

FROM rust:latest
LABEL maintainer "Az33 <azyklus@disroot.org>"

ARG BINUTILS_VERSION=2.41
ARG GCC_VERSION=13.2.0

RUN echo "deb http://archive.debian.org/debian stretch main security contrib non-free" > /etc/apt/sources.list

RUN apt-get update
RUN apt-get upgrade -y

RUN apt-get install -y bison
RUN apt-get install -y build-essential
RUN apt-get install -y bzip2
RUN apt-get install -y curl
RUN apt-get install -y flex
RUN apt-get install -y gcc
RUN apt-get install -y grub-common
RUN apt-get install -y libgmp3-dev
RUN apt-get install -y libmpc-dev
RUN apt-get install -y libisl-dev
RUN apt-get install -y libmpfr-dev
RUN apt-get install -y make
RUN apt-get install -y nasm
RUN apt-get install -y patch
RUN apt-get install -y texinfo
RUN apt-get install -y wget
RUN apt-get install -y xorriso

# Pull binutils and GCC source code
RUN set -x && mkdir -p /usr/local/src
RUN set -x && cd /usr/local/src
RUN set -x && wget -q http://ftp.gnu.org/gnu/binutils/binutils-${BINUTILS_VERSION}.tar.gz
RUN set -x && wget -q http://ftp.gnu.org/gnu/gcc/gcc-${GCC_VERSION}/gcc-${GCC_VERSION}.tar.gz
RUN set -x && tar xzf binutils-${BINUTILS_VERSION}.tar.gz
RUN set -x && tar xzf gcc-${GCC_VERSION}.tar.gz
RUN set -x && rm binutils-${BINUTILS_VERSION}.tar.gz gcc-${GCC_VERSION}.tar.gz
RUN set -x && chown -R root:root binutils-${BINUTILS_VERSION}
RUN set -x && chown -R root:root gcc-${GCC_VERSION}
RUN set -x && chmod -R o-w,g+w binutils-${BINUTILS_VERSION}
RUN set -x && chmod -R o-w,g+w gcc-${GCC_VERSION}

# Copy compile and build scripts
COPY src /usr/local/src/

# Copy GCC patches
COPY gcc/t-x86_64-elf /usr/local/src/gcc-${GCC_VERSION}/gcc/config/i386/
COPY gcc/config.gcc.patch /usr/local/src/gcc-${GCC_VERSION}/gcc/

# Build and install binutils and the cross-compiler
RUN set -x \
	&& cd /usr/local/src \
	&& ./build-binutils.sh ${BINUTILS_VERSION} \
	&& ./build-gcc.sh ${GCC_VERSION}

RUN set -x \
	&& rustup toolchain install nightly

VOLUME /root/env
WORKDIR /root/env

CMD ["/bin/bash"]
