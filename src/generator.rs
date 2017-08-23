
use assets::Assets;
use unitypack::error::Result;

pub struct Generator {
    assets: Assets,
}

impl Generator {
    pub fn new(assets_path: &str) -> Result<Self> {
        Ok(Generator { assets: Assets::new(assets_path)? })
    }

    pub fn generate_card(card_id: &str) {}
}
