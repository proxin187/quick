use crate::tokenizer::{Attribute, Doctype};

use super::quirks::QuirksMode;


/// The local name of an element and its namespace.
pub struct ElementName<'a> {
    pub namespace: Option<&'a str>,
    pub namespace_prefix: Option<&'a str>,
    pub local_name: &'a str,
}

impl<'a> ElementName<'a> {
    pub fn new(namespace: Option<&'a str>, namespace_prefix: Option<&'a str>, local_name: &'a str) -> ElementName<'a> {
        ElementName {
            namespace,
            namespace_prefix,
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
    fn document(&self) -> Handle;

    fn element_name(&self, handle: &Handle) -> ElementName;

    fn parse_error<Message: AsRef<str>>(&mut self, message: Message);

    fn create_element(&mut self, name: ElementName, attributes: &[Attribute]) -> Handle;

    fn create_comment(&mut self, content: &str) -> Handle;

    fn append(&mut self, parent: &Handle, child: &Handle);

    fn append_doctype(&mut self, doctype: &Doctype);

    fn set_quirks_mode(&mut self, mode: QuirksMode);
}


