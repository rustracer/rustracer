Heavily based on top of : https://wasmbyexample.dev

# Features
- [x] Builds with wasm-pack
- [x] Uses raytracer_core to generate an image on a canvas
- [ ] TRY: Do not impact raytracer core that much by enabling wasm-bindgen feature for rand crate FROM wasm project
- [ ] Build script copies every necessary files into a neat package so it can be served as-is: `pkg/` + `index.(js|html)`
- [ ] ? Build script copies debug information only if necessary ?