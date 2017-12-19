use unitypack::engine::texture::{IntoTexture2D, Texture2D};
use std::collections::HashMap;
use sfml::graphics::{BlendMode, Image, RenderStates, RenderTarget, RenderTexture, Shader, Texture,
                     TextureRef, Transform};
use sfml::system::Vector2u;
use cards::CardClass;
use error::{Error, Result};
use unitypack::engine::mesh::Mesh;
use assets::Assets;
use builder::common::{create_vertex_array, create_vertex_array_};
use builder::TRANSPARENT_COLOR;

pub fn build_ability_frame_for_class(
    texture_map: &HashMap<String, String>,
    meshes_map: &HashMap<String, Mesh>,
    shader: Option<&Shader>,
    card_class: &CardClass,
) -> Result<RenderTexture> {
    let frame_texture = match *card_class {
        CardClass::Mage => {
            Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Mage")?.to_texture2d()?
        }
        CardClass::Priest => {
            Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Priest")?.to_texture2d()?
        }
        CardClass::Warrior => {
            Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Warrior")?.to_texture2d()?
        }
        CardClass::Hunter => {
            Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Hunter")?.to_texture2d()?
        }
        CardClass::Warlock => {
            Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Warlock")?.to_texture2d()?
        }
        CardClass::Paladin => {
            Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Paladin")?.to_texture2d()?
        }
        CardClass::Shaman => {
            Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Shaman")?.to_texture2d()?
        }
        CardClass::Rogue => {
            Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Rogue")?.to_texture2d()?
        }
        CardClass::Druid => {
            Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Rogue")?.to_texture2d()?
        }
        _ => {
            return Err(Error::NotImplementedError(format!(
                "Card frame generation for class {:?} is not implemented",
                card_class
            )));
        }
    };
    let textbox_texture =
        Assets::catalog_get(&texture_map, "Card_InHand_BannerAtlas")?.to_texture2d()?;
    build_card_ability_frame(frame_texture, textbox_texture, meshes_map, shader)
}

fn build_card_ability_frame(
    frame_texture: Texture2D,
    textbox_texture: Texture2D,
    meshes_map: &HashMap<String, Mesh>,
    shader: Option<&Shader>,
) -> Result<RenderTexture> {
    // generate texture
    let source_width = frame_texture.width;
    let source_height = frame_texture.height;
    let source_image =
        Image::create_from_pixels(source_width, source_height, &frame_texture.to_image()?)
            .ok_or(Error::SFMLError)?;
    let mut texture = Texture::from_image(&source_image).ok_or(Error::SFMLError)?;
    texture.set_smooth(true);

    // generate frame mesh
    let mesh = meshes_map
        .get("InHand_Ability_Base_mesh")
        .ok_or(Error::AssetNotFoundError(format!(
            "Cannot find InHand_Ability_Base_mesh"
        )))?;

    let vertex_array = create_vertex_array(mesh, 0, 0, source_width, source_height)?;

    // create canvas
    let bounds = vertex_array.bounds();

    let mut canvas = RenderTexture::new(
        (bounds.width.ceil()) as u32,
        (bounds.height.ceil()) as u32,
        false,
    ).ok_or(Error::SFMLError)?;
    canvas.set_smooth(true);
    canvas.clear(&TRANSPARENT_COLOR);

    let render_states = RenderStates::new(
        BlendMode::default(),
        Transform::default(),
        Some(&texture),
        shader,
    );
    canvas.draw_with_renderstates(&vertex_array, render_states);

    // create text background
    let source_width = textbox_texture.width;
    let source_height = textbox_texture.height;
    let source_image =
        Image::create_from_pixels(source_width, source_height, &textbox_texture.to_image()?)
            .ok_or(Error::SFMLError)?;
    let mut texture = Texture::from_image(&source_image).ok_or(Error::SFMLError)?;
    texture.set_smooth(true);

    let mesh = meshes_map.get("InHand_Ability_Description_mesh").ok_or(
        Error::AssetNotFoundError(format!("Cannot find InHand_Ability_Description_mesh")),
    )?;

    let vertex_array = create_vertex_array(mesh, 0, 0, source_width, source_height)?;
    let bounds = vertex_array.bounds();
    let mut transform = Transform::default();

    let translate_aspect_factor = source_width as f32 / 1024f32;
    transform.translate(
        82f32 * translate_aspect_factor,
        76f32 * translate_aspect_factor,
    ); // 41 , 38
    transform.scale(
        508f32 * translate_aspect_factor / bounds.width,
        508f32 * translate_aspect_factor / bounds.width,
    ); // 254

    let render_states = RenderStates::new(BlendMode::default(), transform, Some(&texture), shader);
    canvas.draw_with_renderstates(&vertex_array, render_states);

    canvas.display();
    Ok(canvas)
}

pub fn build_card_name(
    name_texture: &TextureRef,
    mesh: &Mesh,
    width: usize,
) -> Result<RenderTexture> {
    let source_width = name_texture.size().x;
    let source_height = name_texture.size().y;

    let vertex_array = create_vertex_array_(
        mesh,
        0,
        0,
        3,
        source_width,
        source_height,
        width,
        false,
        &Vector2u { x: 0, y: 0 },
    )?;

    // create canvas
    let bounds = vertex_array.bounds();

    let mut canvas = RenderTexture::new(
        (bounds.width.ceil()) as u32,
        (bounds.height.ceil()) as u32,
        false,
    ).ok_or(Error::SFMLError)?;
    canvas.set_smooth(true);
    canvas.clear(&TRANSPARENT_COLOR);

    let render_states = RenderStates::new(
        BlendMode::default(),
        Transform::default(),
        Some(name_texture),
        None,
    );
    canvas.draw_with_renderstates(&vertex_array, render_states);

    canvas.display();

    Ok(canvas)
}
