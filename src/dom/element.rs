use super::node::Node;

use std::cell::RefCell;
use std::rc::Rc;


#[derive(Clone)]
pub struct Element {
    node: Rc<RefCell<Node>>,
}


