use error::{Result, Error};
use serde_xml_rs::deserialize;

const CARDDEF_DATA: &'static [u8] = include_bytes!("../res/CardDefs.xml");

#[derive(Debug, Deserialize)]
pub struct CardDefs {

}

impl CardDefs {

    pub fn new() -> Result<Self> {

        let carddefs: CardDefs = deserialize(CARDDEF_DATA)?;

        Ok(carddefs)
    }
}