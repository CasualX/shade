//! GLSL 3.30 core shader sources.

pub const COLOR_VS: &str = include_str!("glsl330core/color.vs.glsl");
pub const COLOR_FS: &str = include_str!("glsl330core/color.fs.glsl");

pub const GRADIENT_VS: &str = include_str!("glsl330core/gradient.vs.glsl");
pub const GRADIENT_FS: &str = include_str!("glsl330core/gradient.fs.glsl");

pub const TEXTURED_VS: &str = include_str!("glsl330core/textured.vs.glsl");
pub const TEXTURED_FS: &str = include_str!("glsl330core/textured.fs.glsl");

pub const MTSDF_VS: &str = include_str!("glsl330core/mtsdf.vs.glsl");
pub const MTSDF_FS: &str = include_str!("glsl330core/mtsdf.fs.glsl");

pub const COLOR3D_VS: &str = include_str!("glsl330core/color3d.vs.glsl");
pub const COLOR3D_FS: &str = include_str!("glsl330core/color3d.fs.glsl");

pub const POST_PROCESS_VS: &str = include_str!("glsl330core/post_process.vs.glsl");
pub const POST_PROCESS_COPY_FS: &str = include_str!("glsl330core/post_process.copy.fs.glsl");
pub const POST_PROCESS_MELT_FS: &str = include_str!("glsl330core/post_process.melt.fs.glsl");
