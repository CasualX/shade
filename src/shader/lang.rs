use std::fmt::Write as _;
use std::collections::HashSet;

use super::*;

/// Rewrites shader source one directive line at a time while preserving source locations.
///
/// For each line whose first character is `#`, `replacer` receives the full line and appends replacement text into the output buffer.
/// Non-directive lines are copied through unchanged.
///
/// After each replaced directive, a `#line` directive is emitted so compiler diagnostics map back to the original source line and logical source `index`.
pub fn preprocess(source: &str, index: usize, replacer: &mut dyn FnMut(&str, &mut String), result: &mut String) {
	for (i, line) in source.lines().enumerate() {
		if line.starts_with('#') {
			replacer(line, result);
			let line = i + 2; // #line applies to the following physical line.
			_ = write!(result, "\n#line {line} {index}\n");
		}
		else {
			result.push_str(line);
			result.push('\n');
		}
	}
}

fn parse_include_name(line: &str) -> Option<&str> {
	let line = line.trim_ascii();
	let include = line.strip_prefix("#include")?.trim_ascii();
	include.strip_prefix('"')?.strip_suffix('"')
}

fn compile_shader_source_impl(
	name: &str,
	source: &str,
	index: usize,
	next_index: &mut usize,
	once: &mut HashSet<String>,
	kind: ShaderKind,
	version: &str,
	backend: &str,
	defines: &[ShaderDefine<'_>],
	interface: &mut dyn IShaderInterface,
	result: &mut String,
) {
	preprocess(source, index, &mut |line, output| {
		let line = line.trim_ascii();
		if line.starts_with("#version") {
			_ = writeln!(output, "#version {version}");
			_ = writeln!(output, "#define {backend}");
			match kind {
				ShaderKind::Vertex => {
					output.push_str("#define VERTEX_SHADER\n");
					output.push_str("#define VARYING out\n");
				}
				ShaderKind::Fragment => {
					output.push_str("#define FRAGMENT_SHADER\n");
					output.push_str("#define VARYING in\n");
				}
			}
			for def in defines {
				_ = match def.value {
					Some(val) => writeln!(output, "#define {} {}", def.name, val),
					None => writeln!(output, "#define {}", def.name),
				};
			}
		}
		else if line.starts_with("#include") {
			match parse_include_name(line) {
				Some(include_name) => {
					if once.contains(include_name) {
						return;
					}
					match interface.include_source(include_name) {
						Ok(source) => {
							*next_index += 1;
							let include_index = *next_index;
							compile_shader_source_impl(include_name, &source, include_index, next_index, once, kind, version, backend, defines, interface, output)
						}
						Err(err) => _ = writeln!(output, "#error Failed to include shader source: {err}"),
					}
				}
				None => output.push_str("#error Invalid include directive\n"),
			}
		}
		else if line == "#pragma once" {
			once.insert(name.to_owned());
		}
		else {
			output.push_str(line);
		}
	}, result);
}

/// Builds backend-specific shader source for `shader_compile`.
///
/// The root shader is loaded from `name` through `interface`. `#version` is
/// replaced with the backend-specific version plus the stage and backend
/// defines. `#include "..."` is expanded through `interface` and processed with
/// the same rules recursively.
pub fn compile(name: &str, index: usize, kind: ShaderKind, version: &str, backend: &str, defines: &[ShaderDefine<'_>], interface: &mut dyn IShaderInterface) -> String {
	let source = match interface.include_source(name) {
		Ok(source) => source,
		Err(err) => return format!("#version\n#error Failed to include shader source: {err}\n"),
	};

	let mut result = String::new();
	let mut next_index = index;
	let mut once = HashSet::new();
	compile_shader_source_impl(name, &source, index, &mut next_index, &mut once, kind, version, backend, defines, interface, &mut result);
	result
}
