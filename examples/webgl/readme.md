WebGL Demo
==========

Live demos are available here:

<https://casualhacks.net/shade/>

Usage
-----

To run the WebGL demo, you need to have a web server set up. For example Live Server or Show Preview in Visual Studio Code or any other web server of your choice.

Build the single WebGL wasm module and copy it into the HTML folder with one of these scripts:

```
./build.sh
build.bat
```

Both scripts run a release build for the `webgl` package and copy `target/wasm32-unknown-unknown/release/webgl.wasm` to `examples/webgl/html/webgl.wasm`.
