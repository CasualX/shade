use super::*;
use std::f32::consts::*;

fn write_bin<T: dataview::Pod>(path: &std::path::Path, slice: &[T]) {
	std::fs::write(path, dataview::bytes(slice)).unwrap();
}

/// Icosahedron with vertex-up orientation.
/// - Vertex 0 is at the north pole (0, 1, 0)
/// - Vertex 11 is at the south pole (0, -1, 0)
/// - Vertices 1-5 form the upper ring
/// - Vertices 6-10 form the lower ring
/// - One vertex (index 1) lies on the +Z axis
fn icosahedron_base() -> ([Vec3f; 12], [[u8; 3]; 20]) {
	// Latitude of the upper and lower rings
	let lat = (0.5_f32).atan(); // arctan(1/2) ≈ 26.565°

	let y_top = lat.sin();
	let r_top = lat.cos();
	let y_bot = -y_top;
	let r_bot = r_top;

	let mut positions = [Vec3f::ZERO; 12];

	// North pole
	positions[0] = Vec3f(0.0, 1.0, 0.0);

	// Upper ring (5 vertices) - first one on +Z axis
	for i in 0..5 {
		let angle = (i as f32) * 2.0 * PI / 5.0;
		positions[1 + i] = Vec3f(r_top * angle.sin(), y_top, r_top * angle.cos());
	}

	// Lower ring (5 vertices) - offset by 36° (half step)
	for i in 0..5 {
		let angle = (i as f32 + 0.5) * 2.0 * PI / 5.0;
		positions[6 + i] = Vec3f(r_bot * angle.sin(), y_bot, r_bot * angle.cos());
	}

	// South pole
	positions[11] = Vec3f(0.0, -1.0, 0.0);

	// Normalize all positions to unit sphere
	for p in &mut positions {
		*p = p.norm();
	}

	// 20 triangles with CCW winding (outward-facing)
	let triangles: [[u8; 3]; 20] = [
		// 5 triangles around north pole
		[0, 1, 2],
		[0, 2, 3],
		[0, 3, 4],
		[0, 4, 5],
		[0, 5, 1],
		// 10 triangles in the middle band
		[1, 6, 2],
		[2, 6, 7],
		[2, 7, 3],
		[3, 7, 8],
		[3, 8, 4],
		[4, 8, 9],
		[4, 9, 5],
		[5, 9, 10],
		[5, 10, 1],
		[1, 10, 6],
		// 5 triangles around south pole
		[11, 7, 6],
		[11, 8, 7],
		[11, 9, 8],
		[11, 10, 9],
		[11, 6, 10],
	];

	(positions, triangles)
}

/// Convert a normalized direction to spherical UV coordinates.
/// U wraps around the sphere horizontally (longitude), V goes from bottom to top (latitude).
fn spherical_uv(dir: Vec3f) -> Vec2f {
	let u = 0.5 + dir.z.atan2(dir.x) / (2.0 * PI);
	let v = 0.5 + dir.y.asin() / PI;
	Vec2f(u, v)
}

/// Check if a vertex is at a pole (on the Y axis, where U is undefined).
fn is_pole(pos: Vec3f) -> bool {
	pos.x.abs() < 1e-6 && pos.z.abs() < 1e-6
}

/// Compute UVs for a triangle, handling seam wrap-around and pole vertices.
fn triangle_uvs(a: Vec3f, b: Vec3f, c: Vec3f) -> [Vec2f; 3] {
	let mut uva = spherical_uv(a);
	let mut uvb = spherical_uv(b);
	let mut uvc = spherical_uv(c);

	let a_pole = is_pole(a);
	let b_pole = is_pole(b);
	let c_pole = is_pole(c);

	// Collect U values from non-pole vertices
	let non_pole_us: Vec<f32> = [
		if !a_pole { Some(uva.x) } else { None },
		if !b_pole { Some(uvb.x) } else { None },
		if !c_pole { Some(uvc.x) } else { None },
	]
	.into_iter()
	.flatten()
	.collect();

	// Fix seam wrap-around for non-pole vertices
	if non_pole_us.len() >= 2 {
		let min_u = non_pole_us.iter().cloned().reduce(f32::min).unwrap();
		let max_u = non_pole_us.iter().cloned().reduce(f32::max).unwrap();

		if max_u - min_u > 0.5 {
			// Triangle crosses the seam - add 1.0 to low-U vertices
			if !a_pole && uva.x < 0.5 {
				uva.x += 1.0;
			}
			if !b_pole && uvb.x < 0.5 {
				uvb.x += 1.0;
			}
			if !c_pole && uvc.x < 0.5 {
				uvc.x += 1.0;
			}
		}
	}

	// For pole vertices, set U to average of the (adjusted) non-pole U values
	if a_pole || b_pole || c_pole {
		let mut sum = 0.0;
		let mut count = 0;
		if !a_pole {
			sum += uva.x;
			count += 1;
		}
		if !b_pole {
			sum += uvb.x;
			count += 1;
		}
		if !c_pole {
			sum += uvc.x;
			count += 1;
		}
		let avg_u = if count > 0 { sum / count as f32 } else { 0.5 };

		if a_pole {
			uva.x = avg_u;
		}
		if b_pole {
			uvb.x = avg_u;
		}
		if c_pole {
			uvc.x = avg_u;
		}
	}

	[uva, uvb, uvc]
}

