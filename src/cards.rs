use error::Result;
use serde_xml_rs::deserialize;

const CARDDEF_DATA: &'static [u8] = include_bytes!("../res/CardDefs.xml");

#[derive(Debug, Deserialize)]
pub struct CardDefs {
    pub build: String,
    
    #[serde(rename = "Entity")]
    pub entities: Vec<Entity>,
}

#[derive(Debug, Deserialize)]
pub struct Entity {
    #[serde(rename = "CardID")]
    pub card_id: String,
    #[serde(rename = "ID")]
    pub id: String,
    pub version: String,

    #[serde(rename = "Tag")]
    pub tags: Option<Vec<Tag>>,
}

#[derive(Debug, Deserialize)]
pub struct Tag {
    #[serde(rename = "enumID")]
    pub enum_id: String,

    name: String,

    #[serde(rename = "type")]
    ttype: String,
    
    value: Option<String>,
}


impl CardDefs {
    pub fn new() -> Result<Self> {

        let carddefs: CardDefs = deserialize(CARDDEF_DATA)?;

        Ok(carddefs)
    }
}