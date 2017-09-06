use unitypack::assetbundle::AssetBundle;
use unitypack::object::{ObjectValue, ObjectPointer, ObjectInfo};
use unitypack::engine::texture::IntoTexture2D;
use unitypack::engine::text::IntoTextAsset;
use unitypack::engine::object::IntoGameObject;
use unitypack::engine::font::{Font, IntoFont, IntoFontDef};
use unitypack::engine::EngineObject;
use unitypack::asset::Asset;
use unitypack::assetbundle::Signature;
use error::{Error, Result};
use cards::*;
use std::collections::HashMap;
use glob::glob;
use rayon::prelude::*;
use resources::*;

/// Stores graphic elements to construct cards
pub struct Assets {
    texture_cache: HashMap<String, String>, // object_name -> file|asset
    card_frames: HashMap<String, &'static [u8]>,
    card_assets: HashMap<String, &'static [u8]>,
    fonts: HashMap<String, Font>,
}

struct UnpackDef {
    pub file_paths: Vec<String>,
    pub object_types: Vec<String>,
}

trait Contains {
    fn contains(&self, item: &str) -> bool;
}

impl Contains for Vec<String> {
    fn contains(&self, item: &str) -> bool {
        for i in self {
            if *i == *item {
                return true;
            }
        }
        false
    }
}

impl UnpackDef {
    fn new(path: &str, object_types: Vec<String>) -> Self {
        UnpackDef {
            file_paths: glob(path)
                .unwrap()
                .map(|x| x.unwrap().to_str().unwrap().to_string())
                .collect::<Vec<String>>(),
            object_types: object_types,
        }
    }
}

// -> cards, textures
fn extract_textures(unpackdef: &UnpackDef) -> Result<(HashMap<String, String>, HashMap<String, ObjectPointer>)> {
    unpackdef
        .file_paths
        .par_iter()
        .fold(|| Ok((HashMap::new(),HashMap::new())) , |map, asset_path| {
            let map_pair = match map {
                    Ok(m) => m,
                    Err(e) => {
                        return Err(e);
                    },
                };
                let mut cards = map_pair.0;
                let mut textures = map_pair.1;
            {
                let mut asset_bundle = match AssetBundle::load_from_file(asset_path) {
                    Ok(asset_bundle) => asset_bundle,
                    Err(e) => {
                        println!("Skipping file at {} due to error: {:?}", asset_path, e);
                        return Ok((cards, textures));
                    }
                };

                for i in 0..asset_bundle.assets.len() {
                    match asset_bundle.resolve_asset(i) {
                        Err(e) => {
                            println!("Skipping file at {} due to error: {:?}", asset_path, e);
                            return Ok((cards, textures));
                        }
                        _ => {}
                    };
                    let asset = &mut asset_bundle.assets[i];
                    let objects = &asset.objects;

                    for (_, ref obj) in objects.iter() {
                        if unpackdef.object_types.contains(&obj.type_name) {
                            let engine_object =
                                match obj.read_signature(asset, &mut asset_bundle.signature) {
                                    Ok(o) => o,
                                    Err(_) => {continue;},
                                };
                            
                            if obj.type_name == "AssetBundle" {
                                match process_asset_bundle(engine_object, &mut textures) {
                                    Ok(_) => {},
                                    Err(_) => {continue;},
                                };
                                
                            } else if obj.type_name == "GameObject" {
                                match process_game_object(engine_object, &mut cards, objects, asset, &mut asset_bundle.signature) {
                                    Ok(_) => {},
                                    Err(_) => {continue;},
                                };
                            }
                        }
                    }
                }
            }
            Ok((cards, textures) )
        })
        .reduce(|| Ok((HashMap::new(),HashMap::new())), |pair, newpair| {
            let mut p = pair?;
            let np = newpair?;
            p.0.extend(np.0);
            p.1.extend(np.1);
            Ok(p)
        }) 


}

fn process_game_object(engine_object: ObjectValue, cards: &mut HashMap<String, String>, objects: &HashMap<i64, ObjectInfo>, asset: &Asset, signature: &mut Signature) -> Result<()> {
    let d = match engine_object {
        ObjectValue::EngineObject(engine_object) => {
            engine_object.to_gameobject()?
        }
        _ => {
            return Err(Error::ObjectTypeError);
        }
    };

    let ref card_id = d.object.name;

    if vec!["CardDefTemplate".to_string(), "HiddenCard".to_string()].contains(card_id) {
        // not a real card
        //cards.insert(cardid = (path: "", tile: nil)
        return Ok(())
    }

    if d.component.len() < 2 {
        // not a real card
        return Err(Error::ObjectTypeError);
    }

    let carddef = match &d.component[1] {
        &ObjectValue::Pair(ref p) => {
            match *p.1 {
                ObjectValue::ObjectPointer(ref op) => {
                    if op.file_id != 0 {
                        return Err(Error::ObjectTypeError);
                    }
                    match objects[&op.path_id].read_signature(asset, signature)? {
                        ObjectValue::EngineObject(engine_object) => engine_object,
                        _ => {
                            return Err(Error::ObjectTypeError);
                        }
                    }
                },
                _ => {
                    return Err(Error::ObjectTypeError);
                }
            }
        },
        _ => {
            return Err(Error::ObjectTypeError);
        }
    };
    
    let mut path = carddef.map.get(&"m_PortraitTexturePath".to_string()).ok_or(Error::ObjectTypeError)?.to_string()?;

    if path == "" {
        return Err(Error::ObjectTypeError);
    }

    path = format!("final/{}", path);

    cards.insert(card_id.clone(), path.to_lowercase());

    Ok(())
}

