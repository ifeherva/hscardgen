use assets::Assets;
use cards::*;
use error::{Error, Result};
use sfml::graphics::{text_style, Color, Font, Image, RenderTarget, RenderTexture, Sprite, Text,
                     Texture, Transformable};
use sfml::system::Vector2f;

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

        let card_frame =
            Image::from_memory(self.assets.get_card_frame(card_type, card_class)?).unwrap();
        let card_frame_texture = Texture::from_image(&card_frame).unwrap();

        // we draw on this canvas
        let mut canvas =
            RenderTexture::new(card_frame.size().x, card_frame.size().y, false).unwrap();
        canvas.clear(&transparent_color);

        // draw card frame
        canvas.draw(&Sprite::with_texture(&card_frame_texture));

        // draw mana gem
        let mana_gem = Image::from_memory(self.assets.get_card_asset("MANA_GEM")?).unwrap();
        let mana_gem_texture = Texture::from_image(&mana_gem).unwrap();
        let mut mana_gem_sprite = Sprite::with_texture(&mana_gem_texture);
        mana_gem_sprite.move2f(24f32, 75f32);
        canvas.draw(&mana_gem_sprite);

        let belwe_raw = self.assets.get_font(FONT_BELWE)?;
        let belwe = Font::from_memory(&belwe_raw.data).ok_or(Error::ObjectTypeError)?;

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
}
