FROM ubuntu:latest
USER STAN
ADD target/release/learn-rust /http_server
WORKDIR /
ENTRYPOINT ["/http_server"]