fn process_asset_bundle(engine_object: ObjectValue, textures: &mut HashMap<String, ObjectPointer>) -> Result<()> {
    match engine_object {
        ObjectValue::EngineObject(mut engine_object) => {
            let mut items = engine_object.map.remove(&"m_Container".to_string()).ok_or(Error::ObjectTypeError)?.into_vec()?;
                                        
            for item in items.drain(0..) {
                let pair = match item.into_pair() {
                    Ok(o) => o,
                    _ => {continue;}
                };
                let mut path = match pair.0.to_string() {
                    Ok(o) => o,
                    _ => {continue;}
                };
                let mut dict = match *pair.1 {
                    ObjectValue::Map(map) => map,
                    _ => {
                        continue;
                    },
                };

                let asset = match dict.remove(&"asset".to_string()) {
                    Some(a) => {
                        match a {
                            ObjectValue::ObjectPointer(op) => op,
                            _ => {
                                continue;
                            }
                        }
                    },
                    None => {
                        continue;
                    }
                };

                if !path.starts_with("final/") {
                    path = format!("final/{}", path);
                }

                if !path.starts_with("final/assets") {
                    continue;
                }
                textures.insert(path.clone(), asset);
            }
            Ok(())
        }
        _ => {
            Err(Error::ObjectTypeError)
        }
    }
}

fn object_hash(unpackdef: &UnpackDef) -> HashMap<String, String> {
    unpackdef
        .file_paths
        .par_iter()
        .fold(|| HashMap::new(), |mut map, asset_path| {
            {
                let mut asset_bundle = match AssetBundle::load_from_file(asset_path) {
                    Ok(asset_bundle) => asset_bundle,
                    Err(e) => {
                        println!("Skipping file at {} due to error: {:?}", asset_path, e);
                        return map;
                    }
                };

                for i in 0..asset_bundle.assets.len() {
                    match asset_bundle.resolve_asset(i) {
                        Err(e) => {
                            println!("Skipping file at {} due to error: {:?}", asset_path, e);
                            return map;
                        }
                        _ => {}
                    };
                    let asset = &mut asset_bundle.assets[i];
                    let objects = &asset.objects;

                    for (id, ref obj) in objects.iter() {
                        if unpackdef.object_types.contains(&obj.type_name) {
                            let engine_object =
                                obj.read_signature(asset, &mut asset_bundle.signature)
                                    .unwrap();
                            if obj.type_name == "Texture2D" {
                                let texture = match engine_object {
                                    ObjectValue::EngineObject(engine_object) => {
                                        engine_object.to_texture2d().unwrap()
                                    }
                                    _ => {
                                        panic!("Invalid engine object: not Texture2D type");
                                    }
                                };
                                //println!("{} - {}|{}|{}", texture.name.clone(), asset_path, i, id);
                                map.insert(texture.name, format!("{}|{}|{}", asset_path, i, id));
                            } else if obj.type_name == "TextAsset" {
                                let text = match engine_object {
                                    ObjectValue::EngineObject(engine_object) => {
                                        engine_object.to_textasset().unwrap()
                                    }
                                    _ => {
                                        panic!("Invalid engine object: not TextAsset type");
                                    }
                                };
                                map.insert(text.object.name, format!("{}|{}|{}", asset_path, i, id));
                            } else if obj.type_name == "FontDef" {
                                let font = match engine_object {
                                    ObjectValue::EngineObject(engine_object) => {
                                        engine_object.to_fontdef(&asset).unwrap()
                                    }
                                    _ => {
                                        panic!("Invalid engine object: not FontDef type");
                                    }
                                };
                                map.insert(format!("{}|{}|{}", asset_path, i, id), format!("{}|{}", font.font.file_name, font.font.path_id) );
                            } else if obj.type_name == "Font" {
                                let font = match engine_object {
                                    ObjectValue::EngineObject(engine_object) => {
                                        engine_object.to_font().unwrap()
                                    }
                                    _ => {
                                        panic!("Invalid engine object: not Font type");
                                    }
                                };
                                map.insert(font.object.name, format!("{}|{}|{}", asset_path, i, id));
                            } else if obj.type_name == "AssetBundle" {
                                /*let asset_bundle = match engine_object {
                                    ObjectValue::EngineObject(mut engine_object) => {
                                        let mut items = engine_object.map.remove(&"m_Container".to_string()).unwrap().into_vec().unwrap();
                                        
                                        for item in items.drain(0..) {
                                            let p = item.into_pair().unwrap();
                                            //println!("{}", p.0.to_string().unwrap());
                                        }
                                    }
                                    _ => {
                                        panic!("Invalid engine object: not AssetBundle type");
                                    }
                                };*/
                            }
                        }
                    }
                }
            }
            map
        })
        .reduce(|| HashMap::new(), |mut a, b| {
            a.extend(b);
            a
        })
}


