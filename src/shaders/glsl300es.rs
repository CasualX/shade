//! GLSL ES 3.00 shader sources.

pub const COLOR_VS: &str = include_str!("glsl300es/color.vs.glsl");
pub const COLOR_FS: &str = include_str!("glsl300es/color.fs.glsl");

pub const GRADIENT_VS: &str = include_str!("glsl300es/gradient.vs.glsl");
pub const GRADIENT_FS: &str = include_str!("glsl300es/gradient.fs.glsl");

pub const TEXTURED_VS: &str = include_str!("glsl300es/textured.vs.glsl");
pub const TEXTURED_FS: &str = include_str!("glsl300es/textured.fs.glsl");

pub const MTSDF_VS: &str = include_str!("glsl300es/mtsdf.vs.glsl");
pub const MTSDF_FS: &str = include_str!("glsl300es/mtsdf.fs.glsl");

pub const COLOR3D_VS: &str = include_str!("glsl300es/color3d.vs.glsl");
pub const COLOR3D_FS: &str = include_str!("glsl300es/color3d.fs.glsl");

pub const POST_PROCESS_VS: &str = include_str!("glsl300es/post_process.vs.glsl");
pub const POST_PROCESS_COPY_FS: &str = include_str!("glsl300es/post_process.copy.fs.glsl");
pub const POST_PROCESS_MELT_FS: &str = include_str!("glsl300es/post_process.melt.fs.glsl");
