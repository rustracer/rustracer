Heavily based on top of : https://wasmbyexample.dev

# How To
To serve via `../web` project : `wasm-pack build --target web --release`

# Features
- [x] Builds with [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [x] Uses raytracer_core to generate an image on a canvas
- [x] Different width from height
- [x] Have only 1 place where width and height are set: currently hardcoded in the Wasm library.
- [x] Clean random dependancy without 2 mutex locks
- [ ] Build script copies every necessary files into a neat package so it can be served as-is: `pkg/` + `index.(js|html)`
- [ ] ? Build script copies debug information only if necessary ?