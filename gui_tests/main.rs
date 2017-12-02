extern crate hscardgen;
extern crate sfml;

use hscardgen::generator::*;
use sfml::graphics::Texture;
use std::env;

fn main() {
    let generator = Generator::new("/Applications/Hearthstone/Data/OSX/").unwrap();
    let texture = Texture::from_image(&generator.generate_card("CS2_031").unwrap()).unwrap();
    //let sprite = Sprite::with_texture(&texture);

    let image = texture.copy_to_image().unwrap();
    let mut path = env::home_dir().unwrap().to_str().unwrap().to_owned();
    path.push_str("/Downloads/test.png");
    image.save_to_file(&path);

    /*let mut window = RenderWindow::new(
        VideoMode::new(800, 1200, 32),
        "AT_001",
        Style::default(),
        &ContextSettings::default(),
    ).unwrap();

    while window.is_open() {
        // consume event queue
        while {
            match window.poll_event() {
                Some(event) => {
                    match event {
                        Event::Closed => {
                            window.close();
                        }
                        _ => {}
                    };
                    false
                }
                None => false,
            }
        } {}

        // Clear screen
        window.clear(&Color::white());

        window.draw(&sprite);

        // Update the window
        window.display();
    }*/
}
