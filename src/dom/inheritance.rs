use crate::dom::node::Node;


/// private module for sealed trait.
pub(crate) mod private {
    /// Sealed trait ensures that only valid DOM types can be downcasted, anything else will result
    /// in a compile-time error.
    pub trait Sealed {}
}

/// Downcast &T into &Self.
pub trait Downcast<T>: private::Sealed {
    fn downcast_ref(value: &T) -> &Self;

    fn downcast_mut(value: &mut T) -> &mut Self;
}

impl Node {
    pub fn downcast_ref<T: Downcast<Node>>(&self) -> &T {
        T::downcast_ref(self)
    }

    pub fn downcast_mut<T: Downcast<Node>>(&mut self) -> &mut T {
        T::downcast_mut(self)
    }

}


