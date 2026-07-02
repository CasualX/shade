import Alpine from 'https://cdn.jsdelivr.net/npm/alpinejs@3.14.9/dist/module.esm.js';
import { createWasmAPI } from './shade.js';

function clampDevicePixelRatio(devicePixelRatio) {
	return Math.max(1, Math.min(2, devicePixelRatio || 1));
}

function createElementResizer(element, onResize) {
	let lastWidth = 0;
	let lastHeight = 0;

	const resizeObserver = new ResizeObserver(() => {
		const rect = element.getBoundingClientRect();
		const devicePixelRatio = clampDevicePixelRatio(window.devicePixelRatio);
		const width = Math.max(1, Math.floor(rect.width * devicePixelRatio));
		const height = Math.max(1, Math.floor(rect.height * devicePixelRatio));
		if (width === lastWidth && height === lastHeight) {
			return;
		}
		lastWidth = width;
		lastHeight = height;
		onResize(width, height);
	});

	resizeObserver.observe(element);

	const rect = element.getBoundingClientRect();
	const devicePixelRatio = clampDevicePixelRatio(window.devicePixelRatio);
	onResize(
		Math.max(1, Math.floor(rect.width * devicePixelRatio)),
		Math.max(1, Math.floor(rect.height * devicePixelRatio)),
	);

	return () => resizeObserver.disconnect();
}

function getDemoKey(code) {
	switch (code) {
		case 'Digit1':
		case 'Numpad1':
			return 1;
		case 'Digit2':
		case 'Numpad2':
			return 2;
		case 'Digit3':
		case 'Numpad3':
			return 3;
		case 'ArrowLeft':
			return 10;
		case 'ArrowRight':
			return 11;
		case 'ArrowUp':
			return 12;
		case 'ArrowDown':
			return 13;
		case 'ShiftLeft':
		case 'ShiftRight':
			return 20;
		case 'KeyP':
			return 21;
		case 'F2':
			return 22;
		case 'Escape':
			return 23;
		default:
			return 0;
	}
}

function getWheelDelta(event, canvas) {
	if (event.deltaMode === WheelEvent.DOM_DELTA_LINE) {
		return event.deltaY * 16;
	}
	if (event.deltaMode === WheelEvent.DOM_DELTA_PAGE) {
		return event.deltaY * canvas.height;
	}
	return event.deltaY;
}

function getCanvasPointerPosition(event, canvas) {
	const rect = canvas.getBoundingClientRect();
	const scaleX = rect.width > 0 ? canvas.width / rect.width : 1;
	const scaleY = rect.height > 0 ? canvas.height / rect.height : 1;
	return {
		x: (event.clientX - rect.left) * scaleX,
		y: (event.clientY - rect.top) * scaleY,
	};
}

function hasTouchSupport() {
	return 'ontouchstart' in window || navigator.maxTouchPoints > 0 || window.matchMedia('(pointer: coarse)').matches;
}

function findDemoById(demoId) {
	return demos.find((demo) => demo.id === demoId) || null;
}

