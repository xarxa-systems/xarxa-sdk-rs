alias gen := generate

[doc('Build plugin')]
build name:
    cargo build --example {{ name }} --target wasm32-wasip2 --release

clean:
    cargo clean

# Generate wit bindings
generate:
    wit-bindgen rust --out-dir src/bindings --world orchestrator wit/orchestrator --pub-export-macro --disable-custom-section-link-helpers