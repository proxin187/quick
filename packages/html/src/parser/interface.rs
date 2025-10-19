use crate::tokenizer::{Attribute, Doctype};

use super::quirks::QuirksMode;


/// The local name of an element and its namespace.
#[derive(Clone, Copy)]
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

/// A reference to a node in the dom.
pub trait Node {
    /// A custom element registry.
    type CustomElementRegistry;

    /// Given a node, return the associated node document handle.
    fn node_document(&self) -> &Self;

    /// Given an element node, return the element name.
    fn element_name<'a>(&self) -> ElementName<'a>;

    /// Given an element, shadow root or document node, return its custom element registry.
    fn custom_element_registry(&self) -> Option<Self::CustomElementRegistry>;
}

/// Recieves updates on the dom.
pub trait TreeSink {
    /// A custom element definition.
    type CustomElementDefinition;

    /// A custom element registry.
    type CustomElementRegistry;

    /// A handle to a node.
    type Handle: Node<CustomElementRegistry = Self::CustomElementRegistry>;

    /// Returns the root document handle.
    fn document(&self) -> Self::Handle;

    /// Given a registry, element name, and is, return the custom element definition if it exists.
    fn custom_element_definition(
        &self,
        registry: Option<Self::CustomElementRegistry>,
        name: ElementName,
        is: Option<&str>
    ) -> Option<Self::CustomElementDefinition>;

    /// Given a handle to a node, return the parent of said node if it exists.
    fn parent_of(&self, handle: &Self::Handle) -> Option<&Self::Handle>;

    /// Called when a parse error is encountered.
    fn parse_error<Message: AsRef<str>>(&mut self, message: Message);

    /// Given a name and attributes, create an element.
    fn create_element(&mut self, name: ElementName, attributes: &[Attribute]) -> Self::Handle;

    /// Given some content, create a comment.
    fn create_comment(&mut self, content: &str) -> Self::Handle;

    /// Given a parent and child node, append said child node into the dom as the last child of said parent node.
    fn append(&mut self, parent: &Self::Handle, child: &Self::Handle);

    /// Append a doctype to the document.
    fn append_doctype(&mut self, doctype: &Doctype);

    /// Set the quirks mode.
    fn set_quirks_mode(&mut self, mode: QuirksMode);
}


