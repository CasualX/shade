
/// Macro to include binary files at compile time.
///
/// ```
/// shade::include_bin!(pub DATA: [u16] = "include_bin.rs");
/// ```
#[macro_export]
macro_rules! include_bin {
	($vis:vis $name:ident: [$ty:ty] = $path:expr) => {
		$vis static $name: [$ty; include_bytes!($path).len() / ::core::mem::size_of::<$ty>()] = {
			fn __assert_pod<T: ::dataview::Pod>() {}
			let _ = __assert_pod::<$ty>;
			unsafe { ::core::mem::transmute(*include_bytes!($path)) }
		};
	};
}
