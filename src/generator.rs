use assets::Assets;
use cards::*;
use error::{Error, Result};
use sfml::system::{Vector2, Vector2f};
use sfml::graphics::{text_style, Color, Font, Image, IntRect, RenderTarget, RenderTexture, Sprite,
                     Text, Texture, Transformable};

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

        let ref card_class = match card.card_class {
            Some(ref class) => class,
            None => {
                return Err(Error::InvalidCardError);
            }
        };

        let transparent_color = Color::rgba(0, 0, 0, 0);
        let card_size = Vector2 { x: 764, y: 1100 };

        // Create transparent canvas
        let mut canvas =
            RenderTexture::new(card_size.x, card_size.y, false).ok_or(Error::SFMLError)?;
        canvas.clear(&transparent_color);

        // draw image portrait
        self.draw_card_portrait(card_id, &card_type, &mut canvas)?;

        // draw card frame
        self.draw_card_frame(&card_type, &card_class, &mut canvas)?;

        // draw mana gem
        let mana_gem =
            Image::from_memory(self.assets.get_card_asset("MANA_GEM")?).ok_or(Error::SFMLError)?;
        let mana_gem_texture = Texture::from_image(&mana_gem).ok_or(Error::SFMLError)?;
        let mut mana_gem_sprite = Sprite::with_texture(&mana_gem_texture);
        mana_gem_sprite.move2f(24f32, 75f32);
        canvas.draw(&mana_gem_sprite);

        let belwe_raw = self.assets.get_font(FONT_BELWE)?;
        let belwe = Font::from_memory(&belwe_raw.data).ok_or(Error::SFMLError)?;

        // draw mana cost
        match card.cost {
            Some(cost) => {
                let center = Vector2f::new(112f32, 161f32);

                let mut mana_text = Text::new_init(&cost.to_string(), &belwe, 160);
                mana_text.set_style(text_style::BOLD);
                mana_text.set_outline_color(&Color::black());
                mana_text.set_outline_thickness(4f32);
                mana_text.scale2f(1f32 + belwe_raw.pixel_scale, 1f32 + belwe_raw.pixel_scale);
                let bounds = mana_text.global_bounds();
                mana_text.set_position2f(
                    center.x - (bounds.width / 2f32) - bounds.left,
                    center.y - (bounds.height / 2f32) - bounds.top,
                );
                canvas.draw(&mana_text);
            }
            None => {}
        };

        // render off screen
        canvas.display();
        Ok(canvas.texture().copy_to_image().unwrap())
    }

    fn draw_card_frame(
        &self,
        card_type: &CardType,
        card_class: &CardClass,
        canvas: &mut RenderTexture,
    ) -> Result<()> {
        // TODO: card frame is currently loaded from a texture, we want to read it from assets
        let card_frame = Image::from_memory(self.assets.get_card_frame(card_type, card_class)?)
            .ok_or(Error::SFMLError)?;
        let texture = Texture::from_image(&card_frame).ok_or(Error::SFMLError)?;
        canvas.draw(&Sprite::with_texture(&texture));
        Ok(())
    }

    fn draw_card_portrait(
        &self,
        card_id: &str,
        card_type: &CardType,
        canvas: &mut RenderTexture,
    ) -> Result<()> {
        let portrait = self.assets.get_card_texture(card_id)?;
        let width = portrait.width;
        let height = portrait.height;
        let portrait_img = Image::create_from_pixels(width, height, &portrait.to_image()?)
            .ok_or(Error::SFMLError)?;
        let portrait_texture = Texture::from_image(&portrait_img).ok_or(Error::SFMLError)?;
        let mut portrait_sprite = Sprite::with_texture(&portrait_texture);

        // flip image
        portrait_sprite.flip_texture();
        portrait_sprite.set_scale2f(529f32 / width as f32, 529f32 / width as f32);

        let portrait_position = match *card_type {
            CardType::SPELL => Vector2f {
                x: 123f32,
                y: 123f32,
            },
            _ => {
                return Err(Error::NotImplementedError(
                    format!("Card type {:?} is not yet implemented", card_type),
                ));
            }
        };
        portrait_sprite.set_position(&portrait_position);
        canvas.draw(&portrait_sprite);
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
