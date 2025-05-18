use bevy::reflect::Reflect;
use serde::Deserialize;
use serde::Serialize;

#[derive(Eq, PartialEq, Clone, Reflect, Hash, Default, Serialize, Deserialize)]
pub struct RuntimeId(Vec<i32>);
impl RuntimeId {
    pub fn new(id: Vec<i32>) -> Self {
        Self(id)
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
impl PartialEq<Vec<i32>> for RuntimeId {
    fn eq(&self, other: &Vec<i32>) -> bool {
        self.0 == *other
    }
}