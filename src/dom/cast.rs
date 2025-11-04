

pub trait Cast {
    fn upcast<'a, T>(&self) -> UpcastRef<'a, T, Self>;
}

pub struct UpcastRef<'a, T, C: ?Sized> {
    downcast: T,
    upcast: &'a C,
}

impl<'a, T, C> std::ops::Deref for UpcastRef<'a, T, C> {
    type Target = &'a C;

    fn deref(&self) -> &&'a C { &self.upcast }
}

impl<'a, T, C> UpcastRef<'a, T, C> {
    pub fn downcast(self) -> T { self.downcast }
}


