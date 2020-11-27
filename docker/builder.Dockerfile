FROM ubuntu:20.04

RUN apt-get update && \
    apt-get install -y python3 curl git && \
    apt-get clean

RUN curl -sSf -o /opt/rustup.sh https://sh.rustup.rs && \
    echo "8928261388c8fae83bfd79b08d9030dfe21d17a8b59e9dcabda779213f6a3d14  /opt/rustup.sh" | sha256sum -c && \
    chmod +x /opt/rustup.sh && \
    /opt/rustup.sh -y && \
    rm /opt/rustup.sh

ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /root