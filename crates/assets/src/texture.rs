use bevy::asset::AssetPath;
use strum::VariantArray;

#[derive(VariantArray, Clone, Copy, Eq, Debug, PartialEq)]
pub enum Texture {
    Icon,
}
impl From<Texture> for AssetPath<'static> {
    fn from(value: Texture) -> Self {
        AssetPath::from_static(match value {
            Texture::Icon => "textures/icon.png",
        })
    }
}
#[cfg(test)]
mod test {
    use crate::Texture;
    use bevy::asset::AssetPath;
    use std::path::Path;
    use strum::VariantArray;
    #[test]
    fn texture_exists() {
        for font in Texture::VARIANTS.iter().cloned() {
            let assets_dir =
                Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../assets");
            let asset_path: AssetPath = font.into();
            let path = assets_dir.join(asset_path.path());
            let exists = path.exists();
            assert!(
                exists,
                "Texture {:?} does not exist in the assets dir: {}",
                font,
                path.display()
            );
        }
    }
}
