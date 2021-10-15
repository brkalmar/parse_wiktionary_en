// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use parse_wiki_text::Positioned;

pub struct Context<'a> {
    pub language: Option<::Language>,
    pub warnings: Vec<::Warning>,
    pub wiki_text: &'a str,
}

pub fn add_warning(context: &mut Context, node: &impl Positioned, message: ::WarningMessage) {
    // This panics when accidentally making an infinite loop that produces warnings. This sometimes happens during development. In release builds, loops are assumed to already be tested and work properly.
    debug_assert!(context.warnings.len() < 10000);
    context.warnings.push(::Warning {
        end: node.end(),
        language: context.language,
        message,
        start: node.start(),
    });
}

#[must_use]
pub fn create_unknown<'a>(
    context: &mut Context<'a>,
    unknown_node: &::Node,
    warning_node: &impl Positioned,
    warning_message: ::WarningMessage,
) -> ::Flowing<'a> {
    add_warning(context, warning_node, warning_message);
    ::Flowing::Unknown {
        value: ::Cow::Borrowed(&context.wiki_text[unknown_node.start()..unknown_node.end()]),
    }
}

#[must_use]
pub fn parse_link<'a>(
    context: &mut Context<'a>,
    node: &::Node,
    target: &'a str,
    text: &[::Node<'a>],
) -> ::Flowing<'a> {
    match parse_text(text) {
        None => create_unknown(context, node, node, ::WarningMessage::Unrecognized),
        Some(text) => ::Flowing::Link {
            target: ::Cow::Borrowed(target),
            text,
        },
    }
}

#[must_use]
pub fn parse_parameter_name<'a>(parameter: &::Parameter<'a>) -> Option<&'a str> {
    parameter
        .name
        .as_ref()
        .and_then(|nodes| match nodes.as_slice() {
            [::Node::Text { value, .. }] => Some(*value),
            _ => None,
        })
}

#[must_use]
pub fn parse_text<'a>(nodes: &[::Node<'a>]) -> Option<::Cow<'a, str>> {
    match nodes {
        [] => Some(::Cow::Borrowed("")),
        [::Node::Text { value, .. }] => Some(::Cow::Borrowed(value)),
        _ => nodes
            .iter()
            .map(|node| match node {
                ::Node::CharacterEntity { character, .. } => Some(character.to_string()),
                ::Node::Text { value, .. } => Some(value.to_string()),
                _ => None,
            })
            .collect::<Option<String>>()
            .map(::Cow::Owned),
    }
}

#[must_use]
pub fn parse_text_not_empty<'a>(nodes: &[::Node<'a>]) -> Option<::Cow<'a, str>> {
    parse_text(nodes).filter(|text| !text.is_empty())
}
