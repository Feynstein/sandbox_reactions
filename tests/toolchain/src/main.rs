// The one-file program the `toolchain` scope builds: native it prints the answer; for wasm32 it
// touches wasm-bindgen so trunk has glue to generate.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("sr-toolchain {}", 6 * 7);
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let answer = wasm_bindgen::JsValue::from_f64(42.0);
    let _ = answer;
}
