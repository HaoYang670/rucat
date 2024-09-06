FROM rust:1.81
WORKDIR /usr/src/rucat
COPY ./rucat_common ./rucat_common
COPY ./rucat_server ./rucat_server

# Install dependencies
RUN apt-get update && apt-get install protobuf-compiler -y

# Build rucat server
WORKDIR /usr/src/rucat/rucat_server
RUN cargo install --path .
ENV RUST_LOG=info
CMD ["cargo run --release"]