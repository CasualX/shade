use std::{fs, io, path};
use cvmath::*;
use crate::*;

#[derive(Clone, Debug)]
pub struct ObjData {
	pub vertices: Vec<d3::TexturedVertexN>,
	pub bounds: Bounds3f,
}

impl ObjData {
	/// Loads an OBJ file from a path.
	#[inline]
	pub fn load_file(path: impl AsRef<path::Path>) -> io::Result<ObjData> {
		let file = fs::File::open(path)?;
		let stream = io::BufReader::new(file);
		Self::load_stream(stream)
	}

	/// Parses OBJ data from a reader.
	#[inline]
	pub fn load_stream(mut stream: impl io::Read) -> io::Result<ObjData> {
		let mut source = String::new();
		stream.read_to_string(&mut source)?;
		parse(&source)
	}

	/// Parses OBJ data from memory.
	#[inline]
	pub fn load_memory(data: &[u8]) -> io::Result<ObjData> {
		let source = std::str::from_utf8(data)
			.map_err(|error| invalid_data(format!("OBJ is not UTF-8: {error}")))?;
		parse(source)
	}
}

#[derive(Copy, Clone)]
struct FaceVertex {
	position: usize,
	uv: Option<usize>,
	normal: Option<usize>,
}

fn parse(source: &str) -> io::Result<ObjData> {
	let mut positions = Vec::new();
	let mut uvs = Vec::new();
	let mut normals = Vec::new();
	let mut data = ObjData {
		vertices: Vec::new(),
		bounds: Bounds3f::EMPTY,
	};

	for (line_index, line) in source.lines().enumerate() {
		let line_number = line_index + 1;
		let line = line.split_once('#').map_or(line, |(line, _)| line).trim();
		if line.is_empty() {
			continue;
		}

		let mut fields = line.split_ascii_whitespace();
		match fields.next() {
			Some("v") => positions.push(Vec3(
				parse_float(fields.next(), line_number, "vertex x")?,
				parse_float(fields.next(), line_number, "vertex y")?,
				parse_float(fields.next(), line_number, "vertex z")?,
			)),
			Some("vt") => uvs.push(Vec2(
				parse_float(fields.next(), line_number, "texture u")?,
				parse_float(fields.next(), line_number, "texture v")?,
			)),
			Some("vn") => normals.push(Vec3(
				parse_float(fields.next(), line_number, "normal x")?,
				parse_float(fields.next(), line_number, "normal y")?,
				parse_float(fields.next(), line_number, "normal z")?,
			)),
			Some("f") => {
				let face = fields
					.map(|field| parse_face_vertex(field, line_number, positions.len(), uvs.len(), normals.len()))
					.collect::<io::Result<Vec<_>>>()?;
				if face.len() < 3 {
					return Err(line_error(line_number, "face has fewer than three vertices"));
				}

				for index in 1..face.len() - 1 {
					for vertex in [face[0], face[index], face[index + 1]] {
						let pos = positions[vertex.position];
						data.vertices.push(d3::TexturedVertexN {
							pos,
							normal: vertex.normal.map_or(Vec3f::ZERO, |index| normals[index]),
							uv: vertex.uv.map_or(Vec2f::ZERO, |index| uvs[index]),
						});
						data.bounds = data.bounds.include(pos);
					}
				}
			},
			_ => {},
		}
	}

	Ok(data)
}

fn parse_float(value: Option<&str>, line: usize, field: &str) -> io::Result<f32> {
	value
		.ok_or_else(|| line_error(line, format!("missing {field}")))?
		.parse()
		.map_err(|_| line_error(line, format!("invalid {field}")))
}

fn parse_face_vertex(value: &str, line: usize, positions: usize, uvs: usize, normals: usize) -> io::Result<FaceVertex> {
	let mut indices = value.split('/');
	let position = parse_index(indices.next().unwrap_or_default(), positions, line, "position")?;
	let uv = parse_optional_index(indices.next(), uvs, line, "texture coordinate")?;
	let normal = parse_optional_index(indices.next(), normals, line, "normal")?;
	if indices.next().is_some() {
		return Err(line_error(line, "face vertex has too many indices"));
	}
	Ok(FaceVertex { position, uv, normal })
}

fn parse_optional_index(value: Option<&str>, len: usize, line: usize, field: &str) -> io::Result<Option<usize>> {
	match value {
		None | Some("") => Ok(None),
		Some(value) => parse_index(value, len, line, field).map(Some),
	}
}

fn parse_index(value: &str, len: usize, line: usize, field: &str) -> io::Result<usize> {
	let index = value
		.parse::<isize>()
		.map_err(|_| line_error(line, format!("invalid {field} index")))?;
	let index = match index {
		1.. => index - 1,
		..=-1 => len as isize + index,
		0 => return Err(line_error(line, format!("{field} index cannot be zero"))),
	};
	if index < 0 || index as usize >= len {
		return Err(line_error(line, format!("{field} index is out of bounds")));
	}
	Ok(index as usize)
}

fn line_error(line: usize, message: impl std::fmt::Display) -> io::Error {
	invalid_data(format!("OBJ line {line}: {message}"))
}

fn invalid_data(message: impl Into<String>) -> io::Error {
	io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_and_triangulates_faces() {
		let obj = ObjData::load_memory(b"\
v 0 0 0
v 1 0 0
v 1 1 0
v 0 1 0
vt 0 0
vt 1 0
vt 1 1
vt 0 1
vn 0 0 1
f 1/1/1 2/2/1 3/3/1 4/4/1
").unwrap();

		assert_eq!(obj.vertices.len(), 6);
		assert_eq!(obj.vertices[0].pos, Vec3(0.0, 0.0, 0.0));
		assert_eq!(obj.vertices[5].uv, Vec2(0.0, 1.0));
		assert_eq!(obj.bounds, Bounds3f(Vec3(0.0, 0.0, 0.0), Vec3(1.0, 1.0, 0.0)));
	}

	#[test]
	fn supports_negative_and_missing_indices() {
		let obj = ObjData::load_memory(b"\
v 0 0 0
v 1 0 0
v 0 1 0
f -3 -2 -1
").unwrap();

		assert_eq!(obj.vertices.len(), 3);
		assert_eq!(obj.vertices[0].normal, Vec3f::ZERO);
		assert_eq!(obj.vertices[0].uv, Vec2f::ZERO);
	}
}
