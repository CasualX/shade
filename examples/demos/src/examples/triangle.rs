use crate::*;

#[derive(Copy, Clone, Default, dataview::Pod)]
#[repr(C)]
struct TriangleVertex {
	position: Vec2f,
	color: [u8; 4],
}

unsafe impl shade::TVertex for TriangleVertex {
	const LAYOUT: &'static shade::VertexLayout = &shade::VertexLayout {
		size: mem::size_of::<TriangleVertex>() as u16,
		alignment: mem::align_of::<TriangleVertex>() as u16,
		attributes: &[
			shade::VertexAttribute {
				name: "aPos",
				format: shade::VertexAttributeFormat::F32v2,
				offset: dataview::offset_of!(TriangleVertex.position) as u16,
			},
			shade::VertexAttribute {
				name: "aColor",
				format: shade::VertexAttributeFormat::U8Normv4,
				offset: dataview::offset_of!(TriangleVertex.color) as u16,
			},
		],
	};
}

const PROGRAM: &str = r#"
#version unified 330 core, 300 es
precision highp float;

#ifdef VERTEX_SHADER
in vec2 aPos;
in vec4 aColor;
#endif

VARYING vec4 v_color;

#ifdef FRAGMENT_SHADER
out vec4 o_fragColor;
#endif

#ifdef VERTEX_SHADER
void main() {
	v_color = aColor;
	gl_Position = vec4(aPos, 0.0, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
	float levels = 10.0;
	vec3 qColor = floor(v_color.rgb * levels) / (levels - 1.0);
	o_fragColor = vec4(qColor, v_color.a);
}
#endif
"#;

pub fn create(g: &mut dyn shade::IGraphics, assets: &dyn AssetLoader) -> Box<dyn DemoInterface> {
	Box::new(Triangle::new(g, assets))
}

struct Triangle {
	vertices: Box<dyn shade::VertexBuffer>,
	shader: Box<dyn shade::ShaderProgram>,
}

impl Triangle {
	fn new(g: &mut dyn shade::IGraphics, _assets: &dyn AssetLoader) -> Triangle {
		let vertices = g.vertex_buffer(&[
				TriangleVertex { position: Vec2(0.0, 0.5), color: [255, 0, 0, 255] },
				TriangleVertex { position: Vec2(-0.5, -0.5), color: [0, 255, 0, 255] },
				TriangleVertex { position: Vec2(0.5, -0.5), color: [0, 0, 255, 255] },
			],
			shade::BufferUsage::Static,
		);
		let mut source = shade::shader_interface! {
			files {
				"main.glsl" => PROGRAM,
			}
		};
		let shader = g.shader_compile(&mut source, "main.glsl", &[]);
		Triangle { vertices, shader }
	}
}

impl DemoInterface for Triangle {
	fn input(&mut self, _input: crate::Input, _g: &mut dyn shade::IGraphics, _shell: &mut dyn ShellServices) {}

	fn draw(&mut self, frame: Frame, g: &mut dyn shade::IGraphics) {
		g.begin(&shade::BeginArgs::BackBuffer { viewport: frame.viewport });
		shade::clear!(g, color: Vec4(0.2, 0.5, 0.2, 1.0));
		g.draw(&shade::DrawArgs {
			scissor: None,
			blend_mode: shade::BlendMode::Solid,
			depth_test: None,
			cull_mode: None,
			mask: shade::DrawMask::COLOR,
			prim_type: shade::PrimType::Triangles,
			shader: &*self.shader,
			vertices: &[shade::DrawVertexBuffer {
				buffer: &*self.vertices,
				divisor: shade::VertexDivisor::PerVertex,
			}],
			uniforms: &[],
			vertex_start: 0,
			vertex_end: 3,
			instances: -1,
		});
		g.end();
	}
}
