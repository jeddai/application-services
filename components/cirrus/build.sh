#!/bin/bash

rm -rf ./build

wasm-pack build -t bundler    -d build/bundler    --release
wasm-pack build -t web        -d build/web        --release
wasm-pack build -t no-modules -d build/no-modules --release
wasm-pack build -t nodejs     -d build/nodejs     --release
