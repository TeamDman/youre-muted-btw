use std::path::PathBuf;
use std::str::FromStr;
use ymb_windy::error::WindyError;

pub enum IconPath {
    Dll { path: PathBuf, resource_id: u8 },
    Ico { path: PathBuf },
}

impl FromStr for IconPath {
    type Err = WindyError;

    fn from_str(icon_path_str: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = icon_path_str.split(",-").collect();
        if parts.len() == 2 {
            Ok(Self::Dll {
                path: parts[0].into(),
                resource_id: parts[1].parse::<u8>().map_err(|_| {
                    WindyError::from(eyre::eyre!(
                        "Failed to parse resource ID from: {}",
                        parts[1]
                    ))
                })?,
            })
        } else if icon_path_str.to_lowercase().ends_with(".ico") {
            Ok(Self::Ico {
                path: PathBuf::from(icon_path_str),
            })
        } else {
            ymb_windy::bail!("Invalid icon path format: {}", icon_path_str);
        }
    }
}
