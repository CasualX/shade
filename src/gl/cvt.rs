use super::*;

pub fn gl_depth_func(v: crate::Compare) -> GLenum {
	match v {
		crate::Compare::Never => gl::NEVER,
		crate::Compare::Less => gl::LESS,
		crate::Compare::Equal => gl::EQUAL,
		crate::Compare::LessEqual => gl::LEQUAL,
		crate::Compare::Greater => gl::GREATER,
		crate::Compare::NotEqual => gl::NOTEQUAL,
		crate::Compare::GreaterEqual => gl::GEQUAL,
		crate::Compare::Always => gl::ALWAYS,
	}
}
