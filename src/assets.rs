use unitypack::assetbundle::AssetBundle;
use unitypack::object::ObjectValue;
use unitypack::engine::texture::IntoTexture2D;
use error::{Result, Error};
use std::collections::HashMap;
use glob::glob;
use rayon::prelude::*;
use std;

/// Stores graphic elements to construct cards
pub struct Assets {
    cache: HashMap<String, String>, // object_name -> file|asset
}

impl Assets {
    pub fn new(assets_path: &str) -> Result<Self> {
        Ok(Assets { cache: Assets::catalog(assets_path)? })
    }

    fn catalog(assets_path: &str) -> Result<HashMap<String, String>> {

        let texture_files = glob(&[assets_path, "/*texture*.unity3d"].join("")).unwrap();
        let file_paths = texture_files
            .map(|x| x.unwrap().to_str().unwrap().to_string())
            .collect::<Vec<String>>();

        let res = file_paths.par_iter().fold(
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
                            if obj.type_name == "Texture2D" {
                                let engine_object =
                                    obj.read_signature(asset, &mut asset_bundle.signature)
                                        .unwrap();
                                let texture = match engine_object {
                                    ObjectValue::EngineObject(engine_object) => {
                                        engine_object.to_texture2d().unwrap()
                                    }
                                    _ => {
                                        panic!("Invalid engine object");
                                    }
                                };
                                println!("{} - {}|{}",texture.name.clone(),asset_path,id);
                                map.insert(texture.name, format!("{}|{}", asset_path, id));

                            }
                        }
                    }

                    //map.insert(asset_path.clone(), asset_path.clone());
                }
                map
            },
        ).reduce(|| HashMap::new(), |mut a, b| { a.extend(b); a });


        //for entry in glob(&[assets_path, "/*.unity3d"].join("")).expect(
        //    "Failed to read glob pattern",
        //)
        /*{
            match entry {
                Ok(path) => {
                    let asset_path = path.to_str().unwrap();
                    let mut asset_bundle = AssetBundle::load_from_file(asset_path)?;

                    for i in 0..asset_bundle.assets.len() {
                        asset_bundle.resolve_asset(i)?;
                        let asset = &mut asset_bundle.assets[i];
                        let objects = &asset.objects;

                        for (id, ref obj) in objects.iter() {
                            if obj.type_name == "Texture2D" {
                                let engine_object =
                                    obj.read_signature(asset, &mut asset_bundle.signature)?;
                                let texture = match engine_object {
                                    ObjectValue::EngineObject(engine_object) => {
                                        engine_object.to_texture2d()?
                                    }
                                    _ => {
                                        panic!("Invalid engine object");
                                    }
                                };
                                //println!("{} - {}|{}",texture.name.clone(),asset_path,id);
                                res.insert(texture.name, format!("{}|{}", asset_path, id));

                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(Error::PathError(Box::new(e)));
                }
            }
        }
        */

        Ok(res)
    }
}
