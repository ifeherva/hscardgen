use unitypack::assetbundle::AssetBundle;
use unitypack::object::ObjectValue;
use unitypack::engine::texture::IntoTexture2D;
use unitypack::engine::text::IntoTextAsset;
use error::{Result, Error};
use std::collections::HashMap;
use glob::glob;
use rayon::prelude::*;

/// Stores graphic elements to construct cards
pub struct Assets {
    cache: HashMap<String, String>, // object_name -> file|asset
}

struct UnpackDef {
    pub file_paths: Vec<String>,
    pub object_type: String,
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
                                println!("{} - {}|{}", texture.name.clone(), asset_path, id);
                                map.insert(texture.name, format!("{}|{}", asset_path, id));
                            } else if obj.type_name == "TextAsset" {
                                let text = match engine_object {
                                    ObjectValue::EngineObject(engine_object) => {
                                        engine_object.to_textasset().unwrap()
                                    }
                                    _ => {
                                        panic!("Invalid engine object: not TextAsset type");
                                    }
                                };
                                println!("{} - {}|{}", text.object.name.clone(), asset_path, id);
                                map.insert(text.object.name, format!("{}|{}", asset_path, id));
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
        Ok(Assets { cache: Assets::catalog(assets_path)? })
    }

    fn catalog(assets_path: &str) -> Result<HashMap<String, String>> {

        // files containing textures
        let textures = UnpackDef {
            file_paths: glob(&[assets_path, "/*texture*.unity3d"].join(""))
                .unwrap()
                .map(|x| x.unwrap().to_str().unwrap().to_string())
                .collect::<Vec<String>>(),
            object_type: "Texture2D".to_string(),
        };

        // files containing text data, e.g. cardsdb
        let textassets = UnpackDef {
            file_paths: glob(&[assets_path, "/*xml*.unity3d"].join(""))
                .unwrap()
                .map(|x| x.unwrap().to_str().unwrap().to_string())
                .collect::<Vec<String>>(),
            object_type: "TextAsset".to_string(),
        };

        let asset_src = vec![textassets, textures];

        let res = asset_src
            .par_iter()
            .fold(|| HashMap::new(), |mut map, unpackdef| {
                map.extend(object_hash(&unpackdef));
                map
            })
            .reduce(|| HashMap::new(), |mut a, b| {
                a.extend(b);
                a
            });

        Ok(res)
    }
}
