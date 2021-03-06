use unitypack::assetbundle::AssetBundle;
use unitypack::object::{ObjectInfo, ObjectPointer, ObjectValue};
use unitypack::engine::texture::{IntoTexture2D, Texture2D};
use unitypack::engine::text::IntoTextAsset;
use unitypack::engine::object::IntoGameObject;
use unitypack::engine::font::{Font, IntoFont, IntoFontDef};
use unitypack::engine::EngineObject;
use unitypack::engine::mesh::{IntoMesh, Mesh};
use unitypack::asset::Asset;
use unitypack::assetbundle::Signature;
use sfml::graphics::RenderTexture;
use error::{Error, Result};
use cards::*;
use std::collections::HashMap;
use glob::glob;
use rayon::prelude::*;
use builder::Builder;
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Fonts {
    Belwe,
    BelweOutline,
    BlizzardGlobal,
    FranklinGothic,
}

const FONT_BELWE: &'static str = "Belwe";
const FONT_BELWE_OUTLINE: &'static str = "Belwe_Outline";
const FONT_BLIZZARDGLOBAL: &'static str = "BlizzardGlobal";
const FONT_FRANKLINGOTHIC: &'static str = "FranklinGothic";

/// Stores graphic elements to construct cards
pub struct Assets {
    portraits: (HashMap<String, String>, HashMap<String, ObjectLocator>), // cards, textures
    pub textures: HashMap<String, String>,
    card_frames: HashMap<String, RenderTexture>,
    fonts: HashMap<Fonts, Font>,
    pub meshes: HashMap<String, Mesh>,
}

struct ObjectLocator {
    object_pointer: ObjectPointer,
    asset_path: String,
    asset_id: usize,
}

impl ObjectLocator {
    pub fn resolve(&self) -> Result<ObjectValue> {
        let mut asset_bundle = AssetBundle::load_from_file(&self.asset_path)?;
        asset_bundle.resolve_asset(self.asset_id)?;
        let asset = &mut asset_bundle.assets[self.asset_id];
        let obj = asset
            .objects
            .get(&self.object_pointer.path_id)
            .ok_or(Error::ObjectTypeError)?;
        Ok(obj.read_signature(asset, &mut asset_bundle.signature)?)
    }
}

struct UnpackDef {
    pub file_paths: Vec<String>,
    pub object_types: Vec<String>,
}

trait Contains<T> {
    fn contains(&self, item: &T) -> bool;
}

impl Contains<String> for Vec<String> {
    fn contains(&self, item: &String) -> bool {
        for i in self {
            if *i == *item {
                return true;
            }
        }
        false
    }
}

impl Contains<&'static str> for Vec<&'static str> {
    fn contains(&self, item: &&'static str) -> bool {
        for i in self {
            if i == item {
                return true;
            }
        }
        false
    }
}

impl UnpackDef {
    fn new(path: &str, object_types: Vec<String>) -> Result<Self> {
        let file_paths: Result<Vec<String>> = glob(path)?
            .map(|x| -> Result<String> { Ok(x?.to_str().ok_or(Error::InternalError)?.to_string()) })
            .collect();

        Ok(UnpackDef {
            file_paths: file_paths?,
            object_types: object_types,
        })
    }
}

// -> cards, textures
fn extract_textures(
    unpackdef: &UnpackDef,
) -> Result<(HashMap<String, String>, HashMap<String, ObjectLocator>)> {
    unpackdef
        .file_paths
        .par_iter()
        .fold(
            || Ok((HashMap::new(), HashMap::new())),
            |map, asset_path| {
                let map_pair = match map {
                    Ok(m) => m,
                    Err(e) => {
                        return Err(e);
                    }
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
                                        Err(_) => {
                                            continue;
                                        }
                                    };

                                if obj.type_name == "AssetBundle" {
                                    match process_asset_bundle(
                                        engine_object,
                                        &mut textures,
                                        asset_path,
                                        i,
                                    ) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            continue;
                                        }
                                    };
                                } else if obj.type_name == "GameObject" {
                                    match process_game_object(
                                        engine_object,
                                        &mut cards,
                                        objects,
                                        asset,
                                        &mut asset_bundle.signature,
                                    ) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            continue;
                                        }
                                    };
                                }
                            }
                        }
                    }
                }
                Ok((cards, textures))
            },
        )
        .reduce(
            || Ok((HashMap::new(), HashMap::new())),
            |pair, newpair| {
                let mut p = pair?;
                let np = newpair?;
                p.0.extend(np.0);
                p.1.extend(np.1);
                Ok(p)
            },
        )
}

