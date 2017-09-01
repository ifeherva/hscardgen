use error;
use serde_json;
use std::collections::HashMap;

const CARDDEF_DATA: &'static [u8] = include_bytes!("../res/cards.json");

#[derive(Debug, Deserialize)]
pub struct CardDb {
    pub cards: HashMap<String, Card>,
}

#[derive(Debug, Deserialize)]
pub struct Card {
    #[serde(rename = "cardClass")] pub card_class: Option<CardClass>,
    pub id: String,
    pub name: Option<Name>,
    #[serde(rename = "playerClass")] pub player_class: Option<CardClass>,
    pub rarity: Option<CardRarity>,
    pub set: Option<CardSet>,
    #[serde(default)] pub collectible: bool,
    pub cost: Option<i32>,
    #[serde(rename = "type")] pub card_type: Option<CardType>,
}

#[derive(Debug, Deserialize)]
pub struct Name {
    #[serde(rename = "enUS")] pub en_us: String,
    #[serde(rename = "deDE")] pub de_de: Option<String>,
    #[serde(rename = "esES")] pub es_es: Option<String>,
    #[serde(rename = "esMX")] pub es_mx: Option<String>,
    #[serde(rename = "frFR")] pub fr_fr: Option<String>,
    #[serde(rename = "itIT")] pub it_it: Option<String>,
    #[serde(rename = "jaJP")] pub ja_jp: Option<String>,
    #[serde(rename = "koKR")] pub ko_kr: Option<String>,
    #[serde(rename = "plPL")] pub pl_pl: Option<String>,
    #[serde(rename = "ptBR")] pub pt_br: Option<String>,
    #[serde(rename = "ruRU")] pub ru_ru: Option<String>,
    #[serde(rename = "thTH")] pub th_th: Option<String>,
    #[serde(rename = "zhCN")] pub zh_cn: Option<String>,
    #[serde(rename = "zhTW")] pub zh_tw: Option<String>,
}

#[derive(Debug, Deserialize)]
pub enum CardClass {
    #[serde(rename = "NEUTRAL")] Neutral,
    #[serde(rename = "DRUID")] Druid,
    #[serde(rename = "DEATHKNIGHT")] Deathknight,
    #[serde(rename = "SHAMAN")] Shaman,
    #[serde(rename = "PALADIN")] Paladin,
    #[serde(rename = "WARRIOR")] Warrior,
    #[serde(rename = "PRIEST")] Priest,
    #[serde(rename = "HUNTER")] Hunter,
    #[serde(rename = "MAGE")] Mage,
    #[serde(rename = "WARLOCK")] Warlock,
    #[serde(rename = "ROGUE")] Rogue,
    #[serde(rename = "DREAM")] Dream,
}

#[derive(Debug, Deserialize)]
pub enum CardRarity {
    FREE,
    COMMON,
    RARE,
    EPIC,
    LEGENDARY,
}

#[derive(Debug, Deserialize)]
pub enum CardType {
    MINION,
    SPELL,
    WEAPON,
    ENCHANTMENT,
    HERO_POWER,
    HERO,
}
#[derive(Debug, Deserialize)]
pub enum CardSet {
    CORE,
    EXPERT1,
    NAXX,
    TB,
    GANGS,
    ICECROWN,
    UNGORO,
    CREDITS,
    GVG,
    BRM,
    LOE,
    KARA,
    TGT,
    #[serde(rename = "HERO_SKINS")] HeroSkins,
    OG,
    MISSIONS,
    HOF,
}

impl CardDb {
    pub fn new() -> error::Result<Self> {
        let cards: Vec<Card> = serde_json::from_reader(CARDDEF_DATA)?;
        Ok(CardDb {
            cards: cards.into_iter().fold(HashMap::new(), |mut map, card| {
                {
                    map.insert(card.id.clone(), card);
                }
                map
            }),
        })
    }
}
