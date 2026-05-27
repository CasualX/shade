use std::{io, path};

pub(crate) mod lang;

define_handle!(ShaderProgram);

/// Shader kind.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ShaderKind {
	/// Vertex shader.
	Vertex,
	/// Fragment shader.
	Fragment,
}

/// Preprocessor definition for unified shader compilation.
///
/// `name` is emitted as `#define NAME` when `value` is `None`, or as
/// `#define NAME VALUE` when `value` is present.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ShaderDefine<'a> {
	pub name: &'a str,
	pub value: Option<&'a str>,
}

/// Abstract shader source loader.
pub trait IShaderInterface {
	fn include_source(&mut self, name: &str) -> io::Result<String>;
}

/// Filesystem-backed shader source loader.
///
/// Resolves names relative to `base_path`.
#[derive(Clone, Debug)]
pub struct ShaderInterfaceIO {
	base_path: path::PathBuf,
}
impl ShaderInterfaceIO {
	/// Creates a new instance.
	#[inline]
	pub fn new(base_path: path::PathBuf) -> ShaderInterfaceIO {
		ShaderInterfaceIO { base_path }
	}
}
impl IShaderInterface for ShaderInterfaceIO {
	fn include_source(&mut self, name: &str) -> io::Result<String> {
		if name.contains("..") {
			return Err(io::Error::from(io::ErrorKind::InvalidInput));
		}
		std::fs::read_to_string(self.base_path.join(name))
	}
}

/// Creates a lightweight in-memory [`IShaderInterface`] implementation.
///
/// Example:
///
/// ```rust
/// let mut interface = shade::shader_interface! {
/// 	files {
/// 		"main.glsl" => "#version unified 330 core, 300 es\nvoid main() {}\n",
/// 	}
/// };
/// ```
#[macro_export]
macro_rules! shader_interface {
	(files { $($name:expr => $source:expr),* $(,)? }) => {{
		struct StaticShaderInterface {
			_private: (),
		}
		impl $crate::IShaderInterface for StaticShaderInterface {
			fn include_source(&mut self, name: &str) -> ::std::io::Result<String> {
				let source = match name {
					$($name => $source,)*
					_ => return Err(::std::io::Error::from(::std::io::ErrorKind::NotFound)),
				};
				Ok(String::from(source))
			}
		}
		StaticShaderInterface {
			_private: (),
		}
	}};
}

#[cfg(test)]
mod tests;
