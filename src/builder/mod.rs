mod common;
mod ability;

use error::{Error, Result};
use std::collections::HashMap;
use sfml::graphics::{Color , Image, RenderTexture};
use unitypack::engine::mesh::Mesh;
use unitypack::engine::texture::IntoTexture2D;
use cards::{CardClass, CardType};
use assets::Assets;

lazy_static! {
    pub static ref TRANSPARENT_COLOR: Color = {
        Color::rgba(0, 0, 0, 0)
    };
}

pub fn build_card_frame(
    texture_map: &HashMap<String, String>,
    meshes_map: &HashMap<String, Mesh>,
    card_class: &CardClass,
    card_type: &CardType,
) -> Result<RenderTexture> {
    match *card_type {
        CardType::Spell | CardType::Enchantment => {
            ability::build_ability_frame_for_class(texture_map, meshes_map, card_class)
        }
        _ => Err(Error::NotImplementedError(
            format!("Card type {:?} is not implemented", card_type),
        )),
    }
}

pub fn build_ability_portrait(
    portrait_image: &Image,
    texture_map: &HashMap<String, String>,
    meshes_map: &HashMap<String, Mesh>,
) -> Result<RenderTexture> {
    let mesh = meshes_map
        .get(&"InHand_Ability_Portrait_mesh".to_string())
        .ok_or(Error::AssetNotFoundError(format!(
            "InHand_Ability_Portrait_mesh is not found in meshes"
        )))?;

    let shadow_texture =
        Assets::catalog_get(&texture_map, "Card_InHand_BannerAtlas")?.to_texture2d()?;
    let shadow_image = Image::create_from_pixels(
        shadow_texture.width,
        shadow_texture.height,
        &shadow_texture.to_image()?,
    ).ok_or(Error::SFMLError)?;
    common::build_portrait(portrait_image, &shadow_image, mesh)
}

// Returned texture needs to be flipped vertically
pub fn build_ability_portrait_frame(
    frame_image: &Image,
    meshes_map: &HashMap<String, Mesh>,
) -> Result<RenderTexture> {
    let mesh = meshes_map
        .get(&"InHand_Ability_Portrait_mesh".to_string())
        .ok_or(Error::AssetNotFoundError(format!(
            "InHand_Ability_Portrait_mesh is not found in meshes"
        )))?;

    common::build_portrait_frame(frame_image, mesh)
}

// Returned texture needs to be flipped vertically
pub fn build_ability_name_banner(
    texture_map: &HashMap<String, String>,
    meshes_map: &HashMap<String, Mesh>,
) -> Result<RenderTexture> {
    let banner_source =
        Assets::catalog_get(texture_map, "Card_InHand_BannerAtlas")?.to_texture2d()?;

    let mesh = meshes_map
        .get(&"InHand_Ability_NameBanner_mesh".to_string())
        .ok_or(Error::AssetNotFoundError(format!(
            "InHand_Ability_NameBanner_mesh is not found in meshes"
        )))?;

    let banner_image = Image::create_from_pixels(
        banner_source.width,
        banner_source.height,
        &banner_source.to_image()?,
    ).ok_or(Error::SFMLError)?;

    common::build_ability_name_banner(&banner_image, mesh)
}
