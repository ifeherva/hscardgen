
use assets::Assets;
use cards::*;
use error::{Result, Error};

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

    pub fn generate_card(&self, card_id: &str) -> Result<()> {
        let card = match self.card_defs.cards.get(card_id){
            Some(c) => c,
            None => {
                return Err(Error::CardNotFoundError);
            },
        };

        /*let cardback = generate_cardback_key(&card.card_type, match &card.player_class {
            &Some(ref pc) => pc,
            &None => {
                return Err(Error::InvalidCardError);
            },
        });*/

        Ok(())
    }
}

fn generate_cardback_key(cardtype: &CardType, player_class: &CardClass) {

}