fn process_game_object(
    engine_object: ObjectValue,
    cards: &mut HashMap<String, String>,
    objects: &HashMap<i64, ObjectInfo>,
    asset: &Asset,
    signature: &mut Signature,
) -> Result<()> {
    let d = match engine_object {
        ObjectValue::EngineObject(engine_object) => engine_object.to_gameobject()?,
        _ => {
            return Err(Error::ObjectTypeError);
        }
    };

    let ref card_id = d.object.name;

    if vec!["CardDefTemplate".to_string(), "HiddenCard".to_string()].contains(card_id) {
        // not a real card
        //cards.insert(cardid = (path: "", tile: nil)
        return Ok(());
    }

    if d.component.len() < 2 {
        // not a real card
        return Err(Error::ObjectTypeError);
    }

    let carddef = match &d.component[1] {
        &ObjectValue::Pair(ref p) => match *p.1 {
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
            }
            _ => {
                return Err(Error::ObjectTypeError);
            }
        },
        &ObjectValue::Map(ref m) => match m.get(&"component".to_string())
            .ok_or(Error::ObjectTypeError)?
        {
            &ObjectValue::ObjectPointer(ref op) => {
                if op.file_id != 0 {
                    return Err(Error::ObjectTypeError);
                }
                match objects[&op.path_id].read_signature(asset, signature)? {
                    ObjectValue::EngineObject(engine_object) => engine_object,
                    _ => {
                        return Err(Error::ObjectTypeError);
                    }
                }
            }
            _ => {
                return Err(Error::ObjectTypeError);
            }
        },
        _ => {
            return Err(Error::ObjectTypeError);
        }
    };

    let mut path = carddef
        .map
        .get(&"m_PortraitTexturePath".to_string())
        .ok_or(Error::ObjectTypeError)?
        .to_string()?;

    if path == "" {
        return Err(Error::ObjectTypeError);
    }

    path = format!("final/{}", path);
    cards.insert(card_id.clone(), path.to_lowercase());

    Ok(())
}

