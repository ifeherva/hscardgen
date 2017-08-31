extern crate rayon;
extern crate unitypack;
extern crate glob;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

pub mod error;
pub mod generator;
mod assets;
mod cards;

#[cfg(test)]
mod tests {

    use generator::*;

    #[test]
    fn it_works() {
        let generator = Generator::new("/Applications/Hearthstone/Data/OSX/").unwrap();
        generator.generate_card("AT_001").unwrap();
    }
}
