use unitypack::engine::texture::IntoTexture2D;
use std::collections::HashMap;
use sfml::graphics::{BlendMode, Image, RenderStates, RenderTarget, RenderTexture, Shader, Texture,
                     TextureRef, Transform};
use sfml::system::Vector2u;
use cards::CardClass;
use error::{Error, Result};
use unitypack::engine::mesh::Mesh;
use assets::Assets;
use builder::common::create_vertex_array;
use builder::TRANSPARENT_COLOR;
use utils::IntoImage;

pub fn build_ability_frame_for_class(
    texture_map: &HashMap<String, String>,
    meshes_map: &HashMap<String, Mesh>,
    shader: Option<&Shader>,
    card_class: &CardClass,
) -> Result<RenderTexture> {
    let textbox_image = Assets::catalog_get(&texture_map, "Card_InHand_BannerAtlas")?
        .to_texture2d()?
        .to_sfml_image()?;

    let frame_image = match *card_class {
        CardClass::Mage => Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Mage")?
            .to_texture2d()?
            .to_sfml_image()?,
        CardClass::Priest => Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Priest")?
            .to_texture2d()?
            .to_sfml_image()?,
        CardClass::Warrior => Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Warrior")?
            .to_texture2d()?
            .to_sfml_image()?,
        CardClass::Hunter => Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Hunter")?
            .to_texture2d()?
            .to_sfml_image()?,
        CardClass::Warlock => Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Warlock")?
            .to_texture2d()?
            .to_sfml_image()?,
        CardClass::Paladin => Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Paladin")?
            .to_texture2d()?
            .to_sfml_image()?,
        CardClass::Shaman => Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Shaman")?
            .to_texture2d()?
            .to_sfml_image()?,
        CardClass::Rogue => Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Rogue")?
            .to_texture2d()?
            .to_sfml_image()?,
        CardClass::Druid => Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Rogue")?
            .to_texture2d()?
            .to_sfml_image()?,
        _ => {
            return Err(Error::NotImplementedError(format!(
                "Card frame generation for class {:?} is not implemented",
                card_class
            )));
        }
    };
    match *card_class {
        CardClass::Warlock => build_card_ability_frame(
            &frame_image,
            &frame_image,
            &textbox_image,
            meshes_map,
            shader,
        ),
        _ => {
            let helper_image = Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Warlock")?
                .to_texture2d()?
                .to_sfml_image()?;
            build_card_ability_frame(
                &frame_image,
                &helper_image,
                &textbox_image,
                meshes_map,
                shader,
            )
        }
    }
}

fn build_card_ability_frame(
    frame_image: &Image,
    helper_image: &Image,
    textbox_image: &Image,
    meshes_map: &HashMap<String, Mesh>,
    shader: Option<&Shader>,
) -> Result<RenderTexture> {
    let mut frame_texture = Texture::from_image(&frame_image).ok_or(Error::SFMLError)?;
    frame_texture.set_smooth(true);

    let mut description_frame_texture = Texture::from_image(&helper_image).ok_or(Error::SFMLError)?;
    description_frame_texture.set_smooth(true);

    let mut textbox_texture = Texture::from_image(&textbox_image).ok_or(Error::SFMLError)?;
    textbox_texture.set_smooth(true);

    let frame_mesh = meshes_map
        .get("InHand_Ability_Base_mesh")
        .ok_or(Error::AssetNotFoundError(format!(
            "Cannot find InHand_Ability_Base_mesh"
        )))?;

    let textbox_mesh = meshes_map.get("InHand_Ability_Description_mesh").ok_or(
        Error::AssetNotFoundError(format!("Cannot find InHand_Ability_Description_mesh")),
    )?;

    let frame_vertex_array = create_vertex_array(
        frame_mesh,
        0,
        0,
        3,
        frame_texture.size().x,
        frame_texture.size().y,
        360,
        false,
        &Vector2u { x: 0, y: 0 },
    )?;

    let description_frame_vertex_array = create_vertex_array(
        textbox_mesh,
        1,
        0,
        3,
        description_frame_texture.size().x,
        description_frame_texture.size().y,
        313,
        true,
        &Vector2u { x: 0, y: 0 },
    )?;

    let textbox_vertex_array = create_vertex_array(
        textbox_mesh,
        0,
        0,
        3,
        textbox_texture.size().x,
        textbox_texture.size().y,
        275,
        false,
        &Vector2u { x: 0, y: 0 },
    )?;

    let frame_bounds = frame_vertex_array.bounds();
    let textbox_bounds = textbox_vertex_array.bounds();
    let description_frame_bounds = description_frame_vertex_array.bounds();

    let mut frame_transform = Transform::default();
    frame_transform.scale_with_center(
        -1f32,
        1f32,
        frame_bounds.width / 2f32,
        frame_bounds.height / 2f32,
    );

    let mut textbox_transform = Transform::default();
    textbox_transform.scale_with_center(
        -1f32,
        1f32,
        textbox_bounds.width / 2f32,
        textbox_bounds.height / 2f32,
    );
    textbox_transform.translate(-41f32, 308f32);

    let mut description_frame_transform = Transform::default();
    description_frame_transform.scale_with_center(
        -1f32,
        1f32,
        description_frame_bounds.width / 2f32,
        description_frame_bounds.height / 2f32,
    );
    description_frame_transform.translate(-22f32, 290f32);

    // create canvas
    let mut canvas = RenderTexture::new(
        (frame_bounds.width.ceil()) as u32,
        (frame_bounds.height.ceil()) as u32,
        false,
    ).ok_or(Error::SFMLError)?;
    canvas.set_smooth(true);
    canvas.clear(&TRANSPARENT_COLOR);

    let frame_render_states = RenderStates::new(
        BlendMode::default(),
        frame_transform,
        Some(&frame_texture),
        shader,
    );

    let description_frame_render_states = RenderStates::new(
        BlendMode::default(),
        description_frame_transform,
        Some(&description_frame_texture),
        shader,
    );

    let textbox_render_states = RenderStates::new(
        BlendMode::default(),
        textbox_transform,
        Some(&textbox_texture),
        shader,
    );
    canvas.draw_with_renderstates(&frame_vertex_array, frame_render_states);
    canvas.draw_with_renderstates(&textbox_vertex_array, textbox_render_states);
    canvas.draw_with_renderstates(
        &description_frame_vertex_array,
        description_frame_render_states,
    );

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

    let vertex_array = create_vertex_array(
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
