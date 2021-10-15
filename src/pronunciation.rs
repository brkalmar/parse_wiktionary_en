// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

#[derive(Default)]
pub struct Pronunciation {
    pub audio: bool,
    pub homophones: bool,
    pub hyphenation: bool,
    pub ipa: bool,
    pub rhymes: bool,
}

pub fn parse_pronunciation(
    context: &mut ::Context,
    heading_node: &::Node,
    nodes: &[::Node],
    output: &mut Option<Pronunciation>,
) -> usize {
    if output.is_some() {
        ::add_warning(context, heading_node, ::WarningMessage::Duplicate);
    }
    let mut has_list = false;
    let mut node_index = 0;
    let mut pronunciation = output.take().unwrap_or_default();
    while let Some(node) = nodes.get(node_index) {
        match node {
            ::Node::Heading { .. } => break,
            ::Node::UnorderedList { items, .. } => {
                node_index += 1;
                if has_list {
                    ::add_warning(context, node, ::WarningMessage::Duplicate);
                }
                has_list = true;
                for item in items {
                    for node in &item.nodes {
                        if let ::Node::Template { name, .. } = node {
                            if let Some(name) = ::parse_text(name) {
                                match &name as _ {
                                    "IPA" | "cs-IPA" => pronunciation.ipa = true,
                                    "audio" => pronunciation.audio = true,
                                    "homophones" => pronunciation.homophones = true,
                                    "hyphenation" => pronunciation.hyphenation = true,
                                    "rhymes" => pronunciation.rhymes = true,
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                node_index += 1;
                ::add_warning(context, node, ::WarningMessage::Unrecognized);
            }
        }
    }
    if !has_list {
        ::add_warning(context, heading_node, ::WarningMessage::SectionEmpty);
    }
    *output = Some(pronunciation);
    node_index
}
