use super::node::Node;

use std::cell::RefCell;
use std::rc::Rc;


#[derive(Clone)]
pub struct Document {
    node: Rc<RefCell<Node>>,
}


