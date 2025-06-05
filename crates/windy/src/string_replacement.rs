use crate::error::WindyResult;
use crate::bail;
use bstr::ByteSlice;
use bstr::ByteVec;
use std::ffi::OsString;
use std::path::PathBuf;

pub trait Replacey: Sized {
    fn replace(&self, needle: impl Into<Self>, replacement: impl Into<Self>) -> WindyResult<Self>;
}

pub fn replace_in_os_string(
    os_string: impl Into<OsString>,
    needle: impl Into<OsString>,
    replacement: impl Into<OsString>,
) -> WindyResult<OsString> {
    let os_string = os_string.into();
    let os_string = os_string.into_encoded_bytes();
    let needle = needle.into();
    let needle = needle.into_encoded_bytes();
    let replacement = replacement.into();
    let replacement = replacement.into_encoded_bytes();
    let os_string = os_string.replace(&needle, &replacement);
    match os_string.into_os_string() {
        Ok(x) => Ok(x),
        Err(e) => {
            bail!("Failed to convert bytes to OsString: {}", e);
        }
    }
}

impl Replacey for OsString {
    fn replace(
        &self,
        needle: impl Into<OsString>,
        replacement: impl Into<OsString>,
    ) -> WindyResult<Self> {
        replace_in_os_string(self, needle, replacement)
    }
}
impl Replacey for PathBuf {
    fn replace(&self, needle: impl Into<Self>, replacement: impl Into<Self>) -> WindyResult<Self> {
        let needle = needle.into();
        let replacement = replacement.into();
        let os_string = replace_in_os_string(self, needle, replacement)?;
        Ok(PathBuf::from(os_string))
    }
}

#[cfg(test)]
mod test {
    use std::ffi::OsString;
    use std::path::PathBuf;

    use crate::string_replacement::Replacey;
    use crate::string_replacement::replace_in_os_string;

    #[test]
    fn it_works() -> eyre::Result<()> {
        let original: OsString = "Ahoy!".into();
        let needle: OsString = "oy".into();
        let replacement: OsString = "ay".into();
        let result = super::replace_in_os_string(original, needle, replacement)?;
        let expected: OsString = "Ahay!".into();
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn it_works_2() -> eyre::Result<()> {
        assert_eq!("/a/b/c", replace_in_os_string("/a/x/c", "/x/", "/b/")?);
        Ok(())
    }

    #[test]
    fn it_works_3() -> eyre::Result<()> {
        let x = PathBuf::from("/home/teamy/bruh");
        let y = x.replace("teamy", "windy")?;
        assert_eq!(y, PathBuf::from("/home/windy/bruh"));
        Ok(())
    }
}
