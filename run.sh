# build rucat server and rucat engine
cargo build --release
# run rucat server
RUST_LOG=debug cargo run --release --bin rucat_server -- --engine-binary-path target/release/rucat_engine