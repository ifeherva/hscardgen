mod common;
mod ability;

use error::{Error, Result};
use std::collections::HashMap;
use sfml::graphics::{Color, Image, RenderTexture, Shader, Text, TextStyle, TextureRef};
use sfml::system::Vector2u;
use utils::ImageUtils;
use unitypack::engine::mesh::Mesh;
use unitypack::engine::texture::IntoTexture2D;
use cards::{CardClass, CardRarity, CardType};
use assets::Assets;

//const VERTEX_SHADER_SOURCE: &'static str = include_str!("../../res/vertex_shader.glsl");
//const FRAGMENT_SHADER_SOURCE: &'static str = include_str!("../../res/fragment_shader.glsl");

lazy_static! {
    pub static ref TRANSPARENT_COLOR: Color = {
        Color::rgba(0, 0, 0, 0)
    };
}

pub struct Builder<'a> {
    shader: Option<Shader<'a>>,
}

impl<'a> Builder<'a> {
    pub fn new() -> Result<Builder<'a>> {
        Ok(Builder {
            shader: None, /*Shader::from_memory(
                // do not use the shader as it is currently broken
                Some(VERTEX_SHADER_SOURCE),
                None,
                Some(FRAGMENT_SHADER_SOURCE),
            )*/
        })
    }

    pub fn build_card_frame(
        &self,
        texture_map: &HashMap<String, String>,
        meshes_map: &HashMap<String, Mesh>,
        card_class: &CardClass,
        card_type: &CardType,
    ) -> Result<RenderTexture> {
        match *card_type {
            CardType::Spell | CardType::Enchantment => ability::build_ability_frame_for_class(
                texture_map,
                meshes_map,
                self.shader.as_ref(),
                card_class,
            ),
            _ => Err(Error::NotImplementedError(format!(
                "Card type {:?} is not implemented",
                card_type
            ))),
        }
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
    width: usize,
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

    common::build_ability_name_banner(&banner_image, mesh, width)
}

pub fn build_mana_gem(
    texture_map: &HashMap<String, String>,
    meshes_map: &HashMap<String, Mesh>,
    width: usize,
) -> Result<RenderTexture> {
    let mana_gem_texture = Assets::catalog_get(texture_map, "Gem_Mana_D")?.to_texture2d()?;

    let mana_gem_mesh = meshes_map
        .get("ManaGem")
        .ok_or(Error::AssetNotFoundError(format!("Cannot find ManaGem")))?;

    let mut mana_gem_image = Image::create_from_pixels(
        mana_gem_texture.width,
        mana_gem_texture.height,
        &mana_gem_texture.to_image()?,
    ).ok_or(Error::SFMLError)?;
    mana_gem_image.remove_transparency();

    common::build_mana_gem(&mana_gem_image, mana_gem_mesh, width)
}

pub fn build_rarity_gem_socket(
    texture_map: &HashMap<String, String>,
    meshes_map: &HashMap<String, Mesh>,
    width: usize,
) -> Result<RenderTexture> {
    let texture = Assets::catalog_get(texture_map, "Card_Inhand_Ability_Warlock")?.to_texture2d()?;
    let mesh = meshes_map.get("InHand_Ability_RarityFrame_mesh").ok_or(
        Error::AssetNotFoundError(format!("Cannot find InHand_Ability_RarityFrame_mesh")),
    )?;

    let gem_socket_image =
        Image::create_from_pixels(texture.width, texture.height, &texture.to_image()?)
            .ok_or(Error::SFMLError)?;

    common::build_rarity_socket(&gem_socket_image, mesh, width)
}

pub fn build_rarity_gem(
    texture_map: &HashMap<String, String>,
    meshes_map: &HashMap<String, Mesh>,
    rarity: &CardRarity,
    width: usize,
) -> Result<RenderTexture> {
    let gem_texture = Assets::catalog_get(texture_map, "RarityGems")?.to_texture2d()?;
    let shader_texture = Assets::catalog_get(texture_map, "clouds3")?.to_texture2d()?;

    let mesh = meshes_map
        .get("RarityGem_mesh")
        .ok_or(Error::AssetNotFoundError(format!(
            "Cannot find RarityGem_mesh"
        )))?;

    let mut gem_image = Image::create_from_pixels(
        gem_texture.width,
        gem_texture.height,
        &gem_texture.to_image()?,
    ).ok_or(Error::SFMLError)?;

    let mut shader_image = Image::create_from_pixels(
        shader_texture.width,
        shader_texture.height,
        &shader_texture.to_image()?,
    ).ok_or(Error::SFMLError)?;

    shader_image = shader_image.resize(gem_image.size().x, gem_image.size().y)?;

    // remove and transfer transparency
    for x in 0..gem_image.size().x {
        for y in 0..gem_image.size().y {
            let mut gem_pixel = gem_image.pixel_at(x, y);
            let mut shader_pixel = shader_image.pixel_at(x, y);
            shader_pixel.a = gem_pixel.a;
            gem_pixel.a = 255;
            gem_image.set_pixel(x, y, &gem_pixel);
            shader_image.set_pixel(x, y, &shader_pixel);
        }
    }

    let offset = match rarity {
        &CardRarity::COMMON => Vector2u { x: 0, y: 0 },
        &CardRarity::RARE => Vector2u {
            x: gem_image.size().x / 2,
            y: 0,
        },
        &CardRarity::EPIC => Vector2u {
            x: 0,
            y: gem_image.size().y / 2,
        },
        &CardRarity::LEGENDARY => Vector2u {
            x: gem_image.size().x / 2,
            y: gem_image.size().y / 2,
        },
        _ => {
            return Err(Error::InvalidCardError);
        }
    };

    common::build_rarity_gem(&gem_image, &shader_image, mesh, &offset, width)
}

pub fn build_card_name(
    name_texture: &TextureRef,
    meshes_map: &HashMap<String, Mesh>,
    width: usize,
) -> Result<RenderTexture> {
    let mesh = meshes_map
        .get("AbilityCardCurvedText")
        .ok_or(Error::AssetNotFoundError(format!(
            "Cannot find AbilityCardCurvedText"
        )))?;

    ability::build_card_name(name_texture, mesh, width)
}

pub fn build_name_texture(card_name: &str, text: &mut Text) -> Result<RenderTexture> {
    text.set_string(card_name);
    text.set_style(TextStyle::REGULAR);
    text.set_character_size(30);
    text.set_outline_thickness(2f32);
    common::build_name_texture(text)
}
