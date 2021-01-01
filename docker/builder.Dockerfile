FROM ubuntu:20.04

# VERSION 0.1.2
# Rust stable version 1.49

RUN apt-get update && \
    apt-get install -y build-essential python3 curl git && \
    apt-get clean

RUN curl -sSf -o /opt/rustup.sh https://sh.rustup.rs && \
    echo "fa50ccf79c30ce9446cc45917e8ea10655674c2a9509221cb12bd865c60ab709  /opt/rustup.sh" | sha256sum -c && \
    chmod +x /opt/rustup.sh && \
    /opt/rustup.sh -y && \
    rm /opt/rustup.sh && \
    mv /root/.cargo/bin/* /usr/bin
