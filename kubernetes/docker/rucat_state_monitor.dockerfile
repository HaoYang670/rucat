FROM rust:1.82
WORKDIR /usr/src/rucat
COPY ./rucat_common ./rucat_common
COPY ./rucat_state_monitor ./rucat_state_monitor

# Install dependencies
RUN apt-get update && apt-get install protobuf-compiler -y

# Build rucat server
WORKDIR /usr/src/rucat/rucat_state_monitor
RUN cargo install --path .

WORKDIR /rucat
RUN rm -rf /usr/src/rucat
ENV RUST_LOG=debug
EXPOSE 3000