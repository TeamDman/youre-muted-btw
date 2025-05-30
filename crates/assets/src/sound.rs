use bevy::asset::AssetPath;
use strum::VariantArray;

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
#[cfg(test)]
mod test {
    use crate::Sound;
    use bevy::asset::AssetPath;
    use std::path::Path;
    use strum::VariantArray;
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
}
