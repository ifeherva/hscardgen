use assets::Assets;
use cards::*;
use error::{Error, Result};
use unitypack::engine::texture::IntoTexture2D;
use sfml::system::{Vector2, Vector2f};
use sfml::graphics::{Color, Font, Image, IntRect, RenderTarget, RenderTexture, Sprite, Text,
                     TextStyle, Texture, Transformable};
use builder;

const FONT_BELWE: &'static str = "Belwe";
const FONT_BELWE_OUTLINE: &'static str = "Belwe_Outline";
const FONT_BLIZZARDGLOBAL: &'static str = "BlizzardGlobal";
const FONT_FRANKLINGOTHIC: &'static str = "FranklinGothic";

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

        //let rarity = card.rarity.as_ref().ok_or(Error::InvalidCardError)?;

        let transparent_color = Color::rgba(0, 0, 0, 0);
        let card_size = Vector2 { x: 764, y: 1100 };

        // Create transparent canvas
        let mut canvas =
            RenderTexture::new(card_size.x, card_size.y, false).ok_or(Error::SFMLError)?;
        canvas.clear(&transparent_color);
        canvas.set_smooth(true);

        // draw card frame
        self.draw_card_frame(&card_type, &card_class, &mut canvas)?;

        // draw image portrait
        self.draw_card_portrait(card_id, &card_type, &mut canvas)?;

        self.draw_portrait_frame(&card_type, &card_class, &mut canvas)?;

        // draw mana gem
        let mana_gem =
            Image::from_memory(self.assets.get_card_asset("MANA_GEM")?).ok_or(Error::SFMLError)?;
        let mut mana_gem_texture = Texture::from_image(&mana_gem).ok_or(Error::SFMLError)?;
        mana_gem_texture.set_smooth(true);
        let mut mana_gem_sprite = Sprite::with_texture(&mana_gem_texture);
        mana_gem_sprite.move_(Vector2f::new(24f32, 75f32));
        canvas.draw(&mana_gem_sprite);

        let belwe_raw = self.assets.get_font(FONT_BELWE)?;
        let belwe = Font::from_memory(&belwe_raw.data).ok_or(Error::SFMLError)?;

        // draw mana cost
        match card.cost {
            Some(cost) => {
                let center = Vector2f::new(112f32, 161f32);

                let mut mana_text = Text::new(&cost.to_string(), &belwe, 160);
                mana_text.set_style(TextStyle::BOLD);
                mana_text.set_outline_color(&Color::BLACK);
                mana_text.set_outline_thickness(4f32);
                mana_text.scale(Vector2f::new(
                    1f32 + belwe_raw.pixel_scale,
                    1f32 + belwe_raw.pixel_scale,
                ));
                let bounds = mana_text.global_bounds();
                mana_text.set_position(Vector2f::new(
                    center.x - (bounds.width / 2f32) - bounds.left,
                    center.y - (bounds.height / 2f32) - bounds.top,
                ));
                canvas.draw(&mana_text);
            }
            None => {}
        };

        // render off screen
        canvas.display();
        Ok(canvas.texture().copy_to_image().ok_or(Error::SFMLError)?)
    }

    fn draw_card_frame(
        &self,
        card_type: &CardType,
        card_class: &CardClass,
        canvas: &mut RenderTexture,
    ) -> Result<()> {
        let card_frame = self.assets.get_card_frame(card_type, card_class)?;
        let width = card_frame.size().x;
        let mut frame_sprite = Sprite::with_texture(card_frame.texture());
        frame_sprite.flip_texture();
        frame_sprite.set_scale(Vector2f::new(675f32 / width as f32, 675f32 / width as f32));
        frame_sprite.set_position(Vector2f::new(53f32, 113f32));
        canvas.draw(&frame_sprite);
        Ok(())
    }

    fn draw_card_portrait(
        &self,
        card_id: &str,
        card_type: &CardType,
        canvas: &mut RenderTexture,
    ) -> Result<()> {
        let portrait = self.assets.get_card_portrait(card_id)?;
        let width = portrait.width;
        let height = portrait.height;
        let mut portrait_img = Image::create_from_pixels(width, height, &portrait.to_image()?)
            .ok_or(Error::SFMLError)?;

        portrait_img.flip_vertically();

        match *card_type {
            CardType::Spell => {
                // draw portrait with shadow
                let portrait_texture = builder::build_ability_portrait(&portrait_img, &self.assets.textures, &self.assets.meshes)?;        
                let mut portrait_sprite = Sprite::with_texture(&portrait_texture.texture());
                portrait_sprite.set_scale(Vector2f::new(528f32 / width as f32, 528f32 / width as f32));

                let portrait_position = Vector2f {
                    x: 130f32,
                    y: 175f32,
                };
                portrait_sprite.set_position(portrait_position);
                canvas.draw(&portrait_sprite);
            },
            _ => {
                return Err(Error::NotImplementedError(
                    format!("Card type {:?} is not yet implemented", card_type),
                ));
            }
        };
        Ok(())
    }

    fn draw_portrait_frame(&self,
        card_type: &CardType,
        card_class: &CardClass,
        canvas: &mut RenderTexture,
    ) -> Result<()>  {
        match *card_type {
            CardType::Spell => {
                let card_frame = match *card_class {
                    CardClass::Mage => {
                        Assets::catalog_get(&self.assets.textures, "Card_Inhand_Ability_Mage")?.to_texture2d()?
                    }
                    CardClass::Priest => {
                        Assets::catalog_get(&self.assets.textures, "Card_Inhand_Ability_Priest")?.to_texture2d()?
                    }
                    _ => {
                        return Err(Error::NotImplementedError(
                            format!("Card class {:?} is not yet implemented", card_class),
                        ));
                    }
                };
                let card_frame_image = Image::create_from_pixels(card_frame.width, card_frame.height, &card_frame.to_image()?)
                .ok_or(Error::SFMLError)?;
                let portrait_frame_texture = builder::build_ability_portrait_frame(&card_frame_image, &self.assets.meshes)?;

                let mut portrait_frame_sprite = Sprite::with_texture(&portrait_frame_texture.texture());
                portrait_frame_sprite.flip_texture();
                portrait_frame_sprite.set_scale(Vector2f::new(675f32 / 338f32, 675f32 / 338f32));
                let portrait_frame_sprite_position = Vector2f {
                    x: 100f32,
                    y: 147f32,
                };
                portrait_frame_sprite.set_position(portrait_frame_sprite_position);
                canvas.draw(&portrait_frame_sprite);
            },
            _ => {
                return Err(Error::NotImplementedError(
                    format!("Card type {:?} is not yet implemented", card_type),
                ));
            }
        };
        Ok(())
    }
}

trait SpriteTransforms {
    fn flip_texture(&mut self);
}

impl<'s> SpriteTransforms for Sprite<'s> {
    fn flip_texture(&mut self) {
        let texture_rect = self.texture_rect();
        self.set_texture_rect(&IntRect::new(
            0,
            texture_rect.height,
            texture_rect.width,
            -1 * texture_rect.height,
        ));
    }
}
