use std::fmt::Display;
use std::sync::Arc;

pub trait Repository<T> {
    type Err: Display;

    fn load(&self) -> Result<T, Self::Err>;

    fn save(&self, data: &T) -> Result<(), Self::Err>;
}

impl<T, R> Repository<T> for Arc<R>
where
    R: Repository<T>,
{
    type Err = R::Err;

    fn load(&self) -> Result<T, Self::Err> {
        (**self).load()
    }

    fn save(&self, data: &T) -> Result<(), Self::Err> {
        (**self).save(data)
    }
}
