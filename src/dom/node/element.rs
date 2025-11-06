use super::Node;

use std::cell::RefCell;
use std::rc::{Rc, Weak};


pub struct Element {
    owner: Weak<RefCell<Node>>,
}


