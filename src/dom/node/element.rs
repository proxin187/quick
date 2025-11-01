use super::Node;

use std::cell::RefCell;
use std::rc::Rc;


pub struct Element {
    node: Rc<RefCell<Node>>,
}


