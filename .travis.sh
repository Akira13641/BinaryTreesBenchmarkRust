set -e
cargo build --release
cd ./target/release
time ./binarytrees_benchmark 21
