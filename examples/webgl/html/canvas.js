let MODULE_NAME = null; {
	let p = new URLSearchParams(window.location.search);
	MODULE_NAME = p.get("module");
}

class HandleTable {
	constructor() {
		this.handles = {
			0: null,
		};
		this.nextHandle = 1;
	}

	add(obj) {
		const handle = this.nextHandle++;
		this.handles[handle] = obj;
		return handle;
	}

	get(handle) {
		if (!this.has(handle)) {
			throw new Error(`Resource with handle ${handle} not found.`);
		}
		return this.handles[handle];
	}

	remove(handle) {
		let value = this.get(handle);
		delete this.handles[handle];
		return value;
	}
	has(handle) {
		return this.handles.hasOwnProperty(handle);
	}
}

class InputController {
	constructor(canvas) {
		this.canvas = canvas;
		this.isMouseDown = false;
		this.lastX = 0;
		this.lastY = 0;
		
		// Accumulated deltas (reset after consumption)
		this.deltaX = 0;
		this.deltaY = 0;
		this.deltaZoom = 0;
		this.needsUpdate = false;
		
		this.boundHandlers = {};
		this.setupEventListeners();
	}
	
	setupEventListeners() {
		// Use pointer events for unified mouse/touch handling
		this.boundHandlers.pointerdown = (e) => this.onPointerDown(e);
		this.boundHandlers.pointermove = (e) => this.onPointerMove(e);
		this.boundHandlers.pointerup = (e) => this.onPointerUp(e);
		this.boundHandlers.wheel = (e) => this.onWheel(e);
		this.boundHandlers.keydown = (e) => this.onKeyDown(e);
		this.boundHandlers.contextlost = (e) => this.handleContextLost(e);
		
		this.canvas.addEventListener('pointerdown', this.boundHandlers.pointerdown);
		this.canvas.addEventListener('pointermove', this.boundHandlers.pointermove);
		this.canvas.addEventListener('pointerup', this.boundHandlers.pointerup);
		this.canvas.addEventListener('wheel', this.boundHandlers.wheel, { passive: false });
		window.addEventListener('keydown', this.boundHandlers.keydown);
		this.canvas.addEventListener('webglcontextlost', this.boundHandlers.contextlost);
	}
	
	onPointerDown(e) {
		this.isMouseDown = true;
		this.lastX = e.clientX;
		this.lastY = e.clientY;
		this.canvas.style.cursor = 'grabbing';
		this.canvas.setPointerCapture(e.pointerId);
	}
	
	onPointerMove(e) {
		if (!this.isMouseDown) return;
		
		// Use movementX/Y for resolution-independent deltas
		this.deltaX += e.movementX;
		this.deltaY += e.movementY;
		this.needsUpdate = true;
	}
	
	onPointerUp(e) {
		this.isMouseDown = false;
		this.canvas.style.cursor = 'grab';
		this.canvas.releasePointerCapture(e.pointerId);
	}
	
	onWheel(e) {
		e.preventDefault();
		this.deltaZoom += e.deltaY;
		this.needsUpdate = true;
	}
	
	onKeyDown(e) {
		if (e.key === 'r' || e.key === 'R') {
			// Signal reset
			this.deltaX = 0;
			this.deltaY = 0;
			this.deltaZoom = -9999; // Special value for reset
			this.needsUpdate = true;
		}
	}
	
	handleContextLost(e) {
		console.warn('WebGL context lost');
		e.preventDefault();
	}
	
	consumeDeltas() {
		if (!this.needsUpdate) return null;
		
		const deltas = {
			x: this.deltaX,
			y: this.deltaY,
			zoom: this.deltaZoom
		};
		
		// Reset accumulated values
		this.deltaX = 0;
		this.deltaY = 0;
		this.deltaZoom = 0;
		this.needsUpdate = false;
		
		return deltas;
	}
	
	dispose() {
		// Clean up event listeners
		this.canvas.removeEventListener('pointerdown', this.boundHandlers.pointerdown);
		this.canvas.removeEventListener('pointermove', this.boundHandlers.pointermove);
		this.canvas.removeEventListener('pointerup', this.boundHandlers.pointerup);
		this.canvas.removeEventListener('wheel', this.boundHandlers.wheel);
		window.removeEventListener('keydown', this.boundHandlers.keydown);
		this.canvas.removeEventListener('webglcontextlost', this.boundHandlers.contextlost);
	}
}