function getRouteDemo() {
	const route = window.location.hash.replace(/^#/, '').trim();
	return route ? findDemoById(route) : null;
}

function getCssCursor(cursor) {
	switch (cursor) {
		case 1:
			return 'pointer';
		case 2:
			return 'grab';
		case 3:
			return 'grabbing';
		case 4:
			return 'crosshair';
		case 5:
			return 'move';
		case 6:
			return 'ew-resize';
		case 7:
			return 'ns-resize';
		case 8:
			return 'nwse-resize';
		case 9:
			return 'nesw-resize';
		default:
			return 'default';
	}
}

async function startShadeDemo({ canvas, stageElement, moduleName, constructorName }) {
	const gl = canvas.getContext('webgl2');
	if (!gl) {
		throw new Error('WebGL2 is not supported by your browser.');
	}
	gl.getExtension('OES_standard_derivatives');

	const webgl = createWasmAPI(canvas, { colorSpace: 'srgb' });
	let instance = null;
	let openFileDialog = null;
	const imports = {
		webgl,
		env: {
			consoleLog: webgl.consoleLog,
			setCursor(cursor) {
				canvas.style.cursor = getCssCursor(cursor);
			},
			openFile(requestId, titlePtr, titleLen, extensionsPtr, extensionsLen) {
				const title = webgl.decodeUtf8(titlePtr, titleLen) || 'Open file';
				const extensions = webgl.decodeUtf8(extensionsPtr, extensionsLen) || '';
				openFileDialog?.(requestId, title, extensions);
			},
			setStatus(textPtr, textLen) {
				const text = webgl.decodeUtf8(textPtr, textLen);
				if (text) {
					console.warn(text);
				}
			},
			randomFill(destPtr, destLen) {
				if (!instance || !instance.exports.memory) {
					return false;
				}
				const memory = new Uint8Array(instance.exports.memory.buffer);
				for (let offset = 0; offset < destLen; offset += 65536) {
					const chunkLen = Math.min(65536, destLen - offset);
					crypto.getRandomValues(memory.subarray(destPtr + offset, destPtr + offset + chunkLen));
				}
				return true;
			},
		},
	};

	const response = await fetch(moduleName);
	if (!response.ok) {
		throw new Error(`Failed to fetch wasm: ${response.status} ${response.statusText}`);
	}

	const bytes = await response.arrayBuffer();
	({ instance } = await WebAssembly.instantiate(bytes, imports));
	webgl.bindInstance(instance);

	let animationFrameId = null;
	let stopped = false;
	let startTimeMs = null;
	let pendingRedraw = false;
	let isContinuous = true;
	const contextHandle = instance.exports[constructorName]();
	const resize = (width, height) => {
		canvas.width = width;
		canvas.height = height;
		if (instance.exports.resize) {
			instance.exports.resize(contextHandle, width, height);
		}
		pendingRedraw = true;
		requestFrame();
	};

	const disconnectResize = createElementResizer(stageElement, resize);

	function syncRedrawState() {
		if (instance.exports.redraw_mode) {
			isContinuous = instance.exports.redraw_mode(contextHandle) !== 0;
		}
		if (instance.exports.take_redraw_request) {
			pendingRedraw = instance.exports.take_redraw_request(contextHandle) || pendingRedraw;
		}
	}

	function requestFrame() {
		if (stopped || animationFrameId !== null) {
			return;
		}
		animationFrameId = requestAnimationFrame(drawFrame);
	}

	function sendFileOpened(requestId, path, bytes) {
		if (!instance.exports.file_opened) {
			return;
		}

		const pathBytes = path ? new TextEncoder().encode(path) : null;
		const dataBytes = bytes ? new Uint8Array(bytes) : null;
		let pathPtr = 0;
		let dataPtr = 0;

		try {
			if (pathBytes && pathBytes.length > 0) {
				pathPtr = instance.exports.allocate(pathBytes.length);
				new Uint8Array(instance.exports.memory.buffer, pathPtr, pathBytes.length).set(pathBytes);
			}
			if (dataBytes && dataBytes.length > 0) {
				dataPtr = instance.exports.allocate(dataBytes.length);
				new Uint8Array(instance.exports.memory.buffer, dataPtr, dataBytes.length).set(dataBytes);
			}
			instance.exports.file_opened(
				contextHandle,
				requestId,
				pathPtr,
				pathBytes?.length || 0,
				dataPtr,
				dataBytes?.length || 0,
			);
		}
		finally {
			if (pathPtr) {
				instance.exports.free(pathPtr, pathBytes.length);
			}
			if (dataPtr) {
				instance.exports.free(dataPtr, dataBytes.length);
			}
		}

		syncRedrawState();
		if (isContinuous || pendingRedraw) {
			requestFrame();
		}
	}

	openFileDialog = (requestId, title, extensions) => {
		const input = document.createElement('input');
		input.type = 'file';
		input.accept = extensions
			.split(',')
			.map((ext) => ext.trim())
			.filter(Boolean)
			.map((ext) => `.${ext}`)
			.join(',');
		input.style.display = 'none';
		input.title = title;
		const cleanup = () => input.remove();
		input.addEventListener('change', async () => {
			const [file] = input.files || [];
			if (!file) {
				sendFileOpened(requestId, null, null);
				cleanup();
				return;
			}
			try {
				sendFileOpened(requestId, file.name, await file.arrayBuffer());
			}
			finally {
				cleanup();
			}
		}, { once: true });
		input.addEventListener('cancel', () => {
			sendFileOpened(requestId, null, null);
			cleanup();
		}, { once: true });
		document.body.appendChild(input);
		input.click();
	};

	function drawFrame(timestampMs) {
		animationFrameId = null;
		if (stopped) {
			return;
		}

		syncRedrawState();
		if (!isContinuous && !pendingRedraw) {
			return;
		}

		if (startTimeMs === null) {
			startTimeMs = timestampMs;
		}

		try {
			pendingRedraw = false;
			instance.exports.draw(contextHandle, (timestampMs - startTimeMs) / 1000.0);
		}
		catch (error) {
			console.error('Draw error:', error);
			stopped = true;
			disconnectResize();
			return;
		}

		syncRedrawState();
		if (isContinuous || pendingRedraw) {
			requestFrame();
		}
	}

	syncRedrawState();
	requestFrame();

	return {
		stop() {
			if (stopped) {
				return;
			}
			stopped = true;
			if (animationFrameId !== null) {
				cancelAnimationFrame(animationFrameId);
				animationFrameId = null;
			}
			disconnectResize();
			canvas.style.cursor = 'default';
			if (instance.exports.drop) {
				instance.exports.drop(contextHandle);
			}
		},
		dispatchInput(callback) {
			callback();
			syncRedrawState();
			if (isContinuous || pendingRedraw) {
				requestFrame();
			}
		},
		mousemove(deltaX, deltaY) {
			if (instance.exports.mousemove) {
				this.dispatchInput(() => instance.exports.mousemove(contextHandle, deltaX, deltaY));
			}
		},
		mousedown(button) {
			if (instance.exports.mousedown) {
				this.dispatchInput(() => instance.exports.mousedown(contextHandle, button));
			}
		},
		mouseup(button) {
			if (instance.exports.mouseup) {
				this.dispatchInput(() => instance.exports.mouseup(contextHandle, button));
			}
		},
		wheel(deltaY) {
			if (instance.exports.wheel) {
				this.dispatchInput(() => instance.exports.wheel(contextHandle, deltaY));
			}
		},
		keydown(key) {
			if (instance.exports.keydown) {
				this.dispatchInput(() => instance.exports.keydown(contextHandle, key));
			}
		},
		keyup(key) {
			if (instance.exports.keyup) {
				this.dispatchInput(() => instance.exports.keyup(contextHandle, key));
			}
		},
	};
}

const demos = [
	{
		id: 'triangle',
		title: 'Triangle',
		hint: 'Minimal pipeline + draw call',
		module: 'webgl.wasm',
		constructor: 'new_triangle',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/triangle.rs',
	},
	{
		id: 'oldtree',
		title: 'Old Tree',
		hint: 'Stylized scene rendering',
		module: 'webgl.wasm',
		constructor: 'new_oldtree',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/oldtree.rs',
	},
	{
		id: 'conway',
		title: 'Conway',
		hint: 'Ping-pong texture simulation',
		module: 'webgl.wasm',
		constructor: 'new_conway',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/conway.rs',
	},
	{
		id: 'dither',
		title: 'Dither',
		hint: 'Ordered dither post-process',
		module: 'webgl.wasm',
		constructor: 'new_dither',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/dither.rs',
	},
	{
		id: 'mandelbrot',
		title: 'Mandelbrot',
		hint: 'Interactive fractal zoom',
		module: 'webgl.wasm',
		constructor: 'new_mandelbrot',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/mandelbrot.rs',
	},
	{
		id: 'text',
		title: 'Text',
		hint: 'Signed distance field text',
		module: 'webgl.wasm',
		constructor: 'new_text',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/text.rs',
	},
	{
		id: 'text3d',
		title: 'Text 3D',
		hint: '3D text planes with orbit controls',
		module: 'webgl.wasm',
		constructor: 'new_text3d',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/text3d.rs',
	},
	{
		id: 'textintro',
		title: 'Text Intro',
		hint: 'Animated 3D opening crawl',
		module: 'webgl.wasm',
		constructor: 'new_textintro',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/textintro.rs',
	},
	{
		id: 'pixelart',
		title: 'Pixel Art',
		hint: 'Interactive texture filtering demo',
		module: 'webgl.wasm',
		constructor: 'new_pixelart',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/pixelart.rs',
	},
	{
		id: 'zeldawater',
		title: 'Zelda Water',
		hint: 'Water shader experiment',
		module: 'webgl.wasm',
		constructor: 'new_zeldawater',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/zeldawater.rs',
	},
	{
		id: 'globe',
		title: 'Globe',
		hint: 'Texturing + camera',
		module: 'webgl.wasm',
		constructor: 'new_globe',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/globe.rs',
	},
	{
		id: 'gui_zoo',
		title: 'GUI Zoo',
		hint: 'Retained GUI controls and clipping',
		module: 'webgl.wasm',
		constructor: 'new_gui_zoo',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/gui_zoo.rs',
	},
	{
		id: 'panels',
		title: 'Panels',
		hint: 'Nine-slice panel layout',
		module: 'webgl.wasm',
		constructor: 'new_panels',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/panels.rs',
	},
	{
		id: 'polygon',
		title: 'Polygon',
		hint: 'Interactive triangulation',
		module: 'webgl.wasm',
		constructor: 'new_polygon',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/polygon.rs',
	},
	{
		id: 'scene',
		title: 'Scene',
		hint: 'Simple 3D sprite scene',
		module: 'webgl.wasm',
		constructor: 'new_scene',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/scene.rs',
	},
	{
		id: 'screenmelt',
		title: 'Screen Melt',
		hint: 'Retro post-process transition',
		module: 'webgl.wasm',
		constructor: 'new_screenmelt',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/screenmelt.rs',
	},
	{
		id: 'shadertoy',
		title: 'Shader Toy',
		hint: 'Fullscreen procedural shader',
		module: 'webgl.wasm',
		constructor: 'new_shadertoy',
		source: 'https://github.com/CasualX/shade/blob/master/examples/demos/src/examples/shadertoy.rs',
	},
];

Alpine.data('shadeWebglApp', () => ({
	demos,
	activeDemo: null,
	activeRunner: null,
	isLoading: false,
	lastPointerX: 0,
	lastPointerY: 0,
	touchModeEnabled: hasTouchSupport(),
	touchButton: 0,
	activeTouchPointerId: null,
	activeTouchButton: 0,
	isRouting: false,
	touchButtonModes: [
		{ button: 0, label: 'Left' },
		{ button: 1, label: 'Middle' },
		{ button: 2, label: 'Right' },
	],

	init() {
		this.handleRouteChange = this.handleRouteChange.bind(this);
		window.addEventListener('hashchange', this.handleRouteChange);
		window.addEventListener('popstate', this.handleRouteChange);
		this.handleRouteChange();
	},

	destroy() {
		window.removeEventListener('hashchange', this.handleRouteChange);
		window.removeEventListener('popstate', this.handleRouteChange);
		this.stopActiveDemo();
	},

	setTouchButton(button) {
		this.touchButton = button;
	},

	async openDemo(demo, { updateHistory = true } = {}) {
		if (!demo) {
			return;
		}

		if (updateHistory) {
			window.location.hash = demo.id;
			return;
		}

		this.stopActiveDemo();
		this.activeDemo = demo;
		this.isLoading = true;

		await this.$nextTick();

		const stageElement = this.$refs.playerStage;
		const canvas = this.$refs.canvas;

		try {
			this.activeRunner = await startShadeDemo({
				canvas,
				stageElement,
				moduleName: demo.module,
				constructorName: demo.constructor,
			});
			this.isLoading = false;
			canvas.focus();
		}
		catch (error) {
			console.error('Failed to start demo:', error);
			this.isLoading = false;
		}
	},

	stopActiveDemo() {
		this.releaseActiveTouch();
		if (this.activeRunner) {
			this.activeRunner.stop();
		}

		this.activeRunner = null;
		this.isLoading = false;
		this.lastPointerX = 0;
		this.lastPointerY = 0;
	},

	showGallery({ updateHistory = true } = {}) {
		if (updateHistory) {
			history.pushState(null, '', `${window.location.pathname}${window.location.search}`);
			this.handleRouteChange();
			return;
		}

		this.stopActiveDemo();
		this.activeDemo = null;
	},

	async toggleFullscreen() {
		if (document.fullscreenElement) {
			screen.orientation?.unlock?.();
			document.exitFullscreen?.();
			return;
		}

		const stage = this.$refs.playerStage;
		if (!stage?.requestFullscreen) {
			return;
		}

		try {
			await stage.requestFullscreen();
			if (screen.orientation?.lock) {
				await screen.orientation.lock('landscape');
			}
		}
		catch (error) {
			console.warn('Fullscreen/orientation request failed:', error);
		}
	},

	syncPointer(event) {
		if (!this.activeRunner) {
			return;
		}

		const { x, y } = getCanvasPointerPosition(event, this.$refs.canvas);
		const deltaX = x - this.lastPointerX;
		const deltaY = y - this.lastPointerY;
		this.lastPointerX = x;
		this.lastPointerY = y;
		if (deltaX !== 0 || deltaY !== 0) {
			this.activeRunner.mousemove(deltaX, deltaY);
		}
	},

	getPointerButton(event) {
		return event.pointerType === 'touch' ? this.touchButton : event.button;
	},

	releaseActiveTouch(event = null) {
		if (this.activeTouchPointerId === null) {
			return;
		}

		if (event && event.pointerId !== this.activeTouchPointerId) {
			return;
		}

		const canvas = this.$refs.canvas;
		if (canvas && this.activeTouchPointerId !== null && canvas.hasPointerCapture?.(this.activeTouchPointerId)) {
			canvas.releasePointerCapture(this.activeTouchPointerId);
		}

		this.activeTouchPointerId = null;
		this.activeTouchButton = 0;
	},

	handlePointermove(event) {
		if (!this.activeRunner) {
			return;
		}

		if (event.pointerType === 'touch' && event.pointerId !== this.activeTouchPointerId) {
			return;
		}

		this.syncPointer(event);
		if (event.pointerType === 'touch') {
			event.preventDefault();
		}
	},

	handlePointerdown(event) {
		if (!this.activeRunner) {
			return;
		}

		if (event.pointerType === 'touch' && this.activeTouchPointerId !== null && event.pointerId !== this.activeTouchPointerId) {
			event.preventDefault();
			return;
		}

		const button = this.getPointerButton(event);
		this.syncPointer(event);
		if (event.pointerType === 'touch') {
			this.activeTouchPointerId = event.pointerId;
			this.activeTouchButton = button;
			this.$refs.canvas.setPointerCapture?.(event.pointerId);
			event.preventDefault();
		}
		this.activeRunner.mousedown(button);
	},

	handlePointerup(event) {
		if (!this.activeRunner) {
			return;
		}

		if (event.pointerType === 'touch' && event.pointerId !== this.activeTouchPointerId) {
			return;
		}

		this.syncPointer(event);
		const button = event.pointerType === 'touch' ? this.activeTouchButton : event.button;
		this.activeRunner.mouseup(button);
		if (event.pointerType === 'touch') {
			this.releaseActiveTouch(event);
			event.preventDefault();
		}
	},

	handlePointercancel(event) {
		if (!this.activeRunner || event.pointerType !== 'touch' || event.pointerId !== this.activeTouchPointerId) {
			return;
		}

		this.activeRunner.mouseup(this.activeTouchButton);
		this.releaseActiveTouch(event);
		event.preventDefault();
	},

	handleWheel(event) {
		if (!this.activeRunner) {
			return;
		}

		this.syncPointer(event);
		this.activeRunner.wheel(getWheelDelta(event, this.$refs.canvas));
		event.preventDefault();
	},

	handleKeydown(event) {
		if (event.key === 'f' || event.key === 'F') {
			event.preventDefault();
			this.toggleFullscreen();
			return;
		}

		const key = getDemoKey(event.code);
		if (!key || !this.activeRunner) {
			return;
		}

		this.activeRunner.keydown(key);
		event.preventDefault();
	},

	handleKeyup(event) {
		const key = getDemoKey(event.code);
		if (!key || !this.activeRunner) {
			return;
		}

		this.activeRunner.keyup(key);
		event.preventDefault();
	},

	handleRouteChange() {
		if (this.isRouting) {
			return;
		}

		this.isRouting = true;
		try {
			const routeDemo = getRouteDemo();
			if (!routeDemo) {
				if (this.activeDemo) {
					this.showGallery({ updateHistory: false });
				}
				return;
			}

			if (this.activeDemo?.id === routeDemo.id) {
				return;
			}

			void this.openDemo(routeDemo, { updateHistory: false });
		}
		finally {
			this.isRouting = false;
		}
	},
}));

window.Alpine = Alpine;
Alpine.start();
