use sfml::system::Vector2f;
use unitypack::engine::texture::{IntoTexture2D, Texture2D};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::BufReader;
use std::collections::HashMap;
use sfml::graphics::{BlendMode, Color, Image, PrimitiveType, RenderStates, RenderTarget,
                     RenderTexture, Texture, Transform, Vertex, VertexArray};
use cards::CardClass;
use error::{Error, Result};
use unitypack::engine::mesh::Mesh;
use assets::Assets;


pub fn build_ability_frame_for_class(
    texture_map: &HashMap<String, String>,
    meshes_map: &HashMap<String, Mesh>,
    card_class: &CardClass,
) -> Result<Texture> {
    let frame_texture = match *card_class {
        CardClass::Mage => {
            Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Mage")?.to_texture2d()?
        }
        _ => {
            return Err(Error::NotImplementedError(
                format!("Card class {:?} is not implemented", card_class),
            ));
        }
    };
    build_ability_frame(frame_texture, meshes_map)
}

fn build_ability_frame(
    source_texture: Texture2D,
    meshes_map: &HashMap<String, Mesh>,
) -> Result<Texture> {
    // generate texture
    let source_width = source_texture.width;
    let source_height = source_texture.height;
    let source_image = source_texture.to_image()?;
    let source_image = Image::create_from_pixels(source_width, source_height, &source_image)
        .ok_or(Error::SFMLError)?;
    let texture = Texture::from_image(&source_image).ok_or(Error::SFMLError)?;

    // generate main mesh
    let mesh = meshes_map
        .get("InHand_Ability_Base_mesh")
        .ok_or(Error::AssetNotFoundError(
            format!("Cannot find InHand_Ability_Base_mesh"),
        ))?;

    // process vertices
    let mut raw_vertices = Vec::new();

    let submesh = mesh.submeshes
        .get(0)
        .ok_or(Error::AssetNotFoundError(format!("Submesh 0 not found")))?;

    let vertex_data_size = mesh.vertex_data.data.len() / mesh.vertex_data.vertex_count as usize;

    for i in 0..submesh.index_count as usize {
        // get current vertex index
        let mut reader = BufReader::new(&mesh.index_buffer[(i * 2)..((i * 2) + 2)]);
        let vertex_idx: u16 = ReadBytesExt::read_u16::<LittleEndian>(&mut reader)?;

        // only read the last two 4-byte fields
        let data_idx = (vertex_data_size * vertex_idx as usize) + 24;
        let fields = &mesh.vertex_data.data[data_idx..data_idx + 8];
        reader = BufReader::new(fields);
        let texcoord_x: f32 = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;
        let texcoord_y: f32 = ReadBytesExt::read_f32::<LittleEndian>(&mut reader)?;
        raw_vertices.push((texcoord_x, texcoord_y));
    }

    let (min_x, min_y, max_x, max_y): (f32, f32, f32, f32) = raw_vertices.iter().fold(
        (1f32, 1f32, 0f32, 0f32),
        |minmax: (f32, f32, f32, f32), val| {
            (
                minmax.0.min(val.0),
                minmax.1.min(val.1),
                minmax.2.max(val.0),
                minmax.3.max(val.1),
            )
        },
    );

    let mut vertex_array = VertexArray::new_init(PrimitiveType::Triangles, raw_vertices.len());

    for raw_vertex in raw_vertices {
        let x = (raw_vertex.0 - min_x) * source_width as f32;
        let y = (raw_vertex.1 - min_y) * source_width as f32;
        let vertex = Vertex::new(
            Vector2f { x: x, y: y },
            Color::rgba(255, 255, 255, 255),
            Vector2f {
                x: raw_vertex.0 * source_width as f32,
                y: raw_vertex.1 * source_width as f32,
            },
        );
        vertex_array.append(&vertex);
    }

    let bounds = vertex_array.bounds();
    let mut canvas = RenderTexture::new(
        ((source_width as f32 * (max_x - min_x)) + 1f32) as u32,
        (bounds.height + 1f32) as u32,
        false,
    ).ok_or(Error::SFMLError)?;
    let transparent_color = Color::rgba(0, 0, 0, 0);
    canvas.clear(&transparent_color);

    let render_states = RenderStates::new(
        BlendMode::default(),
        Transform::default(),
        Some(&texture),
        None,
    );
    canvas.draw_with_renderstates(&vertex_array, render_states);
    canvas.set_smooth(true);
    canvas.display();

    let img = canvas.texture().copy_to_image().ok_or(Error::SFMLError)?;
    img.save_to_file("/Users/istvanfe/Downloads/test2.png");

    // this will fail below
    Ok(Texture::new(0, 0).unwrap())
}
