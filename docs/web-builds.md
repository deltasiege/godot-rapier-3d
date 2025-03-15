## Guide

https://godot-rust.github.io/book/toolchain/export-web.html#building-both-with-and-without-multi-threading-support

## Commands

### Install rust nightly

### Install emscripten

git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install 3.1.62
./emsdk activate 3.1.62
source ./emsdk.sh (or ./emsdk.bat on windows)

### Build multi-threaded

RUSTFLAGS="-C link-args=-pthread" cargo +nightly build -Zbuild-std --target wasm32-unknown-emscripten

### Build single-threaded

mv target/debug/{YourCrate}.wasm target/debug/{YourCrate}.threads.wasm
