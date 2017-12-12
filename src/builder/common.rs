use error::{Error, Result};
use sfml::system::Vector2f;
use byteorder::{LittleEndian, ReadBytesExt};
use sfml::graphics::{BlendMode, Color, Image, PrimitiveType, RenderStates, RenderTarget,
                     RenderTexture, Texture, Transform, Vertex, VertexArray};
use unitypack::engine::mesh::Mesh;
use std::{usize, f32};
use std::io::BufReader;

pub fn build_portrait(
    portrait_image: &Image,
    shadow_image: &Image,
    mesh: &Mesh,
) -> Result<RenderTexture> {
    let portrait_vertex_array =
        create_vertex_array(mesh, 1, 0, portrait_image.size().x, portrait_image.size().y)?;

    let white_color = Color::rgb(255, 255, 255);
    let portrait_background = Image::from_color(
        portrait_image.size().x,
        portrait_image.size().y,
        &white_color,
    ).ok_or(Error::SFMLError)?;

    let mut portrait_bg_texture =
        Texture::from_image(&portrait_background).ok_or(Error::SFMLError)?;
    portrait_bg_texture.set_smooth(true);
    let mut portrait_texture = Texture::from_image(&portrait_image).ok_or(Error::SFMLError)?;
    portrait_texture.set_smooth(true);
    let mut shadow_texture = Texture::from_image(&shadow_image).ok_or(Error::SFMLError)?;
    shadow_texture.set_smooth(true);

    // create canvas
    let portrait_bounds = portrait_vertex_array.bounds();
    let mut canvas = RenderTexture::new(
        (portrait_bounds.width + 1f32) as u32,
        (portrait_bounds.height + 1f32) as u32,
        false,
    ).ok_or(Error::SFMLError)?;
    canvas.set_smooth(true);
    let transparent_color = Color::rgba(0, 0, 0, 0);
    canvas.clear(&transparent_color);

    let render_states = RenderStates::new(
        BlendMode::default(),
        Transform::default(),
        Some(&portrait_bg_texture),
        None,
    );
    canvas.draw_with_renderstates(&portrait_vertex_array, render_states);
    let render_states = RenderStates::new(
        BlendMode::default(),
        Transform::default(),
        Some(&portrait_texture),
        None,
    );
    canvas.draw_with_renderstates(&portrait_vertex_array, render_states);

    // render shadow
    let shadow_vertex_array =
        create_vertex_array(mesh, 1, 1, shadow_image.size().x, shadow_image.size().y)?;
    let shadow_bounds = shadow_vertex_array.bounds();

    let mut shadow_transform = Transform::default();
    shadow_transform.scale(
        canvas.size().x as f32 / shadow_bounds.width as f32,
        canvas.size().y as f32 / shadow_bounds.height as f32,
    );

    let render_states = RenderStates::new(
        BlendMode::MULTIPLY,
        shadow_transform,
        Some(&shadow_texture),
        None,
    );

    canvas.draw_with_renderstates(&shadow_vertex_array, render_states);

    // do the rendering
    canvas.display();

    Ok(canvas)
}

pub fn build_portrait_frame(frame_image: &Image, mesh: &Mesh) -> Result<RenderTexture> {
    let frame_vertex_array =
        create_vertex_array(mesh, 0, 0, frame_image.size().x, frame_image.size().y)?;
    let mut frame_image_texture = Texture::from_image(&frame_image).ok_or(Error::SFMLError)?;
    frame_image_texture.set_smooth(true);

    let bounds = frame_vertex_array.bounds();
    let mut canvas = RenderTexture::new(
        (bounds.width + 1f32) as u32,
        (bounds.height + 1f32) as u32,
        false,
    ).ok_or(Error::SFMLError)?;
    canvas.set_smooth(true);
    let transparent_color = Color::rgba(0, 0, 0, 0);
    canvas.clear(&transparent_color);

    let render_states = RenderStates::new(
        BlendMode::default(),
        Transform::default(),
        Some(&frame_image_texture),
        None,
    );
    canvas.draw_with_renderstates(&frame_vertex_array, render_states);
    canvas.display();
    Ok(canvas)
}

pub fn build_ability_name_banner(banner_image: &Image, mesh: &Mesh) -> Result<RenderTexture> {
    let frame_vertex_array = create_vertex_array_(
        mesh,
        0,
        0,
        3,
        banner_image.size().x,
        banner_image.size().y,
        500,
        true,
    )?;
    let mut banner_image_texture = Texture::from_image(&banner_image).ok_or(Error::SFMLError)?;
    banner_image_texture.set_smooth(true);

    let bounds = frame_vertex_array.bounds();
    let mut canvas = RenderTexture::new(
        (bounds.width + 1f32) as u32,
        (bounds.height + 1f32) as u32,
        false,
    ).ok_or(Error::SFMLError)?;
    canvas.set_smooth(true);
    let transparent_color = Color::rgba(0, 0, 0, 0);
    canvas.clear(&transparent_color);

    let render_states = RenderStates::new(
        BlendMode::default(),
        Transform::default(),
        Some(&banner_image_texture),
        None,
    );
    canvas.draw_with_renderstates(&frame_vertex_array, render_states);
    canvas.display();

    // DEBUG DRAW
    {
        let result = canvas.texture();
        let img = result.copy_to_image().ok_or(Error::SFMLError)?;
        img.save_to_file("/Users/Haibane/Downloads/name_banner.png");
        img.save_to_file("/Users/istvanfe/Downloads/name_banner.png");
    }
    // END DEBUG


    Ok(canvas)
}

