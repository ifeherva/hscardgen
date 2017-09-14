mod ability;

use error::{Error, Result};
use std::collections::HashMap;
use sfml::graphics::Texture;
use unitypack::engine::mesh::Mesh;
use cards::{CardClass, CardType};

use builder::ability::build_ability_frame_for_class;

pub fn build_frame(
    texture_map: &HashMap<String, String>,
    meshes_map: &HashMap<String, Mesh>,
    card_class: &CardClass,
    card_type: &CardType,
) -> Result<Texture> {
    match *card_type {
        CardType::Spell | CardType::Enchantment => {
            build_ability_frame_for_class(texture_map, meshes_map, card_class)
        }
        _ => Err(Error::NotImplementedError(
            format!("Card type {:?} is not implemented", card_type),
        )),
    }
}
