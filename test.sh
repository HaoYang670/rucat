# There is no way to run graceful shutdown in tests, which cause engines and databases processes leak.
# Use this script to clean the leaked things

cargo build
cargo test
pkill surreal
pkill rucat_engine
