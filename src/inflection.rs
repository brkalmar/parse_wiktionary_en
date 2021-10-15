// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

pub fn parse_inflection<'a>(
    context: &mut ::Context,
    heading_node: &::Node,
    nodes: &[::Node<'a>],
    output: &mut Vec<::Template<'a>>,
    template_name: &str,
) -> usize {
    let mut inflection = None;
    let mut node_index = 0;
    while let Some(node) = nodes.get(node_index) {
        match node {
            ::Node::Heading { .. } => break,
            ::Node::Template {
                name, parameters, ..
            } => if let Some(name) = ::parse_text(name) {
                let language_code = context.language.unwrap().language_code();
                if name.starts_with(language_code)
                    && name[language_code.len()..].starts_with(template_name)
                {
                    node_index += 1;
                    if inflection.is_some() {
                        inflection = Some(None);
                        ::add_warning(context, node, ::WarningMessage::Duplicate);
                    } else {
                        inflection = Some(::template::parse_template(context, name, parameters));
                    }
                    continue;
                }
            },
            _ => {}
        }
        node_index += 1;
        ::add_warning(context, node, ::WarningMessage::Unrecognized);
    }
    match inflection {
        None => ::add_warning(context, heading_node, ::WarningMessage::SectionEmpty),
        Some(None) => {}
        Some(Some(inflection)) => output.push(inflection),
    }
    node_index
}
