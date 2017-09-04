use unitypack::assetbundle::AssetBundle;
use unitypack::object::ObjectValue;
use unitypack::engine::texture::IntoTexture2D;
use unitypack::engine::text::IntoTextAsset;
use unitypack::engine::font::IntoFontDef;
use unitypack::engine::font::IntoFont;
use unitypack::engine::EngineObject;
use sfml::graphics::Font;
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
    fonts: HashMap<String, HsFont>,
}

struct UnpackDef {
    pub file_paths: Vec<String>,
    pub object_type: String,
}

pub struct HsFont {
    pub font: Font,
    pub ascent: f32,
    pub character_padding: i32,
    pub character_spacing: i32,
    pub font_size: f32,
    pub kerning: Option<f32>,
    pub line_spacing: f32,
    pub pixel_scale: f32,
}

impl UnpackDef {
    fn new(path: &str, object_type: &str) -> Self {
        UnpackDef {
            file_paths: glob(path)
                .unwrap()
                .map(|x| x.unwrap().to_str().unwrap().to_string())
                .collect::<Vec<String>>(),
            object_type: object_type.to_string(),
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
                        //println!("{}",obj.type_name);
                        if obj.type_name == unpackdef.object_type {
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
                                //println!("{} - {}|{}|{}", text.object.name.clone(), asset_path, i, id);
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
                                //println!("{}", font.font.path_id);
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
                                //println!("{} - {}|{}|{}", text.object.name.clone(), asset_path, i, id);
                                map.insert(font.object.name, format!("{}|{}|{}", asset_path, i, id));
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
        let textures = UnpackDef::new(&[assets_path, "/*texture*.unity3d"].join(""), "Texture2D");
        //let shared = UnpackDef::new(&[assets_path, "/shared*.unity3d"].join(""), "Font");

        let asset_src = vec![textures];

        let res = asset_src
            .par_iter()
            .fold(
                || HashMap::new(),
                |mut map, unpackdef| {
                    map.extend(object_hash(&unpackdef));
                    map
                },
            )
            .reduce(
                || HashMap::new(),
                |mut a, b| {
                    a.extend(b);
                    a
                },
            );

        Ok(res)
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

    fn load_fonts(assets_path: &str) -> Result<HashMap<String, HsFont>> {
        
        //let fonts = UnpackDef::new(&[assets_path, "/*font*.unity3d"].join(""), "FontDef");
        let shared = UnpackDef::new(&[assets_path, "/shared*.unity3d"].join(""), "Font");

        let fonts = object_hash(&shared); // contains font definitions

        let mut res = HashMap::new();
        for key in fonts.keys() {
            let engine_object = Assets::catalog_get(&fonts, key)?;
            let font = engine_object.to_font()?;
            
            res.insert(font.object.name.clone(), HsFont {
                ascent: font.ascent,
                character_padding: font.character_padding,
                character_spacing: font.character_spacing,
                font_size: font.font_size,
                kerning: font.kerning,
                line_spacing: font.line_spacing,
                pixel_scale: font.pixel_scale,
                font: Font::from_memory(&font.data).ok_or(Error::ObjectTypeError)?,
            });
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
}
