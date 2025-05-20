use bevy::reflect::Reflect;
use serde::Deserialize;
use serde::Serialize;

/// Construct using From<Vec<u32 | i32>>
#[derive(Eq, PartialEq, Clone, Reflect, Hash, Default, Serialize, Deserialize)]
pub struct RuntimeId(Vec<u32>);
impl From<Vec<u32>> for RuntimeId {
    fn from(v: Vec<u32>) -> Self {
        RuntimeId(v)
    }
}
impl From<Vec<i32>> for RuntimeId {
    fn from(v: Vec<i32>) -> Self {
        RuntimeId(v.iter().map(|x| *x as u32).collect())
    }
}
impl std::fmt::Display for RuntimeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|x| format!("{:X}", x).to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}
impl std::fmt::Debug for RuntimeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
impl PartialEq<Vec<u32>> for RuntimeId {
    fn eq(&self, other: &Vec<u32>) -> bool {
        self.0 == *other
    }
}
impl PartialEq<Vec<i32>> for RuntimeId {
    fn eq(&self, other: &Vec<i32>) -> bool {
        self.0.len() == other.len()
            && self
                .0
                .iter()
                .zip(other.iter())
                .all(|(a, b)| *a == *b as u32)
    }
}
