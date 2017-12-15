mod common;
mod ability;

use error::{Error, Result};
use std::collections::HashMap;
use sfml::graphics::{Sprite, Texture, Color, Image, RenderTexture, Transformable, RenderTarget};
use sfml::system::Vector2f;
use unitypack::engine::mesh::Mesh;
use unitypack::engine::texture::IntoTexture2D;
use cards::{CardClass, CardRarity, CardType};
use assets::Assets;

lazy_static! {
    pub static ref TRANSPARENT_COLOR: Color = {
        Color::rgba(0, 0, 0, 0)
    };
}

trait ImageUtils {
    fn remove_transparency(&mut self);
    fn resize(&mut self, width: u32, height: u32) -> Result<Image>;
}

impl ImageUtils for Image {
    fn remove_transparency(&mut self) {
        for x in 0..self.size().x {
            for y in 0..self.size().y {
                let mut c = self.pixel_at(x, y);
                c.a = 255;
                self.set_pixel(x, y, &c);
            }
        }
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<Image> {
        let mut texture = Texture::from_image(&self).ok_or(Error::SFMLError)?;
        texture.set_smooth(true);
        let mut tmp_sprite = Sprite::with_texture(&texture);
        let scale = Vector2f {
            x: width as f32 / tmp_sprite.local_bounds().width,
            y: height as f32 / tmp_sprite.local_bounds().height,
        };

        tmp_sprite.set_scale(scale);

        let mut canvas = RenderTexture::new(
            width,
            height,
            false,
        ).ok_or(Error::SFMLError)?;
        canvas.set_smooth(true);
        let transparent_color = Color::rgba(0, 0, 0, 0);
        canvas.clear(&transparent_color);
        canvas.draw(&tmp_sprite);
        canvas.display();

        canvas.texture().copy_to_image().ok_or(Error::SFMLError)
    }
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
        _ => Err(Error::NotImplementedError(format!(
            "Card type {:?} is not implemented",
            card_type
        ))),
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
    // gem_image.remove_transparency();
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

    common::build_rarity_gem(&gem_image, &shader_image, mesh, width)
}
