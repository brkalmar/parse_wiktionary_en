// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

macro_rules! module {
    ($( $name:tt $variant:tt ),+) => {
        pub fn parse_language<'a>(
            context: &mut ::Context<'a>,
            heading_node: &::Node,
            nodes: &[::Node<'a>],
            language_entries: &mut Vec<::LanguageEntry<'a>>,
            language: ::Language,
        ) -> usize {
            for entry in language_entries.iter() {
                if entry.language == language {
                    ::add_warning(context, heading_node, ::WarningMessage::Duplicate);
                    break;
                }
            }
            let mut alternative_forms = false;
            let mut anagrams = false;
            let mut etymology = false;
            let mut etymology_entries = vec![];
            let mut further_reading = false;
            let mut node_index = 0;
            let mut pos_entries = vec![];
            let mut pronunciation = None;
            while let Some(node) = nodes.get(node_index) {
                macro_rules! parse_section {
                    ($output:tt $function:path) => {{
                        node_index += 1;
                        node_index += $function(context, node, &nodes[node_index..], &mut $output);
                        continue;
                    }};
                }
                match node {
                    ::Node::Heading {
                        level,
                        nodes: heading_child_nodes,
                        ..
                    } if *level < 4 => {
                        if *level < 3 {
                            break;
                        }
                        if let Some(heading_text) = ::parse_text(heading_child_nodes) {
                            match &heading_text as _ {
                                "Alternative forms" => parse_section!(
                                    alternative_forms::supplementary::parse_supplementary
                                ),
                                "Anagrams" => {
                                    parse_section!(anagrams::supplementary::parse_supplementary)
                                }
                                "Etymology" => {
                                    parse_section!(etymology::supplementary::parse_supplementary)
                                }
                                "Etymology 1" | "Etymology 2" | "Etymology 3" | "Etymology 4" => {
                                    parse_section!(etymology_entries parse_etymology)
                                }
                                "Further reading" => parse_section!(
                                    further_reading::supplementary::parse_supplementary
                                ),
                                "Pronunciation" => parse_section!(
                                    pronunciation::pronunciation::parse_pronunciation
                                ),
                                $( $name => {
                                    node_index += 1;
                                    node_index += ::pos::parse_pos(
                                        context,
                                        node,
                                        &nodes[node_index..],
                                        &mut pos_entries,
                                        4,
                                        ::Pos::$variant,
                                    );
                                    continue;
                                } )+
                                    _ => {}
                            }
                        }
                    }
                    ::Node::Template { name, .. } => {
                        if let Some(name) = ::parse_text(name) {
                            match &name as _ {
                                "number box" | "was fwotd" | "was wotd" | "wikipedia" => {
                                    node_index += 1;
                                    ::add_warning(context, node, ::WarningMessage::Supplementary);
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
                node_index += 1;
                ::add_warning(context, node, ::WarningMessage::Unrecognized);
            }
            if pos_entries.is_empty() && etymology_entries.is_empty() {
                ::add_warning(context, heading_node, ::WarningMessage::SectionEmpty);
            }
            let pronunciation = pronunciation.unwrap_or_default();
            language_entries.push(::LanguageEntry {
                anagrams,
                etymology_entries,
                etymology_entry: ::EtymologyEntry {
                    alternative_forms,
                    audio: pronunciation.audio,
                    etymology,
                    homophones: pronunciation.homophones,
                    hyphenation: pronunciation.hyphenation,
                    ipa: pronunciation.ipa,
                    pos_entries,
                    rhymes: pronunciation.rhymes,
                },
                further_reading,
                language,
            });
            node_index
        }

        fn parse_etymology<'a>(
            context: &mut ::Context<'a>,
            heading_node: &::Node,
            nodes: &[::Node<'a>],
            output: &mut Vec<::EtymologyEntry<'a>>,
        ) -> usize {
            let mut alternative_forms = false;
            let mut etymology = false;
            let mut node_index = 0;
            let mut pos_entries = vec![];
            let mut pronunciation = None;
            while let Some(node) = nodes.get(node_index) {
                if let ::Node::Heading { .. } = node {
                    break;
                }
                etymology = true;
                node_index += 1;
                ::add_warning(context, node, ::WarningMessage::Supplementary);
            }
            while let Some(node) = nodes.get(node_index) {
                macro_rules! parse_section {
                    ($output:tt $function:path) => {{
                        node_index += 1;
                        node_index += $function(context, node, &nodes[node_index..], &mut $output);
                        continue;
                    }};
                }
                match node {
                    ::Node::Heading {
                        level,
                        nodes: heading_child_nodes,
                        ..
                    } if *level < 5 => {
                        if *level < 4 {
                            break;
                        }
                        if let Some(heading_text) = ::parse_text(heading_child_nodes) {
                            match &heading_text as _ {
                                "Alternative forms" => parse_section!(
                                    alternative_forms::supplementary::parse_supplementary
                                ),
                                "Pronunciation" => parse_section!(
                                    pronunciation::pronunciation::parse_pronunciation
                                ),
                                $( $name => {
                                    node_index += 1;
                                    node_index += ::pos::parse_pos(
                                        context,
                                        node,
                                        &nodes[node_index..],
                                        &mut pos_entries,
                                        5,
                                        ::Pos::$variant,
                                    );
                                    continue;
                                } )+
                                    _ => {}
                            }
                        }
                    }
                    _ => {}
                }
                node_index += 1;
                ::add_warning(context, node, ::WarningMessage::Unrecognized);
            }
            if pos_entries.is_empty() {
                ::add_warning(context, heading_node, ::WarningMessage::SectionEmpty);
            }
            let pronunciation = pronunciation.unwrap_or_default();
            output.push(::EtymologyEntry {
                alternative_forms,
                audio: pronunciation.audio,
                etymology,
                homophones: pronunciation.homophones,
                hyphenation: pronunciation.hyphenation,
                ipa: pronunciation.ipa,
                pos_entries,
                rhymes: pronunciation.rhymes,
            });
            node_index
        }
    };
}

module! {
    "Adjective" Adjective,
    "Adverb" Adverb,
    "Article" Article,
    "Conjunction" Conjunction,
    "Interjection" Interjection,
    "Noun" Noun,
    "Numeral" Numeral,
    "Particle" Particle,
    "Phrase" Phrase,
    "Preposition" Preposition,
    "Pronoun" Pronoun,
    "Proper noun" ProperNoun,
    "Verb" Verb
}
