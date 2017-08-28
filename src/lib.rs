extern crate rayon;
extern crate unitypack;
extern crate glob;

pub mod error;
pub mod generator;
mod assets;

#[cfg(test)]
mod tests {

    use generator::*;

    #[test]
    fn it_works() {
        let generator = Generator::new("/Applications/Hearthstone/Data/OSX/");
    }
}
