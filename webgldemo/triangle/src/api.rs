use std::ptr;

use crate::Context;

#[no_mangle]
pub extern "C" fn new() -> *mut Context {
	let ctx = Context::new();
	Box::into_raw(Box::new(ctx))
}

#[no_mangle]
pub extern "C" fn drop(ctx: *mut Context) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		let _ = Box::from_raw(ctx);
	}
}

#[no_mangle]
pub extern "C" fn draw(ctx: *mut Context, time: f64) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(*ctx).draw(time);
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
