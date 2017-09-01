use assets::Assets;
use cards::*;
use error::{Error, Result};
use sfml::graphics::{Color, Image, RenderTarget, RenderTexture, Sprite, Texture, Transformable};

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

        let card_frame =
            Image::from_memory(self.assets.get_card_frame(card_type, card_class)?).unwrap();
        let card_frame_texture = Texture::from_image(&card_frame).unwrap();

        // we draw on this canvas
        let mut canvas =
            RenderTexture::new(card_frame.size().x, card_frame.size().y, false).unwrap();
        canvas.clear(&Color::white());

        // draw card frame
        canvas.draw(&Sprite::with_texture(&card_frame_texture));

        // draw mana gem
        let mana_gem = Image::from_memory(self.assets.get_card_asset("MANA_GEM")?).unwrap();
        let mana_gem_texture = Texture::from_image(&mana_gem).unwrap();
        let mut mana_gem_sprite = Sprite::with_texture(&mana_gem_texture);
        mana_gem_sprite.move2f(24f32, 75f32);
        canvas.draw(&mana_gem_sprite);

        // render off screen
        canvas.display();
        Ok(canvas.texture().copy_to_image().unwrap())
    }
}
