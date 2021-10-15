// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

pub fn parse_pos<'a>(
    context: &mut ::Context<'a>,
    heading_node: &::Node,
    nodes: &[::Node<'a>],
    pos_entries: &mut Vec<::PosEntry<'a>>,
    heading_level: u8,
    pos: ::Pos,
) -> usize {
    for entry in pos_entries.iter() {
        if entry.pos == pos {
            ::add_warning(context, heading_node, ::WarningMessage::Duplicate);
            break;
        }
    }
    let mut definitions = None;
    let mut head = None;
    let mut node_index = 0;
    while let Some(node) = nodes.get(node_index) {
        match node {
            ::Node::Heading { .. } => break,
            ::Node::Template {
                name, parameters, ..
            } => if let Some(name) = ::parse_text(name) {
                if check_head_template_name(context.language.unwrap(), &name) {
                    node_index += 1;
                    if head.is_some() {
                        head = Some(None);
                        ::add_warning(context, node, ::WarningMessage::Duplicate);
                    } else {
                        head = Some(::template::parse_template(context, name, parameters));
                    }
                    continue;
                }
            },
            ::Node::OrderedList { items, .. } => {
                node_index += 1;
                if definitions.is_some() {
                    definitions = Some(vec![]);
                    ::add_warning(context, node, ::WarningMessage::Duplicate);
                } else {
                    definitions = Some(
                        items
                            .iter()
                            .map(|item| ::definition::parse_definition(context, item))
                            .collect(),
                    );
                }
                continue;
            }
            _ => {}
        }
        node_index += 1;
        ::add_warning(context, node, ::WarningMessage::Unrecognized);
    }
    if definitions.is_none() {
        ::add_warning(context, heading_node, ::WarningMessage::SectionEmpty);
    }
    let mut antonyms = false;
    let mut derived_terms = false;
    let mut hypernyms = false;
    let mut hyponyms = false;
    let mut inflection = vec![];
    let mut related_terms = false;
    let mut synonyms = false;
    let mut translations = false;
    let mut usage_notes = None;
    while let Some(node) = nodes.get(node_index) {
        macro_rules! parse_section { ( $function:path, $( $output:tt )+ ) => { {
            node_index += 1;
            node_index += $function(context, node, &nodes[node_index..], &mut $( $output )+ );
            continue;
        } } }
        if let ::Node::Heading {
            level,
            nodes: heading_child_nodes,
            ..
        } = node
        {
            if *level <= heading_level {
                if *level < heading_level {
                    break;
                }
                if let Some(heading_text) = ::parse_text(&heading_child_nodes) {
                    match &heading_text as _ {
                        "Antonyms" => {
                            parse_section!(::supplementary::parse_supplementary, antonyms)
                        }
                        "Conjugation" => {
                            parse_section!(::inflection::parse_inflection, inflection, "-conj-")
                        }
                        "Declension" => {
                            parse_section!(::inflection::parse_inflection, inflection, "-decl-")
                        }
                        "Derived terms" => {
                            parse_section!(::supplementary::parse_supplementary, derived_terms)
                        }
                        "Hypernyms" => {
                            parse_section!(::supplementary::parse_supplementary, hypernyms)
                        }
                        "Hyponyms" => {
                            parse_section!(::supplementary::parse_supplementary, hyponyms)
                        }
                        "Related terms" => {
                            parse_section!(::supplementary::parse_supplementary, related_terms)
                        }
                        "Synonyms" => {
                            parse_section!(::supplementary::parse_supplementary, synonyms)
                        }
                        "Translations" => {
                            parse_section!(::supplementary::parse_supplementary, translations)
                        }
                        "Usage notes" => {
                            parse_section!(::usage_notes::parse_usage_notes, usage_notes)
                        }
                        _ => {}
                    }
                }
            }
        }
        node_index += 1;
        ::add_warning(context, node, ::WarningMessage::Unrecognized);
    }
    pos_entries.push(::PosEntry {
        antonyms,
        definitions: definitions.unwrap_or_default(),
        derived_terms,
        head: head.unwrap_or_default(),
        hypernyms,
        hyponyms,
        inflection,
        pos,
        related_terms,
        synonyms,
        translations,
        usage_notes: usage_notes.unwrap_or_default(),
    });
    node_index
}

fn check_head_template_name(language: ::Language, template_name: &str) -> bool {
    match (language, template_name) {
        (_, "head")
        | (::Language::Cs, "cs-adj")
        | (::Language::Cs, "cs-adv")
        | (::Language::Cs, "cs-noun")
        | (::Language::Cs, "cs-proper noun")
        | (::Language::De, "de-adj")
        | (::Language::De, "de-adv")
        | (::Language::De, "de-noun")
        | (::Language::De, "de-proper noun")
        | (::Language::De, "de-verb-strong")
        | (::Language::De, "de-verb-weak")
        | (::Language::En, "en-adj")
        | (::Language::En, "en-noun")
        | (::Language::En, "en-proper noun")
        | (::Language::En, "en-verb")
        | (::Language::Es, "es-adj")
        | (::Language::Es, "es-adv")
        | (::Language::Es, "es-noun")
        | (::Language::Sv, "sv-adj")
        | (::Language::Sv, "sv-adv")
        | (::Language::Sv, "sv-noun")
        | (::Language::Sv, "sv-proper noun")
        | (::Language::Sv, "sv-verb-reg") => true,
        _ => false,
    }
}
