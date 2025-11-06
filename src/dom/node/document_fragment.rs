use super::Node;

use std::cell::RefCell;
use std::rc::{Rc, Weak};


pub struct DocumentFragment {
    owner: Weak<RefCell<Node>>,
}


