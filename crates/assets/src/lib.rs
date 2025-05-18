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

#[derive(VariantArray, Clone, Copy, Eq, Debug, PartialEq)]
pub enum Sound {
    ShortSoft,
    SimpleTone,
}
impl From<Sound> for AssetPath<'static> {
    fn from(value: Sound) -> Self {
        AssetPath::from_static(match value {
            Sound::ShortSoft => "sounds/short_soft.ogg",
            Sound::SimpleTone => "sounds/simple_tone.ogg",
        })
    }
}

#[derive(VariantArray, Clone, Copy, Eq, Debug, PartialEq)]
pub enum Texture {
    TargettingCircle,
}
impl From<Texture> for AssetPath<'static> {
    fn from(value: Texture) -> Self {
        AssetPath::from_static(match value {
            Texture::TargettingCircle => "textures/targetting_circle.png",
        })
    }
}

#[cfg(test)]
mod test {
    use crate::Font;
    use crate::Sound;
    use crate::Texture;
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
    #[test]
    fn sound_exists() {
        for sound in Sound::VARIANTS.iter().cloned() {
            let assets_dir =
                Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../assets");
            let asset_path: AssetPath = sound.into();
            let path = assets_dir.join(asset_path.path());
            let exists = path.exists();
            assert!(
                exists,
                "Sound {:?} does not exist in the assets dir: {}",
                sound,
                path.display()
            );
        }
    }
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
