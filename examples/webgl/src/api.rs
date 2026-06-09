use std::ptr;

use super::*;

fn into_handle<T: crate::DemoContext + 'static>(ctx: T) -> *mut DemoHandle {
	Box::into_raw(Box::new(Box::new(ctx)))
}

#[no_mangle]
pub extern "C" fn new_triangle() -> *mut DemoHandle {
	into_handle(triangle::Triangle::new())
}

#[no_mangle]
pub extern "C" fn new_oldtree() -> *mut DemoHandle {
	into_handle(oldtree::Context::new())
}

#[no_mangle]
pub extern "C" fn new_text() -> *mut DemoHandle {
	into_handle(text::Context::new())
}

#[no_mangle]
pub extern "C" fn new_text3d() -> *mut DemoHandle {
	into_handle(text3d::Context::new())
}

#[no_mangle]
pub extern "C" fn new_textintro() -> *mut DemoHandle {
	into_handle(textintro::Context::new())
}

#[no_mangle]
pub extern "C" fn new_pixelart() -> *mut DemoHandle {
	into_handle(pixelart::Context::new())
}

#[no_mangle]
pub extern "C" fn new_zeldawater() -> *mut DemoHandle {
	into_handle(zeldawater::Context::new())
}

#[no_mangle]
pub extern "C" fn new_globe() -> *mut DemoHandle {
	into_handle(globe::Context::new())
}

#[no_mangle]
pub extern "C" fn drop(ctx: *mut DemoHandle) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		let _ = Box::from_raw(ctx);
	}
}

#[no_mangle]
pub extern "C" fn resize(ctx: *mut DemoHandle, width: i32, height: i32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).resize(width, height);
	}
}

#[no_mangle]
pub extern "C" fn mousemove(ctx: *mut DemoHandle, dx: f32, dy: f32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).mousemove(dx, dy);
	}
}

#[no_mangle]
pub extern "C" fn mousedown(ctx: *mut DemoHandle, button: u32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).mousedown(button);
	}
}

#[no_mangle]
pub extern "C" fn mouseup(ctx: *mut DemoHandle, button: u32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).mouseup(button);
	}
}

#[no_mangle]
pub extern "C" fn wheel(ctx: *mut DemoHandle, delta_y: f32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).wheel(delta_y);
	}
}

#[no_mangle]
pub extern "C" fn keydown(ctx: *mut DemoHandle, key: u32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).keydown(key);
	}
}

#[no_mangle]
pub extern "C" fn draw(ctx: *mut DemoHandle, time: f64) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(**ctx).draw(time);
	}
}

#[no_mangle]
pub extern "C" fn allocate(nbytes: usize) -> *mut u8 {
	let v = vec![0u64; (nbytes + 7) / 8].into_boxed_slice();
	Box::into_raw(v) as *mut u8
}

#[no_mangle]
pub extern "C" fn free(p: *mut u8, nbytes: usize) {
	if p.is_null() {
		return;
	}
	unsafe {
		let p = ptr::slice_from_raw_parts_mut(p as *mut u64, (nbytes + 7) / 8);
		let _ = Box::from_raw(p);
	}
}
