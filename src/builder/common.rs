use error::{Error, Result};
use sfml::system::Vector2f;
use byteorder::{LittleEndian, ReadBytesExt};
use sfml::graphics::{BlendMode, Color, Image, PrimitiveType, RenderStates, RenderTarget,
                     RenderTexture, Texture, Transform, Vertex, VertexArray};
use unitypack::engine::mesh::Mesh;
use std::io::BufReader;
use std::usize;

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
    let frame_vertex_array =
        create_vertex_array2(mesh, 0, 0, 3, banner_image.size().x, banner_image.size().y)?;
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
    }
    // END DEBUG


    Ok(canvas)
}

pub fn create_vertex_array2(
    mesh: &Mesh,
    submesh_idx: usize,
    coord_channel_idx: usize,
    texcoord_channel_idx: usize,
    source_width: u32,
    source_height: u32,
) -> Result<VertexArray> {
    // process vertices
    let mut raw_vertices = Vec::new();

    let submesh = mesh.submeshes
        .get(submesh_idx)
        .ok_or(Error::AssetNotFoundError(format!(
            "Submesh {} not found",
            submesh_idx
        )))?;

    let vertex_data_size = mesh.vertex_data.data.len() / mesh.vertex_data.vertex_count as usize;
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

    for i in 0..submesh.index_count as usize {
        // get current vertex index
        let mut reader =
            BufReader::new(&mesh.index_buffer[(i * 2) + data_offset..((i * 2) + 2) + data_offset]);
        let vertex_idx: u16 = ReadBytesExt::read_u16::<LittleEndian>(&mut reader)?;

        // read all channels here
        let data_idx = vertex_data_size * vertex_idx as usize;
        let channel = &mesh.vertex_data.data
            [data_idx + coord_channel_offset..data_idx + coord_channel_offset + coord_channel_size];
        reader = BufReader::new(channel);

        let c_x = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;
        let c_z = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;
        let c_y = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;

        let channel = &mesh.vertex_data.data[data_idx + texcoord_channel_offset
                                                 ..data_idx + texcoord_channel_offset
                                                     + texcoord_channel_size];
        reader = BufReader::new(channel);
        let texcoord1_x: f32 = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;
        let texcoord1_y: f32 = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;

        raw_vertices.push(((c_x, c_y, c_z), (texcoord1_x, texcoord1_y)));
    }

    // compute texcoord offsets
    let (min_x, min_y): (f32, f32) = raw_vertices
        .iter()
        .fold((0f32, 0f32), |min: (f32, f32), val| {
            (min.0.min((val.0).0), min.1.min((val.0).1))
        });


    let mut vertex_array = VertexArray::new(PrimitiveType::Triangles, raw_vertices.len());

    for raw_vertex in raw_vertices {
        let x = ((raw_vertex.0).0 - min_x) * source_width as f32;
        let y = ((raw_vertex.0).1 - min_y) * source_width as f32;
        let vertex = Vertex::new(
            Vector2f { x: x, y: y },
            Color::rgba(255, 255, 255, 255),
            Vector2f {
                x: (raw_vertex.1).0 * source_width as f32,
                y: (raw_vertex.1).1 * source_height as f32,
            },
        );
        vertex_array.append(&vertex);
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
        .ok_or(Error::AssetNotFoundError(format!(
            "Submesh {} not found",
            submesh_idx
        )))?;

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