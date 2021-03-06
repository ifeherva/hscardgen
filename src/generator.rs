use assets::{Assets, Fonts};
use cards::*;
use error::{Error, Result};
use unitypack::engine::texture::IntoTexture2D;
use sfml::system::Vector2f;
use sfml::graphics::{Color, Font, Image, RenderTarget, RenderTexture, Sprite, Text, TextStyle,
                     Transformable};
use builder;
use utils::{ImageUtils, SpriteTransforms};

const CARD_ASPECT_RATIO: f32 = 360f32 / 510f32; //764f32 / 1100f32;

pub struct Generator {
    assets: Assets,
    card_defs: CardDb,
}

impl Generator {
    pub fn new(assets_path: &str) -> Result<Self> {
        let generator = Generator {
            assets: Assets::new(assets_path)?,
            card_defs: CardDb::new()?,
        };

        Ok(generator)
    }

    pub fn generate_card(&self, card_id: &str) -> Result<Image> {
        self.generate_card_with_width(card_id, 360)
    }

    // This function is supposed to generate cards of arbitary size but not all subfunctions are ready for that yet
    fn generate_card_with_width(&self, card_id: &str, card_width: usize) -> Result<Image> {
        // obtain card data
        let card = match self.card_defs.cards.get(card_id) {
            Some(c) => c,
            None => {
                return Err(Error::CardNotFoundError);
            }
        };

        let unknown_str = "Unknown";

        let card_name: &str = match &card.name {
            &Some(ref name) => &name.en_us,
            &None => &unknown_str,
        };

        let ref card_type = match card.card_type {
            Some(ref ctype) => ctype,
            None => {
                return Err(Error::InvalidCardError);
            }
        };

        let card_class = card.card_class.as_ref().ok_or(Error::InvalidCardError)?;

        let card_height = (card_width as f32 / CARD_ASPECT_RATIO).ceil() as usize;

        // Create transparent canvas
        let mut canvas = RenderTexture::new(
            (card_width as f32 * 1.13) as u32,
            (card_height as f32 * 1.13) as u32,
            false,
        ).ok_or(Error::SFMLError)?;
        canvas.clear(&builder::TRANSPARENT_COLOR);
        canvas.set_smooth(true);

        // get card frame, TODO: do not add text background, it should come separate with expansion logo
        let card_frame = self.assets.get_card_frame(card_type, card_class)?;
        let mut frame_sprite = Sprite::with_texture(card_frame.texture());

        // frame sprite accordingly
        let scale_factor = card_width as f32 / card_frame.size().x as f32;
        let card_frame_origin = Vector2f::new(28.25f32 * scale_factor, 60f32 * scale_factor);
        frame_sprite.set_scale(Vector2f::new(scale_factor, scale_factor));
        frame_sprite.set_position(Vector2f::new(card_frame_origin.x, card_frame_origin.y));

        // draw card frame
        canvas.draw(&frame_sprite);

        // draw image portrait, TODO: these should be scale-dependent
        self.draw_card_portrait(card_id, &card_type, &card_frame_origin, &mut canvas)?;
        self.draw_portrait_frame(&card_type, &card_class, &card_frame_origin, &mut canvas)?;

        // draw rarity gem
        match card.rarity {
            Some(ref rarity) => {
                match rarity {
                    &CardRarity::FREE => {}
                    _ => {
                        self.draw_rarity_gem(rarity, &card_frame_origin, &mut canvas)?;
                    }
                };
            }
            None => {}
        };

        // draw name banner
        self.draw_name_banner(&card_type, &card_frame_origin, &mut canvas)?;

        // draw mana gem
        self.draw_mana_gem(&card_frame_origin, &mut canvas)?;

        let belwe_raw = self.assets.get_font(&Fonts::Belwe)?;
        let belwe = Font::from_memory(&belwe_raw.data).ok_or(Error::SFMLError)?;

        let mut belwe_text = Text::new("", &belwe, 87);
        belwe_text.set_style(TextStyle::BOLD);
        belwe_text.set_outline_color(&Color::BLACK);
        belwe_text.set_outline_thickness(3f32);
        belwe_text.scale(Vector2f::new(
            1f32 + belwe_raw.pixel_scale,
            1f32 + belwe_raw.pixel_scale,
        ));

        let mut mana_cost: i32 = -1;

        // draw mana cost
        match card.cost {
            Some(cost) => {
                match card.hide_stats {
                    Some(hide_stats) => {
                        if !hide_stats {
                            mana_cost = cost;
                        } else {
                            mana_cost = -1;
                        }
                    }
                    None => {
                        mana_cost = cost;
                    }
                };
            }
            None => {}
        };
        if mana_cost >= 0 {
            belwe_text.set_string(&mana_cost.to_string());
            let bounds = belwe_text.global_bounds();
            belwe_text.set_position(Vector2f::new(
                card_frame_origin.x - (bounds.width / 2f32) - bounds.left + 31.5f32,
                card_frame_origin.y - (bounds.height / 2f32) - bounds.top + 23f32,
            ));
            canvas.draw(&belwe_text);
        }

        // draw card's name
        self.draw_card_name(card_name, &mut belwe_text, &card_frame_origin, &mut canvas)?;

        // render off screen
        canvas.display();
        Ok(canvas.texture().copy_to_image().ok_or(Error::SFMLError)?)
    }

