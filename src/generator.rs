use assets::Assets;
use cards::*;
use error::{Result, Error};
use sfml::graphics::{Texture, Image};

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

    pub fn generate_card(&self, card_id: &str) -> Result<Texture> {
        // obtain card data
        let card = match self.card_defs.cards.get(card_id){
            Some(c) => c,
            None => {
                return Err(Error::CardNotFoundError);
            },
        };

        let ref card_type = match card.card_type {
            Some(ref ctype) => ctype,
            None => {
                return Err(Error::InvalidCardError);
            },
        };

        let ref card_class = match card.card_class {
            Some(ref class) => class,
            None => {
                return Err(Error::InvalidCardError);
            },
        };

        let card_frame = self.assets.get_card_frame(card_type, card_class)?;
        
        let image = Image::from_memory(card_frame).unwrap();
        Ok(Texture::from_image(&image).unwrap())
    }
}