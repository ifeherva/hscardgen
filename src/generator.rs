
use assets::Assets;
use cards::CardDefs;
use error::Result;

pub struct Generator {
    assets: Assets,
    card_defs: CardDefs,
}

impl Generator {
    pub fn new(assets_path: &str) -> Result<Self> {
        let generator = Generator {
            assets: Assets::new(assets_path)?,
            card_defs: CardDefs::new()?,
        };
        
        println!("build: {}, entities: {}",generator.card_defs.build, generator.card_defs.entities.len());
        Ok(generator)
    }

    pub fn generate_card(&self, card_id: &str) {}
}
