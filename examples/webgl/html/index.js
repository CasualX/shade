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

async function startShadeDemo({ canvas, stageElement, moduleName, constructorName }) {
	const gl = canvas.getContext('webgl2');
	if (!gl) {
		throw new Error('WebGL2 is not supported by your browser.');
	}
	gl.getExtension('OES_standard_derivatives');

	const webgl = createWasmAPI(canvas, { colorSpace: 'srgb' });
	const imports = {
		webgl,
		env: {
			consoleLog: webgl.consoleLog,
		},
	};

	const response = await fetch(moduleName);
	if (!response.ok) {
		throw new Error(`Failed to fetch wasm: ${response.status} ${response.statusText}`);
	}

	const bytes = await response.arrayBuffer();
	const { instance } = await WebAssembly.instantiate(bytes, imports);
	webgl.bindInstance(instance);

	const contextHandle = instance.exports[constructorName]();
	const resize = (width, height) => {
		canvas.width = width;
		canvas.height = height;
		if (instance.exports.resize) {
			instance.exports.resize(contextHandle, width, height);
		}
	};

	const disconnectResize = createElementResizer(stageElement, resize);
	let animationFrameId = 0;
	let stopped = false;

	function drawFrame() {
		if (stopped) {
			return;
		}

		try {
			instance.exports.draw(contextHandle, performance.now() / 1000.0);
		}
		catch (error) {
			console.error('Draw error:', error);
			stopped = true;
			disconnectResize();
			return;
		}

		animationFrameId = requestAnimationFrame(drawFrame);
	}

	animationFrameId = requestAnimationFrame(drawFrame);

	return {
		stop() {
			if (stopped) {
				return;
			}
			stopped = true;
			cancelAnimationFrame(animationFrameId);
			disconnectResize();
			if (instance.exports.drop) {
				instance.exports.drop(contextHandle);
			}
		},
		mousemove(deltaX, deltaY) {
			if (instance.exports.mousemove) {
				instance.exports.mousemove(contextHandle, deltaX, deltaY);
			}
		},
		mousedown(button) {
			if (instance.exports.mousedown) {
				instance.exports.mousedown(contextHandle, button);
			}
		},
		mouseup(button) {
			if (instance.exports.mouseup) {
				instance.exports.mouseup(contextHandle, button);
			}
		},
		wheel(deltaY) {
			if (instance.exports.wheel) {
				instance.exports.wheel(contextHandle, deltaY);
			}
		},
		keydown(key) {
			if (instance.exports.keydown) {
				instance.exports.keydown(contextHandle, key);
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
		source: 'https://github.com/CasualX/shade/blob/master/examples/webgl/src/triangle.rs',
	},
	{
		id: 'oldtree',
		title: 'Old Tree',
		hint: 'Stylized scene rendering',
		module: 'webgl.wasm',
		constructor: 'new_oldtree',
		source: 'https://github.com/CasualX/shade/blob/master/examples/webgl/src/oldtree.rs',
	},
	{
		id: 'text',
		title: 'Text',
		hint: 'Signed distance field text',
		module: 'webgl.wasm',
		constructor: 'new_text',
		source: 'https://github.com/CasualX/shade/blob/master/examples/webgl/src/text.rs',
	},
	{
		id: 'text3d',
		title: 'Text 3D',
		hint: '3D text planes with orbit controls',
		module: 'webgl.wasm',
		constructor: 'new_text3d',
		source: 'https://github.com/CasualX/shade/blob/master/examples/webgl/src/text3d.rs',
	},
	{
		id: 'textintro',
		title: 'Text Intro',
		hint: 'Animated 3D opening crawl',
		module: 'webgl.wasm',
		constructor: 'new_textintro',
		source: 'https://github.com/CasualX/shade/blob/master/examples/webgl/src/textintro.rs',
	},
	{
		id: 'pixelart',
		title: 'Pixel Art',
		hint: 'Interactive texture filtering demo',
		module: 'webgl.wasm',
		constructor: 'new_pixelart',
		source: 'https://github.com/CasualX/shade/blob/master/examples/webgl/src/pixelart.rs',
	},
	{
		id: 'zeldawater',
		title: 'Zelda Water',
		hint: 'Water shader experiment',
		module: 'webgl.wasm',
		constructor: 'new_zeldawater',
		source: 'https://github.com/CasualX/shade/blob/master/examples/webgl/src/zeldawater.rs',
	},
	{
		id: 'globe',
		title: 'Globe',
		hint: 'Texturing + camera',
		module: 'webgl.wasm',
		constructor: 'new_globe',
		source: 'https://github.com/CasualX/shade/blob/master/examples/webgl/src/globe.rs',
	},
];

Alpine.data('shadeWebglApp', () => ({
	demos,
	activeDemo: null,
	activeRunner: null,
	isLoading: false,
	lastPointerX: 0,
	lastPointerY: 0,

	async openDemo(demo) {
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
		if (this.activeRunner) {
			this.activeRunner.stop();
		}

		this.activeRunner = null;
		this.isLoading = false;
		this.lastPointerX = 0;
		this.lastPointerY = 0;
	},

	showGallery() {
		this.stopActiveDemo();
		this.activeDemo = null;
	},

	toggleFullscreen() {
		if (document.fullscreenElement) {
			document.exitFullscreen?.();
			return;
		}

		this.$refs.playerStage.requestFullscreen?.();
	},

	handleMousemove(event) {
		if (!this.activeRunner) {
			return;
		}

		const deltaX = event.clientX - this.lastPointerX;
		const deltaY = event.clientY - this.lastPointerY;
		this.lastPointerX = event.clientX;
		this.lastPointerY = event.clientY;
		this.activeRunner.mousemove(deltaX, deltaY);
	},

	handleMousedown(event) {
		this.lastPointerX = event.clientX;
		this.lastPointerY = event.clientY;
		if (this.activeRunner) {
			this.activeRunner.mousedown(event.button);
		}
	},

	handleMouseup(event) {
		if (this.activeRunner) {
			this.activeRunner.mouseup(event.button);
		}
	},

	handleWheel(event) {
		if (!this.activeRunner) {
			return;
		}

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
}));

window.Alpine = Alpine;
Alpine.start();
