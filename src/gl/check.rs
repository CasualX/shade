//! OpenGL error checking
#![macro_use]

#[cfg(debug_assertions)]
macro_rules! gl_check {
	(gl::$f:ident($($e:expr),* $(,)?)) => {{
		assert!(gl::$f::is_loaded(), "OpenGL function gl{} was not loaded.", stringify!($f));
		let result = unsafe { ::gl::$f($($e),*) };
		$crate::gl::check::check(result, || {
			eprintln!(concat!("gl", stringify!($f), "(", $(stringify!($e), "={:?}, ",)* ")")$(, $e)*);
		})
	}};
}

#[cfg(not(debug_assertions))]
macro_rules! gl_check {
	(gl::$f:ident($($e:expr),* $(,)?)) => {
		unsafe { ::gl::$f($($e),*) }
	};
}

#[cfg(debug_assertions)]
#[doc(hidden)]
#[inline]
#[track_caller]
pub fn check<T, F: FnOnce()>(result: T, on_error: F) -> T {
	let caller = std::panic::Location::caller();
	let error = unsafe { gl::GetError() };
	if error != gl::NO_ERROR {
		eprintln!("OpenGL error {caller}: {error:X}");
		on_error();
		panic!("OpenGL check failed");
	}

	result
}