struct Vertex3D {
    coord_x: f32,
    coord_y: f32,
    coord_z: f32,

    texcoord_x: f32,
    texcoord_y: f32,
}

impl Vertex3D {
    fn read(
        vertex_idx: usize,
        vertex_data_size: usize,
        vertex_buffer: &[u8],
        coord_channel_offset: usize,
        coord_channel_size: usize,
        texcoord_channel_offset: usize,
        texcoord_channel_size: usize,
    ) -> Result<Vertex3D> {
        let data_idx = vertex_data_size * vertex_idx;
        let channel = &vertex_buffer
            [data_idx + coord_channel_offset..data_idx + coord_channel_offset + coord_channel_size];
        let mut reader = BufReader::new(channel);

        let c_x = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;
        let c_z = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;
        let c_y = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;

        let channel = &vertex_buffer[data_idx + texcoord_channel_offset
                                         ..data_idx + texcoord_channel_offset
                                             + texcoord_channel_size];
        let mut reader = BufReader::new(channel);
        let texcoord1_x: f32 = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;
        let texcoord1_y: f32 = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;

        Ok(Vertex3D {
            coord_x: c_x,
            coord_y: c_y,
            coord_z: c_z,

            texcoord_x: texcoord1_x,
            texcoord_y: texcoord1_y,
        })
    }
}

struct Triangle {
    vertices: [Vertex3D; 3],
    pub min_coord_x: f32,
    pub min_coord_y: f32,
    pub min_coord_z: f32,
    pub max_coord_x: f32,
}

impl Triangle {
    fn new(first: Vertex3D, second: Vertex3D, third: Vertex3D) -> Triangle {
        let min_coord_z = third.coord_z.min(first.coord_z.min(second.coord_z));
        let min_coord_x = third.coord_x.min(first.coord_x.min(second.coord_x));
        let min_coord_y = third.coord_y.min(first.coord_y.min(second.coord_y));
        let max_coord_x = third.coord_x.max(first.coord_x.max(second.coord_x));
        Triangle {
            vertices: [first, second, third],
            min_coord_x: min_coord_x,
            min_coord_y: min_coord_y,
            min_coord_z: min_coord_z,
            max_coord_x: max_coord_x,
        }
    }
}

fn read_triangle(
    index_buffer: &[u8],
    vertex_buffer: &[u8],
    vertex_data_size: usize,
    coord_channel_offset: usize,
    coord_channel_size: usize,
    texcoord_channel_offset: usize,
    texcoord_channel_size: usize,
) -> Result<Triangle> {
    let mut reader = BufReader::new(index_buffer);

    let vertex_idx_1: u16 = ReadBytesExt::read_u16::<LittleEndian>(&mut reader)?;
    let vertex_idx_2: u16 = ReadBytesExt::read_u16::<LittleEndian>(&mut reader)?;
    let vertex_idx_3: u16 = ReadBytesExt::read_u16::<LittleEndian>(&mut reader)?;

    // read all channels here
    let vertex_1 = Vertex3D::read(
        vertex_idx_1 as usize,
        vertex_data_size,
        vertex_buffer,
        coord_channel_offset,
        coord_channel_size,
        texcoord_channel_offset,
        texcoord_channel_size,
    )?;
    let vertex_2 = Vertex3D::read(
        vertex_idx_2 as usize,
        vertex_data_size,
        vertex_buffer,
        coord_channel_offset,
        coord_channel_size,
        texcoord_channel_offset,
        texcoord_channel_size,
    )?;
    let vertex_3 = Vertex3D::read(
        vertex_idx_3 as usize,
        vertex_data_size,
        vertex_buffer,
        coord_channel_offset,
        coord_channel_size,
        texcoord_channel_offset,
        texcoord_channel_size,
    )?;

    Ok(Triangle::new(vertex_1, vertex_2, vertex_3))
}

