# gcc 8.2.0 x86_64-elf cross-compiler container
# with Rust + Cargo cross-compilation setup

FROM debian:stretch
LABEL maintainer "Az33 <azyklus@disroot.org>"

ARG BINUTILS_VERSION=2.31.0
ARG GCC_VERSION=8.2.0

RUN set -x \
   && apt update \
   && apt install -y wget gcc libgmp3-dev libmpfr-dev libisl-dev \
      libcloog-isl-dev libmpc-dev texinfo bison flex make bzip2 patch \
      build-essential

# Pull binutils and GCC source code
RUN set -x \
   && mkdir -p /usr/local/src \
   && cd /usr/local/src \
	&& wget -q https://ftp.gnu.org/gnu/binutils/binutils-${BINUTILS_VERSION}.tar.gz \
	&& wget -q https://ftp.gnu.org/gnu/gcc/gcc-${GCC_VERSION}/gcc-${GCC_VERSION}.tar.gz \
   && tar xzf binutils-${BINUTILS_VERSION}.tar.gz \
   && tar xzf gcc-${GCC_VERSION}.tar.gz \
   && rm binutils-${BINUTILS_VERSION}.tar.gz gcc-${GCC_VERSION}.tar.gz \
   && chown -R root:root binutils-${BINUTILS_VERSION} \
   && chown -R root:root gcc-${GCC_VERSION} \
   && chmod -R o-w,g+w binutils-${BINUTILS_VERSION} \
	&& chmod -R o-w,g+w gcc-${GCC_VERSION}

RUN set -x \
   && curl https://sh.rustup.rs -sSf \
   | sh -s -- --profile default --default-toolchain nightly

RUN set -x \
   && rustup component add rust-src \
   && rustup target add x86_64-unknown-none

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

RUN apt-get update 
RUN apt-get upgrade -y
RUN apt-get install -y nasm
RUN apt-get install -y xorriso
RUN apt-get install -y grub-pc-bin
RUN apt-get install -y grub-common

CMD ["/bin/bash"]
