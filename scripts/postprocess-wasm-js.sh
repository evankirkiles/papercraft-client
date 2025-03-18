#!/usr/bin/sh

# Adds `?no-inline` to the `.wasm` import, preventing the WASM file from being
# inlined as a base64 data URL. A Vite 6 thing, I think.
#  - https://github.com/vitejs/vite/issues/4454#issuecomment-2596153539
sed -i.bak "s|\"./index_bg.wasm\"|\"./index_bg.wasm?no-inline\"|" ./src/rs/index.js
rm ./src/rs/index.js.bak
