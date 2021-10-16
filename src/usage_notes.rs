// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

pub fn parse_usage_notes<'a>(
    context: &mut ::Context<'a>,
    heading_node: &::Node,
    nodes: &[::Node<'a>],
    output: &mut Option<Option<Vec<::Flowing<'a>>>>,
) -> usize {
    if output.is_some() {
        *output = Some(None);
        ::add_warning(context, heading_node, ::WarningMessage::Duplicate);
        return 0;
    }
    let mut node_index = 0;
    let mut usage_notes = vec![];
    macro_rules! push {
        ($expression:expr) => {{
            node_index += 1;
            usage_notes.push($expression);
            continue;
        }};
    }
    while let Some(node) = nodes.get(node_index) {
        match node {
            ::Node::Bold { .. } => push!(::Flowing::Bold),
            ::Node::Heading { .. } => break,
            ::Node::Italic { .. } => push!(::Flowing::Italic),
            ::Node::Link { target, text, .. } => push!(::parse_link(context, node, target, text)),
            ::Node::Tag { name, .. } if name == "ref" => {
                ::add_warning(context, node, ::WarningMessage::Supplementary);
                push!(::Flowing::Reference);
            }
            ::Node::Template {
                name, parameters, ..
            } => {
                if let Some(name) = ::parse_text(name) {
                    match &name as _ {
                        "l" | "m" => push!(parse_template_term(context, node, parameters)),
                        _ => {}
                    }
                }
            }
            ::Node::Text { value, .. } => push!(::Flowing::Text {
                value: ::Cow::Borrowed(value)
            }),
            ::Node::UnorderedList { items, .. } => {
                node_index += 1;
                let items: Vec<_> = items
                    .iter()
                    .filter_map(|item| {
                        if item.nodes.is_empty() {
                            ::add_warning(context, item, ::WarningMessage::Empty);
                            None
                        } else {
                            Some(
                                item.nodes
                                    .iter()
                                    .map(|node| {
                                        match node {
                                            ::Node::Bold { .. } => return ::Flowing::Bold,
                                            ::Node::Italic { .. } => return ::Flowing::Italic,
                                            ::Node::Link { target, text, .. } => {
                                                return ::parse_link(context, node, target, text)
                                            }
                                            ::Node::Tag { name, .. } if name == "ref" => {
                                                ::add_warning(
                                                    context,
                                                    node,
                                                    ::WarningMessage::Supplementary,
                                                );
                                                return ::Flowing::Reference;
                                            }
                                            ::Node::Template {
                                                name, parameters, ..
                                            } => {
                                                if let Some(name) = ::parse_text(name) {
                                                    match &name as _ {
                                                        "l" | "m" => {
                                                            return parse_template_term(
                                                                context, node, parameters,
                                                            )
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                            ::Node::Text { value, .. } => {
                                                return ::Flowing::Text {
                                                    value: ::Cow::Borrowed(value),
                                                }
                                            }
                                            _ => {}
                                        }
                                        ::create_unknown(
                                            context,
                                            node,
                                            node,
                                            ::WarningMessage::Unrecognized,
                                        )
                                    })
                                    .collect(),
                            )
                        }
                    })
                    .collect();
                if !items.is_empty() {
                    usage_notes.push(::Flowing::UnorderedList { items });
                }
                continue;
            }
            _ => {}
        }
        node_index += 1;
        usage_notes.push(::create_unknown(
            context,
            node,
            node,
            ::WarningMessage::Unrecognized,
        ));
    }
    if usage_notes.is_empty() {
        *output = Some(None);
        ::add_warning(context, heading_node, ::WarningMessage::SectionEmpty);
    } else {
        *output = Some(Some(usage_notes));
    }
    node_index
}

fn parse_template_term<'a>(
    context: &mut ::Context<'a>,
    template_node: &::Node,
    parameters: &[::Parameter<'a>],
) -> ::Flowing<'a> {
    match parameters {
        [language_parameter @ ::Parameter { name: None, .. }, term_parameter @ ::Parameter { name: None, .. }] => {
            match ::parse_text_not_empty(&language_parameter.value) {
                None => ::create_unknown(
                    context,
                    template_node,
                    language_parameter,
                    ::WarningMessage::ValueUnrecognized,
                ),
                Some(language) => match ::parse_text_not_empty(&term_parameter.value) {
                    None => ::create_unknown(
                        context,
                        template_node,
                        term_parameter,
                        ::WarningMessage::ValueUnrecognized,
                    ),
                    Some(term) => ::Flowing::Term { language, term },
                },
            }
        }
        _ => ::create_unknown(
            context,
            template_node,
            template_node,
            ::WarningMessage::ValueUnrecognized,
        ),
    }
}
