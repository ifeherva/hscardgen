use unitypack::engine::texture::{IntoTexture2D, Texture2D};
use std::collections::HashMap;
use sfml::graphics::{BlendMode, Color, Image, RenderStates, RenderTarget,
                     RenderTexture, Texture, Transform};
use cards::CardClass;
use error::{Error, Result};
use unitypack::engine::mesh::Mesh;
use assets::Assets;
use builder::common::create_vertex_array;

pub fn build_ability_frame_for_class(
    texture_map: &HashMap<String, String>,
    meshes_map: &HashMap<String, Mesh>,
    card_class: &CardClass,
) -> Result<RenderTexture> {
    let frame_texture = match *card_class {
        CardClass::Mage => {
            Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Mage")?.to_texture2d()?
        }
        CardClass::Priest => {
            Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Priest")?.to_texture2d()?
        }
        _ => {
            return Err(Error::NotImplementedError(
                format!("Card class {:?} is not implemented", card_class),
            ));
        }
    };
    let textbox_texture =
        Assets::catalog_get(&texture_map, "Card_InHand_BannerAtlas")?.to_texture2d()?;
    let gembox_texture =
        Assets::catalog_get(&texture_map, "Card_Inhand_Ability_Warlock")?.to_texture2d()?;
    build_card_ability_frame(frame_texture, textbox_texture, meshes_map)
}

fn build_card_ability_frame(
    frame_texture: Texture2D,
    textbox_texture: Texture2D,
    meshes_map: &HashMap<String, Mesh>,
) -> Result<RenderTexture> {
    // generate texture
    let source_width = frame_texture.width;
    let source_height = frame_texture.height;
    let source_image =
        Image::create_from_pixels(source_width, source_height, &frame_texture.to_image()?)
            .ok_or(Error::SFMLError)?;
    let texture = Texture::from_image(&source_image).ok_or(Error::SFMLError)?;

    // generate frame mesh
    let mesh = meshes_map
        .get("InHand_Ability_Base_mesh")
        .ok_or(Error::AssetNotFoundError(
            format!("Cannot find InHand_Ability_Base_mesh"),
        ))?;

    let vertex_array = create_vertex_array(mesh, 0, 0, source_width, source_height)?;

    // create canvas
    let bounds = vertex_array.bounds();
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
        Some(&texture),
        None,
    );
    canvas.draw_with_renderstates(&vertex_array, render_states);

    // create text background
    let source_width = textbox_texture.width;
    let source_height = textbox_texture.height;
    let source_image =
        Image::create_from_pixels(source_width, source_height, &textbox_texture.to_image()?)
            .ok_or(Error::SFMLError)?;
    let texture = Texture::from_image(&source_image).ok_or(Error::SFMLError)?;

    let mesh = meshes_map
        .get("InHand_Ability_Description_mesh")
        .ok_or(Error::AssetNotFoundError(
            format!("Cannot find InHand_Ability_Description_mesh"),
        ))?;

    let vertex_array = create_vertex_array(mesh, 0, 0, source_width, source_height)?;
    let bounds = vertex_array.bounds();
    let mut transform = Transform::default();
    transform.translate(41f32, 38f32);
    transform.scale(254f32 / bounds.width, 254f32 / bounds.width);

    let render_states = RenderStates::new(BlendMode::default(), transform, Some(&texture), None);
    canvas.draw_with_renderstates(&vertex_array, render_states);

    // draw gem socket
    /*let source_width = gembox_texture.width;
    let source_height = gembox_texture.height;
    let source_image =
        Image::create_from_pixels(source_width, source_height, &gembox_texture.to_image()?)
            .ok_or(Error::SFMLError)?;
    let texture = Texture::from_image(&source_image).ok_or(Error::SFMLError)?;

    let mesh = meshes_map
        .get("InHand_Ability_RarityFrame_mesh")
        .ok_or(Error::AssetNotFoundError(
            format!("Cannot find InHand_Ability_RarityFrame_mesh"),
        ))?;

    let vertex_array = create_vertex_array(mesh, 0, source_width, source_height, 249)?;
    let mut transform = Transform::default();
    transform.translate(131f32, 183f32);
    let render_states = RenderStates::new(BlendMode::default(), transform, Some(&texture), None);
    canvas.draw_with_renderstates(&vertex_array, render_states);
*/

    canvas.display();
    //let result = canvas.texture();

    // DEBUG DRAW
    //let img = result.copy_to_image().ok_or(Error::SFMLError)?;
    //img.save_to_file("/Users/istvanfe/Downloads/test2.png");
    // END DEBUG

    Ok(canvas)
}
