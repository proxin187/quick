use super::node::Node;

use std::cell::RefCell;
use std::rc::Rc;


#[derive(Clone)]
pub struct DocumentFragment {
    node: Rc<RefCell<Node>>,
}


