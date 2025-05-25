use bevy::asset::AssetPath;
use strum::VariantArray;

#[derive(VariantArray, Clone, Copy, Eq, Debug, PartialEq)]
pub enum Font {
    FixederSys2x,
}
impl From<Font> for AssetPath<'static> {
    fn from(this: Font) -> AssetPath<'static> {
        AssetPath::from_static(match this {
            Font::FixederSys2x => "fonts/FixederSys2x.ttf",
        })
    }
}

#[cfg(test)]
mod test {
    use crate::Font;
    use bevy::asset::AssetPath;
    use std::path::Path;
    use strum::VariantArray;

    #[test]
    fn font_exists() {
        for font in Font::VARIANTS.iter().cloned() {
            let assets_dir =
                Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../assets");
            let asset_path: AssetPath = font.into();
            let path = assets_dir.join(asset_path.path());
            let exists = path.exists();
            assert!(
                exists,
                "Font {:?} does not exist in the assets dir: {}",
                font,
                path.display()
            );
        }
    }
}