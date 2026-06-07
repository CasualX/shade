use super::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StlColorFormat {
	RGB15,
	Materialise,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct StlTriangle {
	pub normal: Vec3f,
	pub v1: Vec3f,
	pub v2: Vec3f,
	pub v3: Vec3f,
	pub attribute: u16,
}

#[derive(Clone, Debug)]
pub struct StlMaterial {
	pub color: Vec4<u8>,
	pub diffuse: Vec4<u8>,
	pub specular: Vec4<u8>,
	pub ambient: Vec4<u8>,
}

#[derive(Clone, Debug)]
pub struct StlData {
	pub vertices: Vec<d3::ColorVertexN>,
	pub bounds: Bounds3f,
}

#[derive(Clone, Debug)]
pub struct StlIndexedData {
	pub vertices: Vec<d3::ColorVertexN>,
	pub indices: Vec<Index3>,
	pub bounds: Bounds3f,
}

#[derive(Clone, Debug)]
pub struct StlHeader {
	pub header: [u8; 80],
}

/// Stl data.
#[derive(Clone, Debug)]
pub struct StlFile {
	pub header: StlHeader,
	pub faces: Vec<StlTriangle>,
}

impl StlFile {
	/// Loads a binary STL file from a path.
	#[inline]
	pub fn load_file(path: impl AsRef<path::Path>) -> io::Result<StlFile> {
		let file = fs::File::open(path)?;
		let stream = io::BufReader::new(file);
		read_binary(stream, false)
	}
	/// Parses a binary STL file from a reader.
	#[inline]
	pub fn load_stream(stream: impl io::Read) -> io::Result<StlFile> {
		read_binary(stream, false)
	}
	/// Parses a binary STL file from memory.
	#[inline]
	pub fn load_memory(mut data: &[u8]) -> io::Result<StlFile> {
		read_binary(&mut data, false)
	}

	/// Returns one vertex per face corner, preserving the face normals.
	pub fn vertices(&self, color_format: Option<StlColorFormat>) -> StlData {
		let mut data = StlData {
			vertices: Vec::new(),
			bounds: Bounds3f::EMPTY,
		};

		for face in &self.faces {
			let color = color(color_format, face.attribute, Vec4::dup(255));
			for position in [face.v1, face.v2, face.v3] {
				data.vertices.push(d3::ColorVertexN { pos: position, normal: face.normal, color: color.into() });
				data.bounds = data.bounds.include(position);
			}
		}

		data
	}

	/// Returns indexed vertices with normals averaged across shared positions.
	pub fn smooth_vertices(&self, _color_format: Option<StlColorFormat>) -> StlIndexedData {
		let mut lookup = HashMap::new();
		let mut data = StlIndexedData {
			vertices: Vec::new(),
			indices: Vec::new(),
			bounds: Bounds3f::EMPTY,
		};

		fn vertex(lookup: &mut HashMap<Vec3<u32>, u32>, stl: &mut StlIndexedData, v: &Vec3f, n: &Vec3f) -> u32 {
			let index = *lookup.entry(v.map(f32::to_bits)).or_insert_with(|| {
				let index = stl.vertices.len() as u32;
				stl.vertices.push(d3::ColorVertexN { pos: *v, normal: Vec3f::ZERO, color: [255; 4] });
				stl.bounds = stl.bounds.include(*v);
				index
			});
			stl.vertices[index as usize].normal += *n;
			index
		}

		for face in &self.faces {
			let p1 = vertex(&mut lookup, &mut data, &face.v1, &face.normal);
			let p2 = vertex(&mut lookup, &mut data, &face.v2, &face.normal);
			let p3 = vertex(&mut lookup, &mut data, &face.v3, &face.normal);
			data.indices.push(Index3 { p1, p2, p3 });
		}

		for vertex in &mut data.vertices {
			if vertex.normal != Vec3f::ZERO {
				vertex.normal = vertex.normal.norm();
			}
		}

		data
	}
}

fn color(format: Option<StlColorFormat>, attribute: u16, default: Vec4<u8>) -> Vec4<u8> {
	match format {
		Some(StlColorFormat::RGB15) => {
			let c = attribute & 0x7FFF;
			let red = ((c >> 10) as u8) << 3;
			let green = (((c >> 5) & 0x1F) as u8) << 3;
			let blue = ((c & 0x1F) as u8) << 3;
			Vec4(red, green, blue, 255)
		},
		Some(StlColorFormat::Materialise) => {
			if (attribute & 0x8000) != 0 {
				default
			}
			else {
				let red = ((attribute & 0x1F) as u8) << 3;
				let green = (((attribute >> 5) & 0x1F) as u8) << 3;
				let blue = (((attribute >> 10) & 0x1F) as u8) << 3;
				Vec4(red, green, blue, 255)
			}
		},
		None => default,
	}
}

fn read_binary<R: io::Read>(mut reader: R, recompute_normals: bool) -> io::Result<StlFile> {
	let mut header = [0; 80];
	let mut count = [0; 4];
	reader.read_exact(&mut header)?;
	reader.read_exact(&mut count)?;
	let count = u32::from_le_bytes(count);

	let mut faces: Vec<StlTriangle> = Vec::with_capacity(count as usize);
	let mut data = [0; 50];
	for _ in 0..count {
		reader.read_exact(&mut data)?;

		let data = dataview::DataView::from(&data);
		let mut normal = data.read::<Vec3f>(0);
		let v1 = data.read::<Vec3f>(12);
		let v2 = data.read::<Vec3f>(24);
		let v3 = data.read::<Vec3f>(36);
		let attribute = data.read::<u16>(48);

		if recompute_normals || normal == Vec3f::ZERO {
			let edge1 = v2 - v1;
			let edge2 = v3 - v1;
			normal = edge1.cross(edge2).norm();
		}

		faces.push(StlTriangle { normal, v1, v2, v3, attribute });
	}
	let header = StlHeader { header };
	Ok(StlFile { header, faces })
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rgb15_decodes_viscam_channel_order() {
		let color = color(Some(StlColorFormat::RGB15), 0xFC00, Vec4::dup(0));
		assert_eq!(color, Vec4(248, 0, 0, 255));
	}

	#[test]
	fn materialise_decodes_facet_color_from_u16_attribute() {
		let color = color(Some(StlColorFormat::Materialise), 0x7C1F, Vec4::dup(0));
		assert_eq!(color, Vec4(248, 0, 248, 255));
	}

	#[test]
	fn materialise_uses_default_when_object_color_flag_is_set() {
		let default = Vec4(1, 2, 3, 4);
		let color = color(Some(StlColorFormat::Materialise), 0x8000, default);
		assert_eq!(color, default);
	}
}
