FROM rust:1.81
WORKDIR /usr/src/rucat
COPY ./rucat_common ./rucat_common
COPY ./rucat_server ./rucat_server

# Install dependencies
RUN apt-get update && apt-get install protobuf-compiler -y

# Install surreal. This is used for embedded database, only for testing.
RUN curl -sSf https://install.surrealdb.com | sh

# Build rucat server
WORKDIR /usr/src/rucat/rucat_server
RUN cargo install --path .

WORKDIR /rucat
RUN rm -rf /usr/src/rucat
ENV RUST_LOG=debug
EXPOSE 3000