use std::fmt::Display;

pub trait Repository<'de, T>
where
    T: serde::Serialize + serde::Deserialize<'de>,
{
    type Err: Display;

    fn load(&self) -> Result<T, Self::Err>;

    fn save(&self, data: &T) -> Result<(), Self::Err>;
}
