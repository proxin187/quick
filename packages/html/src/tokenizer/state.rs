// https://html.spec.whatwg.org/multipage/parsing.html#tokenization

#[derive(PartialEq, Clone, Copy)]
pub enum DoctypeKind {
    Public,
    System,
}

#[derive(PartialEq, Clone, Copy)]
pub enum IdentifierKind {
    DoubleQuoted,
    SingleQuoted,
}

#[derive(PartialEq, Clone, Copy)]
pub enum EscapeKind {
    Escaped,
    DoubleEscaped,
}

#[derive(PartialEq, Clone, Copy)]
pub enum RawKind {
    RcData,
    RawText,
    ScriptData,
}

#[derive(PartialEq, Clone, Copy)]
pub enum State {
    Data,
    RawData(RawKind),
    Plaintext,
    TagOpen,
    EndTagOpen,
    TagName,
    RawLessThanSign(RawKind),
    RawEndTagOpen(RawKind),
    RawEndTagName(RawKind),
    ScriptDataEscapeStart(EscapeKind),
    ScriptDataEscapeStartDash,
    ScriptDataEscaped(EscapeKind),
    ScriptDataEscapedDash(EscapeKind),
    ScriptDataEscapedDashDash(EscapeKind),
    ScriptDataEscapedLessThanSign(EscapeKind),
    ScriptDataEscapedEndTagOpen,
    ScriptDataEscapedEndTagName,
    ScriptDataDoubleEscapeEnd,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    BogusComment,
    MarkupDeclarationOpen,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentLessThenSign,
    CommentLessThenSignBang,
    CommentLessThenSignBangDash,
    CommentLessThenSignBangDashDash,
    CommentEndDash,
    CommentEnd,
    CommentEndBang,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
    AfterDoctypeName,
    AfterDoctypeKeyword(DoctypeKind),
    BeforeDoctypeIdentifier(DoctypeKind),
    DoctypeIdentifier(IdentifierKind, DoctypeKind),
    AfterDoctypeIdentifier(DoctypeKind),
    BetweenDoctypePublicAndSystemIdentifiers,
    BogusDoctype,
    CDataSection,
    CDataSectionBracket,
    CDataSectionEnd,
}
