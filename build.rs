extern crate curl;

use curl::easy::Easy;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

static CARDDEF_URL: &'static str = "https://raw.githubusercontent.com/HearthSim/hsdata/master/CardDefs.xml";
static CARDDEF_PATH: &'static str = "/res/CardDefs.xml";

fn main() {
    // Download card definitons if file does not exist
    if Path::new(CARDDEF_PATH).exists() {
        let mut dst = Vec::new();
        let mut easy = Easy::new();
        easy.url(CARDDEF_URL).unwrap();
        {
            let mut transfer = easy.transfer();
            transfer.write_function(|data| {
                    dst.extend_from_slice(data);
                    Ok(data.len())
            }).unwrap();
        transfer.perform().unwrap();
        }

        let mut file = File::create(CARDDEF_PATH).unwrap();
        file.write(&dst).unwrap();
    }
}