    fn draw_card_portrait(
        &self,
        card_id: &str,
        card_type: &CardType,
        frame_origin: &Vector2f,
        canvas: &mut RenderTexture,
    ) -> Result<()> {
        let portrait = self.assets.get_card_portrait(card_id)?;
        let width = portrait.width;
        let height = portrait.height;
        let mut portrait_img = Image::create_from_pixels(width, height, &portrait.to_image()?)
            .ok_or(Error::SFMLError)?;

        portrait_img.remove_transparency();

        match *card_type {
            CardType::Spell => {
                // draw portrait with shadow
                let portrait_texture = builder::build_ability_portrait(
                    &portrait_img,
                    &self.assets.textures,
                    &self.assets.meshes,
                )?;
                let mut portrait_sprite = Sprite::with_texture(&portrait_texture.texture());

                let portrait_position = Vector2f {
                    x: 36f32 + frame_origin.x,
                    y: 32f32 + frame_origin.y,
                };
                portrait_sprite.set_position(portrait_position);
                canvas.draw(&portrait_sprite);
            }
            _ => {
                return Err(Error::NotImplementedError(format!(
                    "Card type {:?} is not yet implemented",
                    card_type
                )));
            }
        };
        Ok(())
    }

    fn draw_portrait_frame(
        &self,
        card_type: &CardType,
        card_class: &CardClass,
        frame_origin: &Vector2f,
        canvas: &mut RenderTexture,
    ) -> Result<()> {
        match *card_type {
            CardType::Spell => {
                let card_frame = match *card_class {
                    CardClass::Mage => {
                        Assets::catalog_get(&self.assets.textures, "Card_Inhand_Ability_Mage")?
                            .to_texture2d()?
                    }
                    CardClass::Priest => {
                        Assets::catalog_get(&self.assets.textures, "Card_Inhand_Ability_Priest")?
                            .to_texture2d()?
                    }
                    CardClass::Hunter => {
                        Assets::catalog_get(&self.assets.textures, "Card_Inhand_Ability_Hunter")?
                            .to_texture2d()?
                    }
                    CardClass::Warlock => {
                        Assets::catalog_get(&self.assets.textures, "Card_Inhand_Ability_Warlock")?
                            .to_texture2d()?
                    }
                    CardClass::Shaman => {
                        Assets::catalog_get(&self.assets.textures, "Card_Inhand_Ability_Shaman")?
                            .to_texture2d()?
                    }
                    CardClass::Druid => {
                        Assets::catalog_get(&self.assets.textures, "Card_Inhand_Ability_Druid")?
                            .to_texture2d()?
                    }
                    CardClass::Paladin => {
                        Assets::catalog_get(&self.assets.textures, "Card_Inhand_Ability_Paladin")?
                            .to_texture2d()?
                    }
                    CardClass::Rogue => {
                        Assets::catalog_get(&self.assets.textures, "Card_Inhand_Ability_Rogue")?
                            .to_texture2d()?
                    }
                    CardClass::Warrior => {
                        Assets::catalog_get(&self.assets.textures, "Card_Inhand_Ability_Warrior")?
                            .to_texture2d()?
                    }
                    _ => {
                        return Err(Error::NotImplementedError(format!(
                            "Card class {:?} for portrait frame is not yet implemented",
                            card_class
                        )));
                    }
                };
                let card_frame_image = Image::create_from_pixels(
                    card_frame.width,
                    card_frame.height,
                    &card_frame.to_image()?,
                ).ok_or(Error::SFMLError)?;
                let portrait_frame_texture =
                    builder::build_ability_portrait_frame(&card_frame_image, &self.assets.meshes)?;
                let mut portrait_frame_sprite =
                    Sprite::with_texture(&portrait_frame_texture.texture());
                portrait_frame_sprite.flip_vertically();

                let portrait_frame_sprite_position = Vector2f {
                    x: 25f32 + frame_origin.x,
                    y: 20f32 + frame_origin.y,
                };
                portrait_frame_sprite.set_position(portrait_frame_sprite_position);
                canvas.draw(&portrait_frame_sprite);
            }
            _ => {
                return Err(Error::NotImplementedError(format!(
                    "Card type {:?} is not yet implemented",
                    card_type
                )));
            }
        };
        Ok(())
    }

