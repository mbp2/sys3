FROM randomdude/gcc-cross-x86_64-elf

RUN echo "deb http://archive.debian.org/debian stretch main contrib non-free" > /etc/apt/sources.list

RUN apt update
RUN apt upgrade -y
RUN apt install -y nasm
RUN apt install -y xorriso
RUN apt install -y grub-pc-bin
RUN apt install -y grub-common

VOLUME /root/env
WORKDIR /root/env
