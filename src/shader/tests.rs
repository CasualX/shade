use super::*;
use std::collections::HashMap;

struct TestShaderInterface {
	files: HashMap<&'static str, &'static str>,
}

impl TestShaderInterface {
	fn new(files: &[(&'static str, &'static str)]) -> Self {
		let files = files.iter().copied().collect();
		Self { files }
	}
}

impl IShaderInterface for TestShaderInterface {
	fn include_source(&mut self, name: &str) -> io::Result<String> {
		self.files
			.get(name)
			.copied()
			.map(String::from)
			.ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
	}
}

#[test]
fn compile_shader_source_injects_stage_and_backend_defines() {
	let mut interface = TestShaderInterface::new(&[("main.glsl", "#version unified 330 core, 300 es\n#ifdef VERTEX_SHADER\nfloat x = 1.0;\n#endif\n")]);
	let output = lang::compile(
		"main.glsl",
		0,
		ShaderKind::Vertex,
		"330 core",
		"GLSL_CORE",
		&[ShaderDefine { name: "TEST_DEFINE", value: Some("7") }],
		&mut interface,
	);

	assert!(output.contains("#version 330 core\n#define GLSL_CORE\n#define VERTEX_SHADER\n#define VARYING out\n#define TEST_DEFINE 7\n"));
	assert!(output.contains("#ifdef VERTEX_SHADER\n#line 3 0\n"));
}

#[test]
fn compile_shader_source_expands_nested_includes() {
	let mut interface = TestShaderInterface::new(&[
		("main.glsl", "#version unified 330 core\n#include \"shared.glsl\"\nvoid main() {}\n"),
		("shared.glsl", "#include \"math.glsl\"\nfloat shared_value = compute_value();\n"),
		("math.glsl", "float compute_value() { return 1.0; }\n"),
	]);
	let output = lang::compile("main.glsl", 0, ShaderKind::Fragment, "330 core", "GLSL_CORE", &[], &mut interface);

	assert!(output.contains("float compute_value() { return 1.0; }"));
	assert!(output.contains("#line 2 1"));
	assert!(output.contains("float shared_value = compute_value();"));
	assert!(output.contains("#line 3 0"));
	assert!(output.contains("void main() {}"));
}

#[test]
fn compile_shader_source_reports_missing_include() {
	let mut interface = TestShaderInterface::new(&[("main.glsl", "#version unified 330 core\n#include \"missing.glsl\"\nvoid main() {}\n")]);
	let output = lang::compile("main.glsl", 0, ShaderKind::Fragment, "330 core", "GLSL_CORE", &[], &mut interface);

	assert!(output.contains("#error Failed to include shader source"));
}

#[test]
fn compile_shader_source_reports_missing_root_source() {
	let mut interface = TestShaderInterface::new(&[]);
	let output = lang::compile("main.glsl", 0, ShaderKind::Fragment, "330 core", "GLSL_CORE", &[], &mut interface);

	assert!(output.contains("#error Failed to include shader source"));
}

#[test]
fn compile_shader_source_skips_reincluded_pragma_once_files() {
	let mut interface = TestShaderInterface::new(&[
		("main.glsl", "#version unified 330 core\n#include \"shared.glsl\"\n#include \"shared.glsl\"\nvoid main() {}\n"),
		("shared.glsl", "#pragma once\nfloat shared_value = 1.0;\n"),
	]);
	let output = lang::compile("main.glsl", 0, ShaderKind::Fragment, "330 core", "GLSL_CORE", &[], &mut interface);

	assert_eq!(output.matches("float shared_value = 1.0;").count(), 1);
	assert!(output.contains("void main() {}"));
}