fn process_asset_bundle(
    engine_object: ObjectValue,
    textures: &mut HashMap<String, ObjectLocator>,
    asset_path: &String,
    asset_id: usize,
) -> Result<()> {
    match engine_object {
        ObjectValue::EngineObject(mut engine_object) => {
            let mut items = engine_object
                .map
                .remove(&"m_Container".to_string())
                .ok_or(Error::ObjectTypeError)?
                .into_vec()?;

            for item in items.drain(0..) {
                let pair = match item.into_pair() {
                    Ok(o) => o,
                    _ => {
                        continue;
                    }
                };
                let mut path = match pair.0.to_string() {
                    Ok(o) => o,
                    _ => {
                        continue;
                    }
                };
                let mut dict = match *pair.1 {
                    ObjectValue::Map(map) => map,
                    _ => {
                        continue;
                    }
                };

                let asset = match dict.remove(&"asset".to_string()) {
                    Some(a) => match a {
                        ObjectValue::ObjectPointer(op) => op,
                        _ => {
                            continue;
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

                let asset_clone = asset.clone();
                textures.insert(
                    path.clone(),
                    ObjectLocator {
                        object_pointer: asset,
                        asset_path: asset_path.clone(),
                        asset_id: asset_id,
                    },
                );

                // Also store a lookup by basename to deal with Unity 5.6
                let path2 = path.clone();
                match Path::new(&path2).file_name() {
                    Some(basename) => {
                        match basename.to_str() {
                            Some(b_name) => {
                                textures.insert(
                                    b_name.to_string(),
                                    ObjectLocator {
                                        object_pointer: asset_clone,
                                        asset_path: asset_path.clone(),
                                        asset_id: asset_id,
                                    },
                                );
                            }
                            None => {}
                        };
                    }
                    None => {}
                };
            }
            Ok(())
        }
        _ => Err(Error::ObjectTypeError),
    }
}

fn object_hash(unpackdef: &UnpackDef) -> HashMap<String, String> {
    unpackdef
        .file_paths
        .par_iter()
        .fold(
            || HashMap::new(),
            |mut map, asset_path| {
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
                                    map.insert(
                                        texture.name,
                                        format!("{}|{}|{}", asset_path, i, id),
                                    );
                                } else if obj.type_name == "TextAsset" {
                                    let text = match engine_object {
                                        ObjectValue::EngineObject(engine_object) => {
                                            engine_object.to_textasset().unwrap()
                                        }
                                        _ => {
                                            panic!("Invalid engine object: not TextAsset type");
                                        }
                                    };
                                    map.insert(
                                        text.object.name,
                                        format!("{}|{}|{}", asset_path, i, id),
                                    );
                                } else if obj.type_name == "FontDef" {
                                    let font = match engine_object {
                                        ObjectValue::EngineObject(engine_object) => {
                                            engine_object.to_fontdef(&asset).unwrap()
                                        }
                                        _ => {
                                            panic!("Invalid engine object: not FontDef type");
                                        }
                                    };
                                    map.insert(
                                        format!("{}|{}|{}", asset_path, i, id),
                                        format!("{}|{}", font.font.file_name, font.font.path_id),
                                    );
                                } else if obj.type_name == "Font" {
                                    let font = match engine_object {
                                        ObjectValue::EngineObject(engine_object) => {
                                            engine_object.to_font().unwrap()
                                        }
                                        _ => {
                                            panic!("Invalid engine object: not Font type");
                                        }
                                    };
                                    map.insert(
                                        font.object.name,
                                        format!("{}|{}|{}", asset_path, i, id),
                                    );
                                } else if obj.type_name == "Mesh" {
                                    let mesh = match engine_object {
                                        ObjectValue::EngineObject(engine_object) => {
                                            engine_object.to_mesh().unwrap()
                                        }
                                        _ => {
                                            panic!("Invalid engine object: not Mesh type");
                                        }
                                    };
                                    map.insert(
                                        mesh.object.name,
                                        format!("{}|{}|{}", asset_path, i, id),
                                    );
                                }
                            }
                        }
                    }
                }
                map
            },
        )
        .reduce(
            || HashMap::new(),
            |mut a, b| {
                a.extend(b);
                a
            },
        )
}


impl Assets {
    pub fn new(assets_path: &str) -> Result<Self> {
        // generate asset catalog
        let meshes = Assets::load_meshes(assets_path)?;
        let portraits = Assets::load_portraits(assets_path)?;
        let textures = Assets::load_textures(assets_path)?;
        let card_frames = Assets::load_card_frames(&textures, &meshes)?;
        let fonts = Assets::load_fonts(assets_path)?;


        Ok(Assets {
            portraits: portraits,
            textures: textures,
            card_frames: card_frames,
            fonts: fonts,
            meshes: meshes,
        })
    }

    pub fn catalog_get(catalog: &HashMap<String, String>, key: &str) -> Result<EngineObject> {
        let path = match catalog.get(key) {
            Some(p) => p,
            None => {
                return Err(Error::AssetNotFoundError(format!(
                    "Asset not found in cache: {}",
                    key
                )));
            }
        };

        let elems: Vec<&str> = path.split("|").collect();
        let file_path = elems[0];
        let asset_num = elems[1].parse::<usize>()?;
        let object_id = elems[2].parse::<i64>()?;
        let mut asset_bundle = AssetBundle::load_from_file(file_path)?;
        asset_bundle.resolve_asset(asset_num)?;
        let asset = &mut asset_bundle.assets[asset_num];

        match asset.objects[&object_id].read_signature(asset, &mut asset_bundle.signature)? {
            ObjectValue::EngineObject(engine_object) => Ok(engine_object),
            _ => Err(Error::ObjectTypeError),
        }
    }