impl Assets {
    pub fn new(assets_path: &str) -> Result<Self> {
        // generate asset catalog
        let textures = Assets::load_textures(assets_path)?;
        let card_frames = Assets::load_card_frames();
        let card_assets = Assets::load_card_assets();
        let fonts = Assets::load_fonts(assets_path)?;

        Ok(Assets {
            texture_cache: textures,
            card_frames: card_frames,
            card_assets: card_assets,
            fonts: fonts,
        })
    }

    fn catalog_get(catalog: &HashMap<String, String>, key: &str) -> Result<EngineObject> {
        let path = match catalog.get(key) {
            Some(p) => p,
            None => {
                return Err(Error::ItemNotFoundError);
            }
        };

        let elems: Vec<&str> = path.split("|").collect();
        let file_path = elems[0];
        let asset_num = elems[1].parse::<usize>().unwrap();
        let object_id = elems[2].parse::<i64>().unwrap();
        let mut asset_bundle = AssetBundle::load_from_file(file_path)?;
        asset_bundle.resolve_asset(asset_num)?;
        let asset = &mut asset_bundle.assets[asset_num];

        match asset.objects[&object_id].read_signature(asset, &mut asset_bundle.signature)? {
            ObjectValue::EngineObject(engine_object) => Ok(engine_object),
            _ => Err(Error::ObjectTypeError),
        }
    }

    fn load_textures(assets_path: &str) -> Result<HashMap<String, String>> {
        // files containing textures
        let textures = UnpackDef::new(&[assets_path, "/*texture*.unity3d"].join(""), vec!["GameObject".to_string(), "AssetBundle".to_string()]);
        let cards = UnpackDef::new(&[assets_path, "/cards*.unity3d"].join(""), vec!["GameObject".to_string(),"AssetBundle".to_string()]);

        let asset_src = vec![textures, cards];

        let res = asset_src
            .par_iter()
            .fold(
                || (HashMap::new(),HashMap::new()),
                |mut maps, unpackdef| {
                    let map_pair = extract_textures(&unpackdef);
                    let z = map_pair.unwrap();
                    maps.0.extend(z.0);
                    maps.1.extend(z.1);
                    maps
                },
            )
            .reduce(
                || (HashMap::new(),HashMap::new()),
                |mut a, b| {
                    a.0.extend(b.0);
                    a.1.extend(b.1);
                    a
                },
            );

        Ok(HashMap::new())
    }

    fn load_card_frames() -> HashMap<String, &'static [u8]> {
        let mut res = HashMap::new();
        res.insert(
            format!("{:?}_{:?}", CardType::SPELL, CardClass::Mage),
            FRAME_SPELL_MAGE,
        );
        res
    }

    fn load_card_assets() -> HashMap<String, &'static [u8]> {
        let mut res = HashMap::new();
        res.insert(
            format!("MANA_GEM"),
            MANA_GEM,
        );
        res
    }

    fn load_fonts(assets_path: &str) -> Result<HashMap<String, Font>> {
        
        let shared = UnpackDef::new(&[assets_path, "/shared*.unity3d"].join(""), vec!["Font".to_string()]);

        let fonts = object_hash(&shared); // contains font definitions

        let mut res = HashMap::new();
        for key in fonts.keys() {
            let engine_object = Assets::catalog_get(&fonts, key)?;
            let font = engine_object.to_font()?;
            res.insert(font.object.name.clone(), font);
        }

        Ok(res)
    }

    pub fn get_card_frame(&self, card_type: &CardType, card_class: &CardClass) -> Result<&[u8]> {
        Ok(match self.card_frames
            .get(&format!("{:?}_{:?}", card_type, card_class))
        {
            Some(k) => k,
            None => {
                return Err(Error::AssetNotFoundError);
            }
        })
    }

    pub fn get_card_asset(&self, asset: &str) -> Result<&[u8]> {
        Ok(match self.card_assets
            .get(asset)
        {
            Some(k) => k,
            None => {
                return Err(Error::AssetNotFoundError);
            }
        })
    }

    pub fn get_font(&self, font_name: &str) -> Result<&Font> {
        let font = self.fonts.get(font_name).ok_or(Error::AssetNotFoundError)?;

        Ok(font)
    }

    pub fn get_card_texture(&self, card_id: &str) -> Result<Vec<u8>> {
        let engine_object = Assets::catalog_get(&self.texture_cache, card_id)?;
        let texture2d = engine_object.to_texture2d()?;
        let image = texture2d.to_image()?;
        Ok(image)
    }
}
