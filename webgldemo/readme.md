WebGL Demo
==========

Usage
-----

To run the WebGL demo, you need to have a web server set up. For example Live Server in Visual Studio Code or any other web server of your choice.

To build the sample

```bash
cargo build --package shadegldemo --target=wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/debug/shadegldemo.wasm webgldemo/html/
cargo wasm2map webgldemo/html/shadegldemo.wasm --patch --base-url http://127.0.0.1:5500/webgldemo/html
```
