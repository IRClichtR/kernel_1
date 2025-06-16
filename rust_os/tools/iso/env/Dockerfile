FROM randomdude/gcc-cross-x86_64-elf

RUN apt-get update 
RUN apt-get upgrade -y
# Dependencies
RUN apt-get install -y nasm grub-pc-bin grub-common xorriso

VOLUME /root/env

WORKDIR /root/env