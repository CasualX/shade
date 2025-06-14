use std::{fmt, panic, ptr};

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
pub extern "C" fn resize(ctx: *mut Context, width: i32, height: i32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(*ctx).resize(width, height);
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
pub extern "C" fn update_camera(ctx: *mut Context, delta_x: f32, delta_y: f32, delta_zoom: f32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		// Special case for reset
		if delta_zoom < -9000.0 {
			(*ctx).camera.reset();
		} else {
			(*ctx).camera.update_from_input(delta_x, delta_y, delta_zoom);
		}
	}
}

#[no_mangle]
pub extern "C" fn pan_camera(ctx: *mut Context, delta_x: f32, delta_y: f32) {
	if ctx.is_null() {
		return;
	}
	unsafe {
		(*ctx).camera.pan(delta_x, delta_y);
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

pub fn setup_panic_hook() {
	panic::set_hook(Box::new(|info| {
		let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
			*s
		}
		else if let Some(s) = info.payload().downcast_ref::<String>() {
			s.as_str()
		}
		else {
			"Unknown panic payload"
		};

		struct DisplayLocation<'a>(&'a panic::PanicHookInfo<'a>);
		impl<'a> fmt::Display for DisplayLocation<'a> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				let info = self.0;
				if let Some(loc) = info.location() {
					write!(f, "{}:{}:{}", loc.file(), loc.line(), loc.column())
				}
				else {
					f.write_str("unknown location")
				}
			}
		}

		shade::webgl::log(format_args!("Panic at {}: {}", DisplayLocation(info), message));
	}));
}
