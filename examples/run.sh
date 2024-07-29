# build rucat server and rucat engine
cargo build --release
# run rucat server
RUST_LOG=info cargo run --release --bin rucat_server -- --config-path ./config.json
