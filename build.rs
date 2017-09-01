extern crate curl;
extern crate rayon;

use rayon::prelude::*;
use curl::easy::Easy;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

//static CARDDEF_URL: &'static str = "https://raw.githubusercontent.com/HearthSim/hsdata/master/CardDefs.xml";
//static CARDDEF_PATH: &'static str = "./res/CardDefs.xml";
static CARDDEF_URL: &'static str = "https://api.hearthstonejson.com/v1/20457/all/cards.json";
static CARDDEF_PATH: &'static str = "./res/cards.json";

// spells
static FRAME_SPELL_MAGE_URL: &'static str = "https://raw.githubusercontent.com/HearthSim/Sunwell/master/assets/frame-spell-mage.png";
static FRAME_SPELL_MAGE_PATH: &'static str = "./res/frame-spell-mage.png";

// assets
static MANA_GEM_URL: &'static str = "https://raw.githubusercontent.com/HearthSim/Sunwell/master/assets/cost-mana.png";
static MANA_GEM_PATH: &'static str = "./res/cost-mana.png";

struct DlData {
    name: &'static str,
    url: &'static str,
    destination: &'static str,
}

impl DlData {
    fn new(name: &'static str, url: &'static str, destination: &'static str) -> Self {
        DlData {
            name: name,
            url: url,
            destination: destination,
        }
    }
}

fn main() {
    let file_list = vec![
        DlData::new("Card definitions", CARDDEF_URL, CARDDEF_PATH),
        DlData::new("Mana gem", MANA_GEM_URL, MANA_GEM_PATH),
        DlData::new(
            "Mage spell frame",
            FRAME_SPELL_MAGE_URL,
            FRAME_SPELL_MAGE_PATH,
        ),
    ];

    file_list
        .par_iter()
        .map(|x| { download_resource(x); })
        .count();
}

fn download_resource(res: &DlData) {
    // Download card definitons if file does not exist
    if !Path::new(res.destination).exists() {
        println!("Downloading {}...", res.name);
        let mut dst = Vec::new();
        let mut easy = Easy::new();
        easy.url(res.url).unwrap();
        {
            let mut transfer = easy.transfer();
            transfer
                .write_function(|data| {
                    dst.extend_from_slice(data);
                    Ok(data.len())
                })
                .unwrap();
            transfer.perform().unwrap();
        }

        let mut file = File::create(res.destination).unwrap();
        file.write(&dst).unwrap();
    }
}