/// Generate smooth-shaded icosahedron.
/// Returns vertices and indices. Vertices are duplicated where needed for UV seams and poles.
fn generate_smooth() -> (Vec<TexturedVertexN>, Vec<u8>) {
	let (positions, triangles) = icosahedron_base();

	// For smooth shading, we want to share vertices where possible.
	// However, vertices need to be duplicated when:
	// 1. They're on the UV seam (need different U for different triangles)
	// 2. They're at a pole (U depends on the triangle)
	//
	// Strategy: for each triangle, compute the required UVs, then deduplicate
	// vertices that have the same position and UV.

	let mut vertices: Vec<TexturedVertexN> = Vec::new();
	let mut indices: Vec<u8> = Vec::new();

	// Map from (position_index, quantized_u) to vertex index for deduplication
	// We quantize U to handle floating point comparison
	use std::collections::HashMap;
	let mut vertex_map: HashMap<(u8, i32, i32), u8> = HashMap::new();

	for tri in triangles {
		let pa = positions[tri[0] as usize];
		let pb = positions[tri[1] as usize];
		let pc = positions[tri[2] as usize];
		let uvs = triangle_uvs(pa, pb, pc);

		for (i, &pos_idx) in tri.iter().enumerate() {
			let pos = positions[pos_idx as usize];
			let uv = uvs[i];

			// Quantize UV for deduplication (to ~0.001 precision)
			let qu = (uv.x * 1000.0).round() as i32;
			let qv = (uv.y * 1000.0).round() as i32;
			let key = (pos_idx, qu, qv);

			let vert_idx = if let Some(&idx) = vertex_map.get(&key) {
				idx
			} else {
				let idx = vertices.len() as u8;
				vertices.push(TexturedVertexN {
					pos,
					normal: pos, // For a unit sphere, normal equals position
					uv,
				});
				vertex_map.insert(key, idx);
				idx
			};

			indices.push(vert_idx);
		}
	}

	(vertices, indices)
}

/// Generate flat-shaded icosahedron.
/// Each triangle gets its own vertices with face normals.
fn generate_flat() -> Vec<TexturedVertexN> {
	let (positions, triangles) = icosahedron_base();
	let mut vertices = Vec::with_capacity(60);

	for tri in triangles {
		let a = positions[tri[0] as usize];
		let b = positions[tri[1] as usize];
		let c = positions[tri[2] as usize];

		// Face normal (CCW winding)
		let normal = (b - a).cross(c - a).norm();
		let uvs = triangle_uvs(a, b, c);

		vertices.push(TexturedVertexN { pos: a, normal, uv: uvs[0] });
		vertices.push(TexturedVertexN { pos: b, normal, uv: uvs[1] });
		vertices.push(TexturedVertexN { pos: c, normal, uv: uvs[2] });
	}

	vertices
}

#[test]
#[ignore]
fn generate_icosahedron_bins() {
	let (verts, inds) = generate_smooth();
	assert_eq!(inds.len(), 60);
	assert!(inds.iter().all(|&i| (i as usize) < verts.len()));

	let flat_verts = generate_flat();
	assert_eq!(flat_verts.len(), 60);

	let out_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
		.join(std::path::Path::new(file!()).parent().unwrap());
	write_bin(&out_dir.join("vertices.bin"), &verts);
	write_bin(&out_dir.join("indices.bin"), &inds);
	write_bin(&out_dir.join("flat_vertices.bin"), &flat_verts);
}
