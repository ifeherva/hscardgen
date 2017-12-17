extern crate byteorder;
extern crate glob;
#[macro_use]
extern crate lazy_static;
extern crate rayon;
extern crate serde_json;
extern crate sfml;
extern crate time;
extern crate unitypack;

#[macro_use]
extern crate serde_derive;

pub mod error;
pub mod generator;
mod builder;
mod assets;
mod cards;
mod resources;

#[cfg(test)]
mod tests {

    use generator::*;
    use std::env;
    use time::PreciseTime;

    const CARD_ID_ICE_LANCE: &str = "CS2_031";
    const CARD_ID_ICE_BARRIER: &str = "EX1_289";

    #[test]
    fn generate_mage_spell() {
        let start = PreciseTime::now();
        let generator = Generator::new("/Applications/Hearthstone/Data/OSX/").unwrap();
        let end = PreciseTime::now();
        println!("Generator loading took {} seconds.", start.to(end));

        let start = PreciseTime::now();
        let card_image = generator.generate_card(CARD_ID_ICE_BARRIER).unwrap();
        let end = PreciseTime::now();
        println!("Card image generation took {} seconds.", start.to(end));

        let mut path = env::home_dir().unwrap().to_str().unwrap().to_owned();
        path.push_str("/Downloads/test.png");
        card_image.save_to_file(&path);
    }
}
