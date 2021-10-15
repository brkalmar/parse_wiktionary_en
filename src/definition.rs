// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

pub fn parse_definition<'a>(
    context: &mut ::Context<'a>,
    list_item: &::ListItem<'a>,
) -> ::Definition<'a> {
    let mut definition = vec![];
    let mut definitions = None;
    let mut examples = 0;
    let mut quotations = 0;
    macro_rules! push {
        ($expression:expr) => {{
            definition.push($expression);
            continue;
        }};
    }
    for node in &list_item.nodes {
        match node {
            ::Node::DefinitionList { items, .. } => {
                examples += items.len() as u32;
                ::add_warning(context, node, ::WarningMessage::Supplementary);
                continue;
            }
            ::Node::Italic { .. } => push!(::Flowing::Italic),
            ::Node::Link { target, text, .. } => push!(::parse_link(context, node, target, text)),
            ::Node::OrderedList { items, .. } => {
                definitions = Some(if definitions.is_some() {
                    ::add_warning(context, node, ::WarningMessage::Duplicate);
                    vec![]
                } else {
                    items
                        .iter()
                        .map(|item| parse_definition(context, item))
                        .collect()
                });
                continue;
            }
            ::Node::Text { value, .. } => push!(::Flowing::Text {
                value: ::Cow::Borrowed(value)
            }),
            ::Node::Template {
                name, parameters, ..
            } => if let Some(name) = ::parse_text(name) {
                match &name as _ {
                    "defdate" => push!(parse_definition_date(context, node, parameters)),
                    "label" | "lb" => push!(parse_labels(context, node, parameters)),
                    "n-g" | "non-gloss definition" => {
                        push!(parse_non_gloss_definition(context, node, parameters))
                    }
                    _ => {}
                }
            },
            ::Node::UnorderedList { items, .. } => {
                quotations += items.len() as u32;
                ::add_warning(context, node, ::WarningMessage::Supplementary);
                continue;
            }
            _ => {}
        }
        definition.push(::create_unknown(
            context,
            node,
            node,
            ::WarningMessage::Unrecognized,
        ))
    }
    if definition.is_empty() {
        ::add_warning(context, list_item, ::WarningMessage::Empty);
    }
    ::Definition {
        definition,
        definitions: definitions.unwrap_or_default(),
        examples,
        quotations,
    }
}

fn parse_definition_date<'a>(
    context: &mut ::Context<'a>,
    template_node: &::Node,
    parameters: &[::Parameter<'a>],
) -> ::Flowing<'a> {
    match parameters {
        [parameter @ ::Parameter { name: None, .. }] => {
            match ::parse_text_not_empty(&parameter.value) {
                None => ::create_unknown(
                    context,
                    template_node,
                    parameter,
                    ::WarningMessage::ValueUnrecognized,
                ),
                Some(value) => ::Flowing::DefinitionDate { value },
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

fn parse_labels<'a>(
    context: &mut ::Context<'a>,
    template_node: &::Node,
    parameters: &[::Parameter<'a>],
) -> ::Flowing<'a> {
    if let Some(language_parameter) = parameters.first() {
        if language_parameter.name.is_some() {
            return ::create_unknown(
                context,
                template_node,
                language_parameter,
                ::WarningMessage::Unrecognized,
            );
        }
        match ::parse_text(&language_parameter.value) {
            None => {
                return ::create_unknown(
                    context,
                    template_node,
                    language_parameter,
                    ::WarningMessage::ValueUnrecognized,
                )
            }
            Some(language) => {
                if language != context.language.unwrap().language_code() {
                    return ::create_unknown(
                        context,
                        template_node,
                        language_parameter,
                        ::WarningMessage::ValueConflicting,
                    );
                }
                if parameters.len() > 1 {
                    let mut labels = vec![];
                    for parameter in &parameters[1..] {
                        if parameter.name.is_some() {
                            return ::create_unknown(
                                context,
                                template_node,
                                language_parameter,
                                ::WarningMessage::Unrecognized,
                            );
                        }
                        match ::parse_text_not_empty(&parameter.value) {
                            None => {
                                return ::create_unknown(
                                    context,
                                    template_node,
                                    parameter,
                                    ::WarningMessage::ValueUnrecognized,
                                )
                            }
                            Some(value) => if labels.contains(&value) {
                                ::add_warning(context, parameter, ::WarningMessage::Duplicate);
                            } else {
                                labels.push(value);
                            },
                        }
                    }
                    return ::Flowing::Labels { labels };
                }
            }
        }
    }
    ::create_unknown(
        context,
        template_node,
        template_node,
        ::WarningMessage::Empty,
    )
}

fn parse_non_gloss_definition<'a>(
    context: &mut ::Context<'a>,
    template_node: &::Node,
    parameters: &[::Parameter<'a>],
) -> ::Flowing<'a> {
    match parameters {
        [parameter @ ::Parameter { name: None, .. }] => if parameter.value.is_empty() {
            ::create_unknown(context, template_node, parameter, ::WarningMessage::Empty)
        } else {
            ::Flowing::NonGlossDefinition {
                value: parameter
                    .value
                    .iter()
                    .map(|node| match node {
                        ::Node::Link { target, text, .. } => {
                            ::parse_link(context, node, target, text)
                        }
                        ::Node::Text { value, .. } => ::Flowing::Text {
                            value: ::Cow::Borrowed(value),
                        },
                        _ => ::create_unknown(context, node, node, ::WarningMessage::Unrecognized),
                    })
                    .collect(),
            }
        },
        _ => ::create_unknown(
            context,
            template_node,
            template_node,
            ::WarningMessage::ValueUnrecognized,
        ),
    }
}
