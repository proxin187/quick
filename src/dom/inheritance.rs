use std::cell::RefCell;
use std::rc::Rc;
use std::any::Any;


pub trait Castable: Any {
    fn upcast<T: 'static>(&self) -> Option<UpcastRef<T>>;
}

pub struct UpcastRef<A> {
    downcast: Box<dyn Any>,
    inner: Rc<RefCell<A>>,
}

impl<A> std::ops::Deref for UpcastRef<A> {
    type Target = Rc<RefCell<A>>;

    fn deref(&self) -> &Rc<RefCell<A>> {
        &self.inner
    }
}

impl<A> UpcastRef<A> {
    pub fn new(downcast: Box<dyn Any>, inner: Rc<RefCell<A>>) -> UpcastRef<A> {
        UpcastRef {
            downcast,
            inner,
        }
    }

    pub fn downcast<T: 'static>(self) -> Option<Rc<RefCell<T>>> {
        self.downcast.downcast::<Rc<RefCell<T>>>()
            .map(|boxed| Box::into_inner(boxed))
            .ok()
    }
}


