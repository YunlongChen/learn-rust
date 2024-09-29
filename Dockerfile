FROM ubuntu:latest
USER STAN
ADD target/release/learn-rust /http_server

ENTRYPOINT ["/http_server"]