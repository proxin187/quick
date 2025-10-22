use crate::tokenizer::{Attribute, Doctype};

use super::quirks::QuirksMode;


/// The local name of an element and its namespace.
#[derive(Clone, Copy)]
pub struct QualifiedName<'a> {
    pub namespace: Option<&'a str>,
    pub namespace_prefix: Option<&'a str>,
    pub local_name: &'a str,
}

impl<'a> QualifiedName<'a> {
    /// Create a new element name with a local_name and namespace.
    pub fn new_with_ns(local_name: &'a str, namespace: &'a str) -> QualifiedName<'a> {
        QualifiedName {
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

    /// Check if the element is a mathml annotation xml element
    pub fn is_mathml_annotation_xml(&self) -> bool {
        self.is_namespace("http://www.w3.org/1998/Math/MathML")
            && self.local_name == "annotation-xml"
    }

    // TODO: implement html integration point, im not sure if we really need this
    /// Check if the element is a html integration point.
    pub fn is_html_integration_point(&self) -> bool {
        self.is_mathml_annotation_xml()
    }

    // TODO: form associated custom elements
    /// Check if the element is a form associated element.
    pub fn is_form_associated(&self) -> bool {
        self.is_namespace("http://www.w3.org/1999/xhtml")
            && ["button", "fieldset", "input", "object", "output", "select", "textarea", "img"].contains(&self.local_name)
    }

    /// Check if the element is a listed element.
    pub fn is_listed(&self) -> bool {
        self.is_namespace("http://www.w3.org/1999/xhtml")
            && ["button", "fieldset", "input", "object", "output", "select", "textarea"].contains(&self.local_name)
    }
}

/// A reference to a node in the dom.
pub trait Node: Clone + PartialEq {
    /// A custom element registry.
    type CustomElementRegistry;

    /// Given a node, return the associated node document handle.
    fn node_document(&self) -> &Self;

    /// Given a node, return its root node.
    fn root(&self) -> &Self;

    /// Given an element node, return the element name.
    fn element_name<'a>(&self) -> QualifiedName<'a>;

    /// Given an element, shadow root or document node, return its custom element registry.
    fn custom_element_registry(&self) -> Option<Self::CustomElementRegistry>;

    /// Given a handle to a node, return the parent of said node if it exists.
    fn parent(&self) -> Option<&Self>;

    /// Given a parent and child node, append said child node into the dom as the last child of said parent node.
    fn append(&mut self, child: &Self);

    /// Append a child node before another node.
    fn append_before(&mut self, before: &Self, child: &Self);

    /// Given a node, qualified name, name and a value, append an attribute with those values.
    fn append_attribute(&mut self, name: QualifiedName, value: &str);

    /// Given a node and a qualified name, return true if there is an attribute under the qualified name.
    fn has_attribute(&self, name: QualifiedName) -> bool;

    /// Given a node, set the nodes parser inserted flag.
    fn set_parser_inserted(&self);

    /// Associate a node with a form element.
    fn set_associated_form(&self, form: Self);
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
        registry: &Option<Self::CustomElementRegistry>,
        name: QualifiedName,
        is: Option<&str>
    ) -> Option<Self::CustomElementDefinition>;

    /// Called when a parse error is encountered.
    fn parse_error<Message: AsRef<str>>(&mut self, message: Message);

    /// Given a name and attributes, create an element.
    fn create_element(
        &mut self,
        document: &Self::Handle,
        name: QualifiedName,
        is: Option<&str>,
        sync: bool,
        registry: &Option<Self::CustomElementRegistry>
    ) -> Self::Handle;

    /// Given some content, create a comment.
    fn create_comment(&mut self, content: &str) -> Self::Handle;

    /// Append a doctype to the document.
    fn append_doctype(&mut self, doctype: &Doctype);

    /// Set the quirks mode.
    fn set_quirks_mode(&mut self, mode: QuirksMode);
}


