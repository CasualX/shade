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

function clampDevicePixelRatio(dpr) {
	// Prevent extreme DPR from creating huge canvases.
	return Math.max(1, Math.min(2, dpr || 1));
}

function createElementResizer(element, onResize) {
	let lastW = 0;
	let lastH = 0;
	const ro = new ResizeObserver(() => {
		const rect = element.getBoundingClientRect();
		const dpr = clampDevicePixelRatio(window.devicePixelRatio);
		const w = Math.max(1, Math.floor(rect.width * dpr));
		const h = Math.max(1, Math.floor(rect.height * dpr));
		if (w === lastW && h === lastH) return;
		lastW = w;
		lastH = h;
		onResize(w, h);
	});
	ro.observe(element);
	// Trigger once.
	const rect = element.getBoundingClientRect();
	const dpr = clampDevicePixelRatio(window.devicePixelRatio);
	onResize(Math.max(1, Math.floor(rect.width * dpr)), Math.max(1, Math.floor(rect.height * dpr)));
	return () => ro.disconnect();
}

export async function runShadeDemo({ canvas, moduleName, resizeTo = null, onLoaded = null, onError = null }) {
	if (!canvas) throw new Error('runShadeDemo: canvas is required');
	if (!moduleName) throw new Error('runShadeDemo: moduleName is required');

	const gl = canvas.getContext('webgl');
	if (!gl) throw new Error('WebGL is not supported by your browser.');
	gl.getExtension('OES_standard_derivatives');

	let stopped = false;
	let rafId = null;
	let wasmInstance = null;
	let ctx = null;
	let updatesize = null;
	let memory = null;
	const abort = new AbortController();

	function stopAnimation() {
		if (rafId != null) {
			cancelAnimationFrame(rafId);
			rafId = null;
		}
	}

	function stop() {
		if (stopped) return;
		stopped = true;
		stopAnimation();
		try { abort.abort(); } catch (_) {}
		try {
			if (wasmInstance?.exports?.drop && ctx != null) {
				wasmInstance.exports.drop(ctx);
			}
		} catch (_) {}
		wasmInstance = null;
		ctx = null;
		updatesize = null;
		memory = null;
	}

	const disconnectResize = createElementResizer(resizeTo || canvas, (w, h) => {
		// Keep CSS size controlled by layout; set backing store in device pixels.
		canvas.width = w;
		canvas.height = h;
		if (updatesize) updatesize(w, h);
	});

	function attachMouseControls() {
		let lastX = 0, lastY = 0;
		canvas.addEventListener('mousemove', (e) => {
			const dx = e.clientX - lastX;
			const dy = e.clientY - lastY;
			lastX = e.clientX;
			lastY = e.clientY;
			if (wasmInstance?.exports?.mousemove && ctx != null) {
				wasmInstance.exports.mousemove(ctx, dx, dy);
			}
		}, { signal: abort.signal });

		canvas.addEventListener('mousedown', (e) => {
			lastX = e.clientX;
			lastY = e.clientY;
			if (wasmInstance?.exports?.mousedown && ctx != null) {
				wasmInstance.exports.mousedown(ctx, e.button);
			}
		}, { signal: abort.signal });

		canvas.addEventListener('mouseup', (e) => {
			if (wasmInstance?.exports?.mouseup && ctx != null) {
				wasmInstance.exports.mouseup(ctx, e.button);
			}
		}, { signal: abort.signal });

		canvas.addEventListener('contextmenu', (e) => e.preventDefault(), { signal: abort.signal });
	}

	async function loadWasm() {
		let handles = new HandleTable();
		const decoder = new TextDecoder();
		const encoder = new TextEncoder();
		const imports = {
			webgl: {
				consoleLog(msg_ptr, msg_len) {
					if (!memory) return;
					const msg = decoder.decode(new Uint8Array(memory.buffer, msg_ptr, msg_len));
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
					if (!memory) return;
					const data = new Uint8Array(memory.buffer, data_ptr, size);
					gl.bufferData(target, data, usage);
				},
				deleteBuffer(buffer) {
					gl.deleteBuffer(handles.get(buffer));
					handles.remove(buffer);
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
					if (!memory) return;
					const source = decoder.decode(new Uint8Array(memory.buffer, source_ptr, source_len));
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
					const nameBytes = encoder.encode(name);
					if (nameBytes.length >= bufSize) {
						throw new Error(`Uniform name buffer too small: ${bufSize} bytes needed, ${nameBytes.length} bytes provided.`);
					}
					if (!memory) return;
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
					if (!memory) return 0;
					const name = decoder.decode(new Uint8Array(memory.buffer, name_ptr, name_len));
					const location = gl.getUniformLocation(handles.get(program), name);
					if (!location) {
						throw new Error(`Uniform location for '${name}' not found in program.`);
					}
					return handles.add(location);
				},
				getActiveAttrib(program, index, bufSize, length_ptr, size_ptr, type_ptr, name_ptr) {
					const info = gl.getActiveAttrib(handles.get(program), index);
					if (!info) {
						throw new Error(`No active uniform at index ${index}`);
					}

					const name = info.name;
					const nameBytes = encoder.encode(name);
					if (nameBytes.length >= bufSize) {
						throw new Error(`Attrib name buffer too small: ${bufSize} bytes needed, ${nameBytes.length} bytes provided.`);
					}
					if (!memory) return;
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
				getAttribLocation(program, name_ptr, name_len) {
					if (!memory) return -1;
					const name = decoder.decode(new Uint8Array(memory.buffer, name_ptr, name_len));
					return gl.getAttribLocation(handles.get(program), name);
				},
				uniform1fv(location, count, value) {
					if (!memory) return;
					gl.uniform1fv(handles.get(location), new Float32Array(memory.buffer, value, count * 1));
				},
				uniform2fv(location, count, value) {
					if (!memory) return;
					gl.uniform2fv(handles.get(location), new Float32Array(memory.buffer, value, count * 2));
				},
				uniform3fv(location, count, value) {
					if (!memory) return;
					gl.uniform3fv(handles.get(location), new Float32Array(memory.buffer, value, count * 3));
				},
				uniform4fv(location, count, value) {
					if (!memory) return;
					gl.uniform4fv(handles.get(location), new Float32Array(memory.buffer, value, count * 4));
				},
				uniform1iv(location, count, value) {
					if (!memory) return;
					gl.uniform1iv(handles.get(location), new Int32Array(memory.buffer, value, count * 1));
				},
				uniform2iv(location, count, value) {
					if (!memory) return;
					gl.uniform2iv(handles.get(location), new Int32Array(memory.buffer, value, count * 2));
				},
				uniform3iv(location, count, value) {
					if (!memory) return;
					gl.uniform3iv(handles.get(location), new Int32Array(memory.buffer, value, count * 3));
				},
				uniform4iv(location, count, value) {
					if (!memory) return;
					gl.uniform4iv(handles.get(location), new Int32Array(memory.buffer, value, count * 4));
				},
				uniformMatrix2fv(location, count, transpose, value) {
					if (!memory) return;
					gl.uniformMatrix2fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 4));
				},
				uniformMatrix2x3fv(location, count, transpose, value) {
					if (!memory) return;
					gl.uniformMatrix2x3fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 6));
				},
				uniformMatrix2x4fv(location, count, transpose, value) {
					if (!memory) return;
					gl.uniformMatrix2x4fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 8));
				},
				uniformMatrix3x2fv(location, count, transpose, value) {
					if (!memory) return;
					gl.uniformMatrix3x2fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 6));
				},
				uniformMatrix3fv(location, count, transpose, value) {
					if (!memory) return;
					gl.uniformMatrix3fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 9));
				},
				uniformMatrix3x4fv(location, count, transpose, value) {
					if (!memory) return;
					gl.uniformMatrix3x4fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 12));
				},
				uniformMatrix4x2fv(location, count, transpose, value) {
					if (!memory) return;
					gl.uniformMatrix4x2fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 8));
				},
				uniformMatrix4x3fv(location, count, transpose, value) {
					if (!memory) return;
					gl.uniformMatrix4x3fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 12));
				},
				uniformMatrix4fv(location, count, transpose, value) {
					if (!memory) return;
					gl.uniformMatrix4fv(handles.get(location), transpose, new Float32Array(memory.buffer, value, count * 16));
				},
				createTexture() { return handles.add(gl.createTexture()); },
				deleteTexture(texture) {
					let value = handles.remove(texture);
					gl.deleteTexture(value);
				},
				activeTexture(texture) { gl.activeTexture(texture); },
				bindTexture(target, texture) { gl.bindTexture(target, handles.get(texture)); },
				generateMipmap(target) { gl.generateMipmap(target); },
				pixelStorei(pname, param) { gl.pixelStorei(pname, param); },
				texParameteri(target, pname, param) { gl.texParameteri(target, pname, param); },
				texImage2D(target, level, internalformat, width, height, border, format, type, pixels_ptr, pixels_len) {
					if (!memory) return;
					const pixels = new Uint8Array(memory.buffer, pixels_ptr, pixels_len);
					gl.texImage2D(target, level, internalformat, width, height, border, format, type, pixels);
				},
				drawArrays(mode, first, count) { gl.drawArrays(mode, first, count); },
				drawElements(mode, count, type, offset) { gl.drawElements(mode, count, type, offset); },
			},
			env: {
				consoleLog(msg_ptr, msg_len) {
					if (!memory) return;
					const msg = decoder.decode(new Uint8Array(memory.buffer, msg_ptr, msg_len));
					console.log(msg);
				},
			}
		};

		const response = await fetch(moduleName);
		if (!response.ok) {
			throw new Error(`Failed to fetch wasm: ${response.status} ${response.statusText}`);
		}

		const bytes = await response.arrayBuffer();
		const { instance } = await WebAssembly.instantiate(bytes, imports);
		wasmInstance = instance;
		memory = wasmInstance.exports.memory;

		if (!wasmInstance.exports.new) {
			throw new Error('WASM module missing required export: new');
		}
		if (!wasmInstance.exports.draw) {
			throw new Error('WASM module missing required export: draw');
		}

		ctx = wasmInstance.exports.new();
		updatesize = (width, height) => {
			if (wasmInstance?.exports?.resize) {
				wasmInstance.exports.resize(ctx, width, height);
			}
		};
		// Force an initial resize call (ResizeObserver may have already run before updatesize existed).
		updatesize(canvas.width, canvas.height);

		attachMouseControls();
	}

	function frame() {
		if (stopped) return;
		try {
			wasmInstance.exports.draw(ctx, performance.now() / 1000.0);
		} catch (e) {
			console.error('Draw error:', e);
			stop();
			if (onError) onError(e);
			return;
		}
		rafId = requestAnimationFrame(frame);
	}

	try {
		await loadWasm();
		if (stopped) return { stop };
		if (onLoaded) onLoaded();
		rafId = requestAnimationFrame(frame);
	} catch (e) {
		console.error('WASM load error:', e);
		stop();
		disconnectResize();
		if (onError) onError(e);
		throw e;
	}

	return {
		stop() {
			stop();
			disconnectResize();
		}
	};
}
