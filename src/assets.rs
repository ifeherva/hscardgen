use unitypack::assetbundle::AssetBundle;
use unitypack::object::ObjectValue;
use unitypack::engine::texture::IntoTexture2D;
use unitypack::error::Result;
use std::collections::HashMap;
use glob::glob;

/// Stores graphic elements to construct cards
pub struct Assets {
    cache: HashMap<String, String>, // object_name -> file|asset
}

impl Assets {
    pub fn new(assets_path: &str) -> Result<Self> {
        Ok(Assets { cache: Assets::catalog(assets_path)? })
    }

    fn catalog(assets_path: &str) -> Result<HashMap<String, String>> {
        let mut res: HashMap<String, String> = HashMap::new();
        for entry in glob(&[assets_path, "/*.unity3d"].join("")).expect(
            "Failed to read glob pattern",
        )
        {
            match entry {
                Ok(path) => {
                    let asset_path = path.to_str().unwrap();
                    let mut asset_bundle = AssetBundle::load_from_file(asset_path)?;

                    for i in 0..asset_bundle.assets.len() {
                        asset_bundle.resolve_asset(i)?;
                        let asset = &asset_bundle.assets[i];

                        let objects = &asset.objects;

                        for (id, ref obj) in objects.iter() {
                            let type_name = obj.get_type(asset, &mut asset_bundle.signature);
                            if type_name == "Texture2D" {
                                let engine_object = obj.read(asset, &mut asset_bundle.signature)?;
                                let texture = match engine_object {
                                    ObjectValue::EngineObject(engine_object) => {
                                        engine_object.to_texture2d()?
                                    }
                                    _ => {
                                        panic!("Invalid engine object");
                                    }
                                };
                                println!("{} - {}|{}",texture.name.clone(),asset_path,id);
                                res.insert(texture.name, format!("{}|{}",asset_path,id));
                                
                            }
                        }
                    }

                    //println!("Loaded bundle at {:?} with {} assets and {} objects", path.display(), asset_bundle.assets.len(), num_objects)

                }
                Err(e) => println!("{:?}", e),
            }
        }

        Ok(res)
    }
}