pub fn create_vertex_array_(
    mesh: &Mesh,
    submesh_idx: usize,
    coord_channel_idx: usize,
    texcoord_channel_idx: usize,
    source_width: u32,
    source_height: u32,
    output_width: usize,
    sort_by_z: bool,
) -> Result<VertexArray> {
    let submesh = mesh.submeshes
        .get(submesh_idx)
        .ok_or(Error::AssetNotFoundError(
            format!("Submesh {} not found", submesh_idx),
        ))?;

    // size of data per vertex
    let vertex_data_size = mesh.vertex_data.data.len() / mesh.vertex_data.vertex_count as usize;
    // vertex data offset of the current submesh
    let data_offset = submesh.first_byte as usize;

    let coord_channel_offset = mesh.vertex_data.channels[coord_channel_idx]
        .get(&"offset".to_string())
        .ok_or(Error::ObjectTypeError)?
        .to_u8()? as usize;
    let coord_channel_size = mesh.vertex_data.channels[coord_channel_idx]
        .get(&"dimension".to_string())
        .ok_or(Error::ObjectTypeError)?
        .to_u8()? as usize * 4;
    let texcoord_channel_offset = mesh.vertex_data.channels[texcoord_channel_idx]
        .get(&"offset".to_string())
        .ok_or(Error::ObjectTypeError)?
        .to_u8()? as usize;
    let texcoord_channel_size = mesh.vertex_data.channels[texcoord_channel_idx]
        .get(&"dimension".to_string())
        .ok_or(Error::ObjectTypeError)?
        .to_u8()? as usize * 4;

    if submesh.index_count % 3 != 0 {
        return Err(Error::InvalidAssetError(
            format!("Invalid vertex count for mesh"),
        ));
    }
    let triangle_count = (submesh.index_count / 3) as usize;

    let mut triangles: Vec<Triangle> = Vec::with_capacity(triangle_count);

    for i in 0..triangle_count as usize {
        let index_buffer = &mesh.index_buffer[(i * 6) + data_offset..];
        // get current vertex index
        let triangle = read_triangle(
            index_buffer,
            &mesh.vertex_data.data,
            vertex_data_size,
            coord_channel_offset,
            coord_channel_size,
            texcoord_channel_offset,
            texcoord_channel_size,
        )?;
        triangles.push(triangle);
    }

    if sort_by_z {
        triangles.sort_by(|a, b| a.min_coord_z.partial_cmp(&b.min_coord_z).unwrap());
    }

    // compute texcoord offsets
    let (min_x, min_y, max_x): (f32, f32, f32) = triangles.iter().fold(
        (f32::MAX, f32::MAX, f32::MIN),
        |min: (f32, f32, f32), val| {
            (
                min.0.min(val.min_coord_x),
                min.1.min(val.min_coord_y),
                min.2.max(val.max_coord_x),
            )
        },
    );

    let scaling_factor: f32 = output_width as f32 / (max_x - min_x);

    let mut vertex_array = VertexArray::new(PrimitiveType::Triangles, triangles.len() * 3);

    for triangle in triangles {
        for vertex in triangle.vertices.iter() {
            let vertex = Vertex::new(
                Vector2f {
                    x: (vertex.coord_x - min_x) * scaling_factor,
                    y: (vertex.coord_y - min_y) * scaling_factor,
                },
                Color::rgba(255, 255, 255, 255),
                Vector2f {
                    x: vertex.texcoord_x * source_width as f32,
                    y: vertex.texcoord_y * source_height as f32,
                },
            );
            vertex_array.append(&vertex);
        }
    }

    Ok(vertex_array)
}

pub fn create_vertex_array(
    mesh: &Mesh,
    submesh_idx: usize,
    texcoord_idx: usize,
    source_width: u32,
    source_height: u32,
) -> Result<VertexArray> {
    // process vertices
    let mut raw_vertices = Vec::new();

    let submesh = mesh.submeshes
        .get(submesh_idx)
        .ok_or(Error::AssetNotFoundError(
            format!("Submesh {} not found", submesh_idx),
        ))?;

    let vertex_data_size = mesh.vertex_data.data.len() / mesh.vertex_data.vertex_count as usize;
    let mut texcoord_offset = 24;
    if texcoord_idx == 1 {
        texcoord_offset = 32;
    }
    let data_offset = submesh.first_byte as usize;
    for i in 0..submesh.index_count as usize {
        // get current vertex index
        let mut reader =
            BufReader::new(&mesh.index_buffer[(i * 2) + data_offset..((i * 2) + 2) + data_offset]);
        let vertex_idx: u16 = ReadBytesExt::read_u16::<LittleEndian>(&mut reader)?;

        // only read the last two 4-byte fields
        let data_idx = (vertex_data_size * vertex_idx as usize) + texcoord_offset;
        let fields = &mesh.vertex_data.data[data_idx..data_idx + 8];
        reader = BufReader::new(fields);
        let texcoord1_x: f32 = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;
        let texcoord1_y: f32 = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;
        raw_vertices.push((texcoord1_x, texcoord1_y));
    }

    let (min_x, min_y): (f32, f32) = raw_vertices
        .iter()
        .fold((1f32, 1f32), |min: (f32, f32), val| {
            (min.0.min(val.0), min.1.min(val.1))
        });

    let mut vertex_array = VertexArray::new(PrimitiveType::Triangles, raw_vertices.len());

    for raw_vertex in raw_vertices {
        let x = (raw_vertex.0 - min_x) * source_width as f32;
        let y = (raw_vertex.1 - min_y) * source_height as f32;
        let vertex = Vertex::new(
            Vector2f { x: x, y: y },
            Color::rgba(255, 255, 255, 255),
            Vector2f {
                x: raw_vertex.0 * source_width as f32,
                y: raw_vertex.1 * source_height as f32,
            },
        );
        vertex_array.append(&vertex);
    }

    Ok(vertex_array)
}
