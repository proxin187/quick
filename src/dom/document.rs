use super::node::Node;

use std::cell::RefCell;
use std::rc::Rc;


pub struct Document {
    node: Rc<RefCell<Node>>,
}


