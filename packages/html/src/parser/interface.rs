use crate::tokenizer::{Attribute, Doctype};

use super::quirks::QuirksMode;


/// The local name of an element and its namespace.
pub struct ElementName<'a> {
    pub namespace: Option<&'a str>,
    pub namespace_prefix: Option<&'a str>,
    pub local_name: &'a str,
}

impl<'a> ElementName<'a> {
    /// Create a new element name with a local_name and namespace.
    pub fn new_with_ns(local_name: &'a str, namespace: &'a str) -> ElementName<'a> {
        ElementName {
            namespace: Some(namespace),
            namespace_prefix: None,
            local_name,
        }
    }

    /// Check if the element name is in a namespace.
    pub fn is_namespace(&self, is: &str) -> bool {
        self.namespace.as_ref().map(|namespace| *namespace == is)
            .unwrap_or_default()
    }

    /// Check if the element is a mathml text integration point.
    pub fn is_mathml_text_integration_point(&self) -> bool {
        self.is_namespace("http://www.w3.org/1998/Math/MathML")
            && ["mi", "mo", "mn", "ms", "mtext"].contains(&self.local_name)
    }

    /// Checks if the element is a mathml annotation xml element
    pub fn is_mathml_annotation_xml(&self) -> bool {
        self.is_namespace("http://www.w3.org/1998/Math/MathML")
            && self.local_name == "annotation-xml"
    }

    // TODO: implement html integration point, im not sure if we really need this
    pub fn is_html_integration_point(&self) -> bool {
        self.is_mathml_annotation_xml()
    }
}

/// Recieves updates on the tree.
pub trait TreeSink<Handle> {
    /// Returns the root document handle.
    fn document(&self) -> Handle;

    /// Given a node, return the associated node document handle.
    fn node_document(&self, handle: &Handle) -> &Handle;

    /// Given a element node, return the element name.
    fn element_name<'a>(&self, handle: &Handle) -> ElementName<'a>;

    /// Given a handle to a node, return the parent of said node if it exists.
    fn parent_of(&self, handle: &Handle) -> Option<&Handle>;

    /// Called when a parse error is encountered.
    fn parse_error<Message: AsRef<str>>(&mut self, message: Message);

    /// Given a name and attributes, create an element.
    fn create_element(&mut self, name: ElementName, attributes: &[Attribute]) -> Handle;

    /// Given some content, create a comment.
    fn create_comment(&mut self, content: &str) -> Handle;

    /// Given a parent and child node, append said child node into the dom as the last child of said parent node.
    fn append(&mut self, parent: &Handle, child: &Handle);

    /// Append a doctype to the document.
    fn append_doctype(&mut self, doctype: &Doctype);

    /// Set the quirks mode.
    fn set_quirks_mode(&mut self, mode: QuirksMode);
}


