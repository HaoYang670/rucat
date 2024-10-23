# There is no way to run graceful shutdown in tests, which cause engines and databases processes leak.
# Use this script to clean the leaked things

cargo test
# kill all the leaked processes
pkill surreal
