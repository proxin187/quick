use super::Node;

use std::cell::RefCell;
use std::rc::Rc;


pub struct DocumentFragment {
    owner: Rc<RefCell<Node>>,
}


