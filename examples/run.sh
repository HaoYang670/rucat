# build rucat server
cargo build --release
# run rucat server
RUST_LOG=info cargo run --release --bin rucat_server -- --config-path ./config.json
