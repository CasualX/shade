use std::ptr;

use super::*;

fn into_handle<T: crate::DemoContext + 'static>(ctx: T) -> *mut DemoHandle {
	Box::into_raw(Box::new(Box::new(ctx)))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_triangle() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::triangle::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_oldtree() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::oldtree::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_conway() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::conway::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_dither() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::dither::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_mandelbrot() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::mandelbrot::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_panels() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::panels::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_polygon() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::polygon::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_scene() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::scene::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_screenmelt() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::screenmelt::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_shadertoy() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::shadertoy::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_text() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::text::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_text3d() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::text3d::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_textintro() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::textintro::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_pixelart() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::pixelart::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_zeldawater() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::zeldawater::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_globe() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::globe::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn new_gui_zoo() -> *mut DemoHandle {
	into_handle(SharedContext::new(demos::examples::gui_zoo::create))
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn drop(ctx: *mut DemoHandle) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		let _ = Box::from_raw(ctx);
	}
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn resize(ctx: *mut DemoHandle, width: i32, height: i32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).resize(width, height);
	}
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn mousemove(ctx: *mut DemoHandle, dx: f32, dy: f32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).mousemove(dx, dy);
	}
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn mousedown(ctx: *mut DemoHandle, button: u32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).mousedown(button);
	}
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn mouseup(ctx: *mut DemoHandle, button: u32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).mouseup(button);
	}
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn wheel(ctx: *mut DemoHandle, delta_y: f32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).wheel(delta_y);
	}
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn keydown(ctx: *mut DemoHandle, key: u32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).keydown(key);
	}
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn keyup(ctx: *mut DemoHandle, key: u32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).keyup(key);
	}
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn draw(ctx: *mut DemoHandle, time: f64) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).draw(time);
	}
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn redraw_mode(ctx: *mut DemoHandle) -> u32 {
	if ctx.is_null() {
		return RedrawMode::Continuous as u32;
	}
	unsafe {
		match (**ctx).redraw_mode() {
			RedrawMode::OnDemand => 0,
			RedrawMode::Continuous => 1,
		}
	}
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn take_redraw_request(ctx: *mut DemoHandle) -> bool {
	if ctx.is_null() {
		return false;
	}
	unsafe { (**ctx).take_redraw_request() }
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn file_opened(ctx: *mut DemoHandle, request_id: u32, path_ptr: *const u8, path_len: usize, bytes_ptr: *const u8, bytes_len: usize) {
	if ctx.is_null() {
		return;
	}
	let path = if path_ptr.is_null() || path_len == 0 {
		None
	}
	else {
		let bytes = unsafe { std::slice::from_raw_parts(path_ptr, path_len) };
		Some(String::from_utf8_lossy(bytes).into_owned())
	};
	let bytes = if bytes_ptr.is_null() || bytes_len == 0 {
		None
	}
	else {
		Some(unsafe { std::slice::from_raw_parts(bytes_ptr, bytes_len) }.to_vec())
	};
	unsafe {
		(**ctx).file_opened(request_id, path, bytes);
	}
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn allocate(nbytes: usize) -> *mut u8 {
	let v = vec![0u64; (nbytes + 7) / 8].into_boxed_slice();
	Box::into_raw(v) as *mut u8
}

#[cfg_attr(target_family = "wasm", no_mangle)]
pub extern "C" fn free(p: *mut u8, nbytes: usize) {
	if p.is_null() {
		return;
	}
	unsafe {
		let p = ptr::slice_from_raw_parts_mut(p as *mut u64, (nbytes + 7) / 8);
		let _ = Box::from_raw(p);
	}
}