    fn load_portraits(
        assets_path: &str,
    ) -> Result<(HashMap<String, String>, HashMap<String, ObjectLocator>)> {
        // files containing textures
        let textures = UnpackDef::new(
            &[assets_path, "/*texture*.unity3d"].join(""),
            vec!["GameObject".to_string(), "AssetBundle".to_string()],
        )?;
        let cards = UnpackDef::new(
            &[assets_path, "/cards*.unity3d"].join(""),
            vec!["GameObject".to_string(), "AssetBundle".to_string()],
        )?;

        let asset_src = vec![textures, cards];

        Ok(asset_src
            .par_iter()
            .fold(
                || Ok((HashMap::new(), HashMap::new())),
                |maps: Result<(HashMap<String, String>, HashMap<String, ObjectLocator>)>,
                 unpackdef| {
                    let z = extract_textures(&unpackdef)?;
                    let mut m = maps?;
                    m.0.extend(z.0);
                    m.1.extend(z.1);
                    Ok(m)
                },
            )
            .reduce(
                || Ok((HashMap::new(), HashMap::new())),
                |a, b| {
                    let mut a_resolved = a?;
                    let b_resolved = b?;
                    a_resolved.0.extend(b_resolved.0);
                    a_resolved.1.extend(b_resolved.1);
                    Ok(a_resolved)
                },
            )?)
    }

    fn load_textures(assets_path: &str) -> Result<HashMap<String, String>> {
        let gameobjects = UnpackDef::new(
            &[assets_path, "/gameobjects*.unity3d"].join(""),
            vec!["Texture2D".to_string()],
        )?;
        let shared = UnpackDef::new(
            &[assets_path, "/shared*.unity3d"].join(""),
            vec!["Texture2D".to_string()],
        )?;
        let mut textures = object_hash(&gameobjects);
        textures.extend(object_hash(&shared));
        Ok(textures)
    }

    fn load_card_frames(
        textures: &HashMap<String, String>,
        meshes: &HashMap<String, Mesh>,
    ) -> Result<HashMap<String, RenderTexture>> {
        let mut res = HashMap::new();
        let builder = Builder::new()?;
        {
            res.insert(
                format!("{:?}_{:?}", CardType::Spell, CardClass::Mage),
                builder.build_card_frame(textures, &meshes, &CardClass::Mage, &CardType::Spell)?,
            );
            res.insert(
                format!("{:?}_{:?}", CardType::Spell, CardClass::Priest),
                builder.build_card_frame(textures, &meshes, &CardClass::Priest, &CardType::Spell)?,
            );
            res.insert(
                format!("{:?}_{:?}", CardType::Spell, CardClass::Warrior),
                builder.build_card_frame(textures, &meshes, &CardClass::Warrior, &CardType::Spell)?,
            );
            res.insert(
                format!("{:?}_{:?}", CardType::Spell, CardClass::Hunter),
                builder.build_card_frame(textures, &meshes, &CardClass::Hunter, &CardType::Spell)?,
            );
            res.insert(
                format!("{:?}_{:?}", CardType::Spell, CardClass::Warlock),
                builder.build_card_frame(textures, &meshes, &CardClass::Warlock, &CardType::Spell)?,
            );
            res.insert(
                format!("{:?}_{:?}", CardType::Spell, CardClass::Rogue),
                builder.build_card_frame(textures, &meshes, &CardClass::Rogue, &CardType::Spell)?,
            );
            res.insert(
                format!("{:?}_{:?}", CardType::Spell, CardClass::Druid),
                builder.build_card_frame(textures, &meshes, &CardClass::Druid, &CardType::Spell)?,
            );
            res.insert(
                format!("{:?}_{:?}", CardType::Spell, CardClass::Paladin),
                builder.build_card_frame(textures, &meshes, &CardClass::Paladin, &CardType::Spell)?,
            );
            res.insert(
                format!("{:?}_{:?}", CardType::Spell, CardClass::Shaman),
                builder.build_card_frame(textures, &meshes, &CardClass::Shaman, &CardType::Spell)?,
            );
        }

        Ok(res)
    }

