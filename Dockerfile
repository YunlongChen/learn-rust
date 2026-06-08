FROM ubuntu:latest
USER STAN
ADD target/release/netops /http_server
WORKDIR /
ENTRYPOINT ["/http_server "]