    fn draw_name_banner(
        &self,
        card_type: &CardType,
        frame_origin: &Vector2f,
        canvas: &mut RenderTexture,
    ) -> Result<()> {
        match *card_type {
            CardType::Spell => {
                let banner_texture = builder::build_ability_name_banner(
                    &self.assets.textures,
                    &self.assets.meshes,
                    346,
                )?;
                let mut banner_sprite = Sprite::with_texture(&banner_texture.texture());
                banner_sprite.flip_vertically();
                banner_sprite.set_position(Vector2f {
                    x: 6f32 + frame_origin.x,
                    y: 221f32 + frame_origin.y,
                });
                canvas.draw(&banner_sprite);
            }
            _ => {
                return Err(Error::NotImplementedError(format!(
                    "Card type {:?} is not yet implemented",
                    card_type
                )));
            }
        };
        Ok(())
    }

    fn draw_mana_gem(&self, frame_origin: &Vector2f, canvas: &mut RenderTexture) -> Result<()> {
        let mana_gem = builder::build_mana_gem(&self.assets.textures, &self.assets.meshes, 94)?;
        let mut mana_gem_sprite = Sprite::with_texture(&mana_gem.texture());
        mana_gem_sprite.flip_horizontally();
        mana_gem_sprite.set_position(Vector2f {
            x: frame_origin.x - 13f32,
            y: frame_origin.y - 20f32,
        });
        canvas.draw(&mana_gem_sprite);
        Ok(())
    }

    fn draw_rarity_gem(
        &self,
        rarity: &CardRarity,
        frame_origin: &Vector2f,
        canvas: &mut RenderTexture,
    ) -> Result<()> {
        // draw socket
        let rarity_gem_socket =
            builder::build_rarity_gem_socket(&self.assets.textures, &self.assets.meshes, 66)?;
        let mut rarity_gem_socket_sprite = Sprite::with_texture(&rarity_gem_socket.texture());
        rarity_gem_socket_sprite.flip_vertically();
        rarity_gem_socket_sprite.set_position(Vector2f {
            x: 143f32 + frame_origin.x,
            y: 279f32 + frame_origin.y,
        });
        canvas.draw(&rarity_gem_socket_sprite);

        // draw gem
        let rarity_gem =
            builder::build_rarity_gem(&self.assets.textures, &self.assets.meshes, rarity, 29)?;
        let mut rarity_gem_sprite = Sprite::with_texture(&rarity_gem.texture());
        rarity_gem_sprite.flip_vertically();
        rarity_gem_sprite.set_position(Vector2f {
            x: 163f32 + frame_origin.x,
            y: 291f32 + frame_origin.y,
        });
        canvas.draw(&rarity_gem_sprite);
        Ok(())
    }

    fn draw_card_name(
        &self,
        card_name: &str,
        text: &mut Text,
        frame_origin: &Vector2f,
        canvas: &mut RenderTexture,
    ) -> Result<()> {
        let name_texture = builder::build_name_texture(card_name, text)?;

        let card_name = builder::build_card_name(name_texture.texture(), &self.assets.meshes, 318)?;
        let mut card_name_sprite = Sprite::with_texture(&card_name.texture());
        card_name_sprite.flip_vertically();
        card_name_sprite.set_position(Vector2f {
            x: 20f32 + frame_origin.x,
            y: 226f32 + frame_origin.y,
        });
        canvas.draw(&card_name_sprite);
        Ok(())
    }
}