document.addEventListener("DOMContentLoaded", () => {
	if (!MODULE_NAME) {
		alert("No module specified. Please provide a module name in the URL hash, e.g., #module=your_module.wasm");
		return;
	}

	const canvas = document.getElementById('canvas');
	if (!canvas) {
		alert("Canvas element not found.");
		return;
	}
	const gl = canvas.getContext('webgl');
	if (!gl) {
		alert("WebGL is not supported by your browser.");
		return;
	}
	window.gl = gl;

	let updatesize = null;
	function resizeCanvas() {
		canvas.width = window.innerWidth;
		canvas.height = window.innerHeight;
		if (updatesize) {
			updatesize(canvas.width, canvas.height);
		}
	}
	window.addEventListener('resize', resizeCanvas);
	resizeCanvas();

	let wasmInstance;
	let drawFn;

	async function loadWasm() {
		try {
			let handles = new HandleTable();
			const imports = {
				webgl: {
					consoleLog(msg_ptr, msg_len) {
						const msg = new TextDecoder().decode(new Uint8Array(memory.buffer, msg_ptr, msg_len));
						console.log(msg);
					},
					enable(cap) { gl.enable(cap); },
					disable(cap) { gl.disable(cap); },
					scissor(x, y, width, height) { gl.scissor(x, y, width, height); },
					blendFunc(sfactor, dfactor) { gl.blendFunc(sfactor, dfactor); },
					blendEquation(mode) { gl.blendEquation(mode); },
					depthFunc(func) { gl.depthFunc(func); },
					cullFace(mode) { gl.cullFace(mode); },
					clearColor(r, g, b, a) { gl.clearColor(r, g, b, a); },
					clearDepth(depth) { gl.clearDepth(depth); },
					clearStencil(s) { gl.clearStencil(s); },
					clear(mask) { gl.clear(mask); },
					colorMask(r, g, b, a) { gl.colorMask(r, g, b, a); },
					depthMask(flag) { gl.depthMask(flag); },
					stencilMask(mask) { gl.stencilMask(mask); },
					viewport(x, y, width, height) { gl.viewport(x, y, width, height); },
					createBuffer() { return handles.add(gl.createBuffer()); },
					bindBuffer(target, buffer) { gl.bindBuffer(target, handles.get(buffer)); },
					bufferData(target, size, data_ptr, usage) {
						const data = new Uint8Array(memory.buffer, data_ptr, size);
						gl.bufferData(target, data, usage);
					},
					deleteBuffer(buffer) {
						gl.deleteBuffer(handles.get(buffer));
						handles.remove(buffer);
					},
					getAttribLocation(program, name_ptr, name_len) {
						const name = new TextDecoder().decode(new Uint8Array(memory.buffer, name_ptr, name_len));
						return gl.getAttribLocation(handles.get(program), name);
					},
					enableVertexAttribArray(index) { gl.enableVertexAttribArray(index); },
					disableVertexAttribArray(index) { gl.disableVertexAttribArray(index); },
					vertexAttribPointer(index, size, type, normalized, stride, offset) {
						gl.vertexAttribPointer(index, size, type, normalized, stride, offset);
					},
					createProgram() { return handles.add(gl.createProgram()); },
					deleteProgram(program) { let value = handles.remove(program); gl.deleteProgram(value); },
					createShader(type) { return handles.add(gl.createShader(type)); },
					deleteShader(shader) { let value = handles.remove(shader); gl.deleteShader(value); },
					shaderSource(shader, source_ptr, source_len) {
						const source = new TextDecoder().decode(new Uint8Array(memory.buffer, source_ptr, source_len));
						gl.shaderSource(handles.get(shader), source);
					},
					compileShader(shader) { gl.compileShader(handles.get(shader)); },
					getShaderParameter(shader, pname) { return gl.getShaderParameter(handles.get(shader), pname); },
					getShaderInfoLog(shader) {
						const log = gl.getShaderInfoLog(handles.get(shader));
						console.error(`Shader Info Log: ${log}`);
					},
					attachShader(program, shader) { gl.attachShader(handles.get(program), handles.get(shader)); },
					linkProgram(program) { gl.linkProgram(handles.get(program)); },
					useProgram(program) { gl.useProgram(handles.get(program)); },
					getProgramParameter(program, pname) { return gl.getProgramParameter(handles.get(program), pname); },
					getProgramInfoLog(program) {
						const log = gl.getProgramInfoLog(handles.get(program));
						console.error(`Program Info Log: ${log}`);
					},
					getActiveUniform(program, index, bufSize, length_ptr, size_ptr, type_ptr, name_ptr) {
						const info = gl.getActiveUniform(handles.get(program), index);
						if (!info) {
							throw new Error(`No active uniform at index ${index}`);
						}

						const name = info.name;
						const nameBytes = new TextEncoder().encode(name);
						if (nameBytes.length >= bufSize) {
							throw new Error(`Uniform name buffer too small: ${bufSize} bytes needed, ${nameBytes.length} bytes provided.`);
						}
						new Uint8Array(memory.buffer, name_ptr, nameBytes.length).set(nameBytes);
						if (length_ptr) {
							new Uint32Array(memory.buffer, length_ptr, 1)[0] = nameBytes.length;
						}
						if (size_ptr) {
							new Uint32Array(memory.buffer, size_ptr, 1)[0] = info.size;
						}
						if (type_ptr) {
							new Uint32Array(memory.buffer, type_ptr, 1)[0] = info.type;
						}
					},
					getUniformLocation(program, name_ptr, name_len) {
						const name = new TextDecoder().decode(new Uint8Array(memory.buffer, name_ptr, name_len));
						const location = gl.getUniformLocation(handles.get(program), name);
						if (!location) {
							throw new Error(`Uniform location for '${name}' not found in program.`);
						}
						return handles.add(location);
					},
					uniform1fv(location, count, value) {
						gl.uniform1fv(handles.get(location), new Float32Array(memory.buffer, value, count * 1));
					},
					uniform2fv(location, count, value) {
						gl.uniform2fv(handles.get(location), new Float32Array(memory.buffer, value, count * 2));
					},
					uniform3fv(location, count, value) {
						gl.uniform3fv(handles.get(location), new Float32Array(memory.buffer, value, count * 3));
					},
					uniform4fv(location, count, value) {
						gl.uniform4fv(handles.get(location), new Float32Array(memory.buffer, value, count * 4));
					},
					uniform1iv(location, count, value) {
						gl.uniform1iv(handles.get(location), new Int32Array(memory.buffer, value, count * 1));
					},
					uniform2iv(location, count, value) {
						gl.uniform2iv(handles.get(location), new Int32Array(memory.buffer, value, count * 2));
					},
					uniform3iv(location, count, value) {
						gl.uniform3iv(handles.get(location), new Int32Array(memory.buffer, value, count * 3));
					},
					uniform4iv(location, count, value) {
						gl.uniform4iv(handles.get(location), new Int32Array(memory.buffer, value, count * 4));
					},
					uniformMatrix2fv(location, count, transpose, value) {
						gl.uniformMatrix2fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 4));
					},
					uniformMatrix2x3fv(location, count, transpose, value) {
						gl.uniformMatrix2x3fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 6));
					},
					uniformMatrix2x4fv(location, count, transpose, value) {
						gl.uniformMatrix2x4fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 8));
					},
					uniformMatrix3x2fv(location, count, transpose, value) {
						gl.uniformMatrix3x2fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 6));
					},
					uniformMatrix3fv(location, count, transpose, value) {
						gl.uniformMatrix3fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 9));
					},
					uniformMatrix3x4fv(location, count, transpose, value) {
						gl.uniformMatrix3x4fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 12));
					},
					uniformMatrix4x2fv(location, count, transpose, value) {
						gl.uniformMatrix4x2fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 8));
					},
					uniformMatrix4x3fv(location, count, transpose, value) {
						gl.uniformMatrix4x3fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 12));
					},
					uniformMatrix4fv(location, count, transpose, value) {
						gl.uniformMatrix4fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 16));
					},
					createTexture() { return handles.add(gl.createTexture()); },
					deleteTexture(texture) {
						let value = handles.remove(texture);
						gl.deleteTexture(value);
					},
					activeTexture(texture) { gl.activeTexture(texture); },
					bindTexture(target, texture) { gl.bindTexture(target, handles.get(texture)); },
					pixelStorei(pname, param) { gl.pixelStorei(pname, param); },
					texParameteri(target, pname, param) { gl.texParameteri(target, pname, param); },
					texImage2D(target, level, internalformat, width, height, border, format, type, pixels_ptr, pixels_len) {
						const pixels = new Uint8Array(memory.buffer, pixels_ptr, pixels_len);
						gl.texImage2D(target, level, internalformat, width, height, border, format, type, pixels);
					},
					drawArrays(mode, first, count) { gl.drawArrays(mode, first, count); },
					drawElements(mode, count, type, offset) { gl.drawElements(mode, count, type, offset); },
				},
				env: {
					consoleLog(msg_ptr, msg_len) {
						const msg = new TextDecoder().decode(new Uint8Array(memory.buffer, msg_ptr, msg_len));
						console.log(msg);
					},
				}
			};

			const response = await fetch(MODULE_NAME);
			if (!response.ok) {
				throw new Error(`Failed to fetch wasm: ${response.statusText}`);
			}

			const bytes = await response.arrayBuffer();
			const { instance } = await WebAssembly.instantiate(bytes, imports);
			wasmInstance = instance;
			let memory = wasmInstance.exports.memory;

			// console.log(wasmInstance.exports);

			var ctx = wasmInstance.exports.new();
			
			// Create input controller
			let inputController = new InputController(canvas);
			canvas.style.cursor = 'grab';

			updatesize = function(width, height) {
				wasmInstance.exports.resize(ctx, width, height);
			};
			updatesize(canvas.width, canvas.height);

			drawFn = function() {
				// Process input if needed
				const deltas = inputController.consumeDeltas();
				if (deltas && wasmInstance.exports.update_camera) {
					wasmInstance.exports.update_camera(ctx, deltas.x, deltas.y, deltas.zoom);
				}
				
				// Always draw
				wasmInstance.exports.draw(ctx, performance.now() / 1000.0);
			}
		} catch (error) {
			console.error("WASM load error:", error);
			alert(`Failed to load the WebAssembly module '${MODULE_NAME}'.`);
		}
	}

	function frame() {
		drawFn();
		requestAnimationFrame(frame);
	}

	loadWasm().then(() => {
		requestAnimationFrame(frame);
	});
});
