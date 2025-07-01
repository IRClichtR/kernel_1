FROM ubuntu:22.04

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        gcc \
        libc6-dev \
        nasm \
        grub-pc-bin \
        grub-common \
        xorriso \
        binutils \
        make \
        curl \
        ca-certificates \
        gnupg && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly && \
    rustup component add rust-src --toolchain nightly

RUN rustup toolchain install nightly && \
    rustup default nightly && \
    rustup component add rust-src --toolchain nightly

WORKDIR /project

CMD ["/bin/bash"]