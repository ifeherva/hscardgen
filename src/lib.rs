extern crate byteorder;
extern crate glob;
extern crate rayon;
extern crate serde_json;
extern crate sfml;
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

    #[test]
    fn it_works() {
        let generator = Generator::new("/Applications/Hearthstone/Data/OSX/").unwrap();
        generator.generate_card("AT_001").unwrap();
    }
}
