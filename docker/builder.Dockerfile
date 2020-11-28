FROM ubuntu:20.04

# VERSION 0.1.1

RUN apt-get update && \
    apt-get install -y build-essential python3 curl git && \
    apt-get clean

RUN curl -sSf -o /opt/rustup.sh https://sh.rustup.rs && \
    echo "7c516cc8c47de454d08556e22d0e97cef386faaa133175266dd59679c7a9f75d  /opt/rustup.sh" | sha256sum -c && \
    chmod +x /opt/rustup.sh && \
    /opt/rustup.sh -y && \
    rm /opt/rustup.sh && \
    mv /root/.cargo/bin/* /usr/bin