    fn load_fonts(assets_path: &str) -> Result<HashMap<Fonts, Font>> {
        let shared = UnpackDef::new(
            &[assets_path, "/shared*.unity3d"].join(""),
            vec!["Font".to_string()],
        )?;
        let fonts = object_hash(&shared);

        let mut res = HashMap::new();
        for key in fonts.keys() {
            let engine_object = Assets::catalog_get(&fonts, key)?;
            let font = engine_object.to_font()?;
            if font.object.name == FONT_BELWE {
                res.insert(Fonts::Belwe, font);
            } else if font.object.name == FONT_BELWE_OUTLINE {
                res.insert(Fonts::BelweOutline, font);
            } else if font.object.name == FONT_BLIZZARDGLOBAL {
                res.insert(Fonts::BlizzardGlobal, font);
            } else if font.object.name == FONT_FRANKLINGOTHIC {
                res.insert(Fonts::FranklinGothic, font);
            }
        }

        Ok(res)
    }

    fn load_meshes(assets_path: &str) -> Result<HashMap<String, Mesh>> {
        let actors = UnpackDef::new(
            &[assets_path, "/actors*.unity3d"].join(""),
            vec!["Mesh".to_string()],
        )?;
        let mut meshes = object_hash(&actors);

        let shared = UnpackDef::new(
            &[assets_path, "/shared*.unity3d"].join(""),
            vec!["Mesh".to_string()],
        )?;
        meshes.extend(object_hash(&shared));

        let meshes_to_keep = vec![
            "InHand_Ability_Base_mesh".to_string(),
            "InHand_Ability_NameBanner_mesh".to_string(),
            "InHand_Ability_Description_mesh".to_string(),
            "InHand_Ability_RarityFrame_mesh".to_string(),
            "InHand_Ability_Portrait_mesh".to_string(),
            "RarityGem_mesh".to_string(),
            "AbilityCardCurvedText".to_string(),
            "ManaGem".to_string(),
        ];

        let mut res = HashMap::new();
        for keep in meshes_to_keep {
            let engine_object = Assets::catalog_get(&meshes, &keep)?;
            let mesh = engine_object.to_mesh()?;
            res.insert(mesh.object.name.clone(), mesh);
        }

        Ok(res)
    }

    pub fn get_card_frame(
        &self,
        card_type: &CardType,
        card_class: &CardClass,
    ) -> Result<&RenderTexture> {
        let key = format!("{:?}_{:?}", card_type, card_class);
        Ok(match self.card_frames.get(&key) {
            Some(k) => k,
            None => {
                return Err(Error::AssetNotFoundError(format!("Cannot find {}", key)));
            }
        })
    }

    pub fn get_font(&self, font_name: &Fonts) -> Result<&Font> {
        let font = self.fonts
            .get(font_name)
            .ok_or(Error::AssetNotFoundError(format!(
                "Cannot find font named {:?}",
                font_name
            )))?;

        Ok(font)
    }

    pub fn get_card_portrait(&self, card_id: &str) -> Result<Texture2D> {
        let path = self.portraits
            .0
            .get(card_id)
            .ok_or(Error::CardNotFoundError)?;

        let oplocator;
        if !self.portraits.1.contains_key(path) && path.contains(":") {
            // final/hs5-033_d.psd:2e354fb03897c45439cdc526c73ee2a1
            let basename: Vec<&str> = Path::new(path)
                .file_name()
                .ok_or(Error::CardNotFoundError)?
                .to_str()
                .ok_or(Error::CardNotFoundError)?
                .split(":")
                .collect();
            oplocator = self.portraits
                .1
                .get(&basename.get(0).ok_or(Error::CardNotFoundError)?.to_string())
                .ok_or(Error::CardNotFoundError)?;
        } else {
            oplocator = self.portraits.1.get(path).ok_or(Error::CardNotFoundError)?;
        }

        let engine_object = match oplocator.resolve()? {
            ObjectValue::EngineObject(engine_object) => engine_object,
            _ => {
                return Err(Error::AssetNotFoundError(format!(
                    "Cannot find portrait for {}",
                    card_id
                )));
            }
        };

        Ok(engine_object.to_texture2d()?)
    }
}
