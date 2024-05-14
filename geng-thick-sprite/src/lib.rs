use std::collections::BTreeMap;

use geng::prelude::{itertools::Itertools, *};

pub struct ThickSprite<V: ugli::Vertex> {
    pub texture: ugli::Texture,
    pub mesh: ugli::VertexBuffer<V>,
}

#[derive(ugli::Vertex)]
pub struct Vertex {
    pub a_pos: vec3<f32>,
    pub a_uv: vec2<f32>,
    pub a_normal: vec3<f32>,
}

#[derive(Copy, Clone)]
struct MarchVertex {
    pos: vec2<f32>,
    value: f32,
}

type MarchFace = [MarchVertex; 3];

/// mesh for value >= iso
fn marching_triangles(bb: Aabb2<i32>, f: impl Fn(vec2<i32>) -> f32, iso: f32) -> Vec<MarchFace> {
    let mut result = Vec::new();
    let mut march = |vs: &[vec2<i32>]| {
        let mut current = Vec::new();
        for (&a, &b) in vs.iter().circular_tuple_windows() {
            let va = f(a);
            let vb = f(b);
            let a = a.map(|x| x as f32);
            let b = b.map(|x| x as f32);
            if va >= iso {
                current.push(MarchVertex { pos: a, value: va });
            }
            let t = (iso - va) / (vb - va);
            if t > 0.0 && t < 1.0 {
                current.push(MarchVertex {
                    pos: a + (b - a) * t,
                    value: iso,
                });
            }
            if vb >= iso {
                current.push(MarchVertex { pos: b, value: vb });
            }
        }
        if current.len() >= 3 {
            let o = current[0];
            for (&a, &b) in current[1..].iter().tuple_windows() {
                result.push([o, a, b]);
            }
        }
    };
    for x in bb.min.x..bb.max.x {
        for y in bb.min.y..bb.max.y {
            // march([vec2(x, y), vec2(x + 1, y), vec2(x + 1, y + 1)]);
            // march([vec2(x, y), vec2(x + 1, y + 1), vec2(x, y + 1)]);
            march(&[
                vec2(x, y),
                vec2(x + 1, y),
                vec2(x + 1, y + 1),
                vec2(x, y + 1),
            ]);
        }
    }
    result
}

fn generate_mesh<V: From<Vertex>>(
    ugli: &Ugli,
    texture: &ugli::Texture,
    options: &Options,
) -> Vec<V> {
    let framebuffer =
        ugli::FramebufferRead::new_color(ugli, ugli::ColorAttachmentRead::Texture(texture));
    let data = framebuffer.read_color();

    let cell_size = options.cell_size;

    let mut cells = vec![
        vec![0.0; (texture.size().y + cell_size - 1) / cell_size];
        (texture.size().x + cell_size - 1) / cell_size
    ];
    for x in 0..texture.size().x {
        for y in 0..texture.size().y {
            let cell_x = x / cell_size;
            let cell_y = y / cell_size;
            cells[cell_x][cell_y] += data.get(x, y).a as f32 / u8::MAX as f32
        }
    }

    for (x, col) in cells.iter_mut().enumerate() {
        for (y, cell) in col.iter_mut().enumerate() {
            let pixels = min(cell_size, texture.size().x - x * cell_size)
                * min(cell_size, texture.size().y - y * cell_size);
            *cell /= pixels as f32;
        }
    }

    let iso = options.iso;

    let faces = marching_triangles(
        Aabb2::ZERO.extend_positive(vec2(cells.len(), cells[0].len()).map(|x| x as i32 - 1)),
        |vec2(x, y)| cells[x as usize][y as usize],
        iso,
    );

    let normals: BTreeMap<[R32; 2], vec2<f32>> = faces
        .iter()
        .flat_map(|face| {
            face.iter()
                .circular_tuple_windows()
                .filter_map(|(a, b)| {
                    let normal = (b.pos - a.pos).rotate_90().normalize_or_zero();
                    (a.value == iso && b.value == iso).then_some([(a.pos, normal), (b.pos, normal)])
                })
                .flatten()
        })
        .map(|(pos, normal)| (**pos.map(r32), normal))
        .collect();

    faces
        .iter()
        .flatten()
        .map(|v| (v, 1.0))
        .chain(faces.iter().flat_map(|face| {
            face.iter()
                .circular_tuple_windows()
                .filter_map(|(a, b)| {
                    (a.value == iso && b.value == iso).then_some([
                        (a, 1.0),
                        (b, -1.0),
                        (a, -1.0),
                        (a, 1.0),
                        (b, 1.0),
                        (b, -1.0),
                    ])
                })
                .flatten()
        }))
        .map(|(v, z)| {
            let normal = normals.get(&**v.pos.map(r32)).copied();
            let pixel_pos = v.pos.map(|x| (x + 0.5) * cell_size as f32);
            let uv = (pixel_pos + normal.unwrap_or(vec2::ZERO) * options.normal_uv_offset)
                / texture.size().map(|x| x as f32);
            Vertex {
                a_pos: uv.map(|x| x * 2.0 - 1.0).extend(z),
                a_uv: uv,
                a_normal: normal
                    .map(|normal| normal.extend(0.0))
                    .unwrap_or(vec3(0.0, 0.0, 1.0)),
            }
            .into()
        })
        .collect()
}

#[derive(Debug, Copy, Clone)]
pub struct Options {
    pub cell_size: usize,
    pub iso: f32,
    pub normal_uv_offset: f32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            cell_size: 10,
            iso: 0.5,
            normal_uv_offset: 2.0,
        }
    }
}

impl<V: ugli::Vertex + From<Vertex> + 'static> geng::asset::Load for ThickSprite<V> {
    type Options = Options;
    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        options: &Self::Options,
    ) -> geng::asset::Future<Self> {
        let texture = manager.load(path);
        let manager = manager.clone();
        let options = *options;
        async move {
            let texture = texture.await?;
            let vertices = generate_mesh(manager.ugli(), &texture, &options);
            let mesh = ugli::VertexBuffer::new_static(manager.ugli(), vertices);
            Ok(Self { texture, mesh })
        }
        .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = Some("png");
}
