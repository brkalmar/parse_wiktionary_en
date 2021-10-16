// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

//! Parse dictionary pages from the English language edition of Wiktionary into structured data.
//!
//! For general information about Parse Wiktionary, see the readme file.
//!
//! # Examples
//!
//! This example prints all definitions found in an article, together with the language and part of speech of the entry.
//!
//! ```
//! # extern crate parse_wiki_text;
//! # extern crate parse_wiktionary_en;
//! #
//! let wiki_text = concat!(
//!     "==English==\n",
//!     "===Noun===\n",
//!     "#The assignment of a [[commercial]] [[value]] to something previously valueless."
//! );
//! let configuration = parse_wiktionary_en::create_configuration();
//! let parsed_wiki_text = configuration.parse(wiki_text);
//! let parsed_article = parse_wiktionary_en::parse(wiki_text, &parsed_wiki_text.nodes);
//! # let mut found = false;
//! for language_entry in parsed_article.language_entries {
//!     for pos_entry in language_entry.etymology_entry.pos_entries {
//!         for definition in pos_entry.definitions {
//!             println!(
//!                 "The word 'commodification' of language {language:?} and part of speech {pos:?} has the definition: {definition}",
//!                 language = language_entry.language,
//!                 pos = pos_entry.pos,
//!                 definition = &definition.definition.iter().map(|node| match node {
//!                     parse_wiktionary_en::Flowing::Link { target, text } => text,
//!                     parse_wiktionary_en::Flowing::Text { value } => value,
//!                     _ => ""
//!                 }).collect::<String>()
//!             );
//! #           found = true;
//!         }
//!     }
//! }
//! # assert!(found);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate parse_wiki_text;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod configuration;
mod definition;
mod inflection;
mod language;
mod pos;
mod pronunciation;
mod supplementary;
mod template;
mod usage_notes;
mod util;

pub use configuration::create_configuration;
use parse_wiki_text::{ListItem, Node, Parameter};
use std::{borrow::Cow, collections::HashMap};
use util::*;

/// A single definition from a list of definitions of an entry.
#[derive(Debug, Deserialize, Serialize)]
pub struct Definition<'a> {
    /// A series of elements to display as the definition.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub definition: Vec<Flowing<'a>>,

    /// Nested definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub definitions: Vec<Definition<'a>>,

    /// Number of examples the definition has.
    pub examples: u32,

    /// Number of quotations the definition has.
    pub quotations: u32,
}

/// Details related to a specific etymology, either one that has a numbered etymology heading or the same format of information directly in the language entry.
#[derive(Debug, Deserialize, Serialize)]
pub struct EtymologyEntry<'a> {
    /// Whether the entry has audio samples.
    pub audio: bool,

    /// Whether the entry has alternative forms.
    pub alternative_forms: bool,

    /// Whether the entry has a description of its etymology.
    pub etymology: bool,

    /// Whether the entry has homophones.
    pub homophones: bool,

    /// Whether the entry has hyphenations.
    pub hyphenation: bool,

    /// Whether the entry has a pronunciation written in IPA.
    pub ipa: bool,

    /// Entries for parts of speech for this etymology.
    ///
    /// Parsed from the sections with the part of speech as their heading.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pos_entries: Vec<PosEntry<'a>>,

    /// Whether the entry has rhymes.
    pub rhymes: bool,
}

/// An element in a sequence that allows different kinds of elements.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Flowing<'a> {
    /// Toggle bold text.
    ///
    /// Parsed from the wiki text `'''`.
    Bold,

    /// Definition date, from the template [`defdate`](https://en.wiktionary.org/wiki/Template:defdate).
    DefinitionDate {
        /// The text to display as the definition date.
        value: Cow<'a, str>,
    },

    /// Toggle italic.
    ///
    /// Parsed from the wiki text `''`.
    Italic,

    /// List of labels, from the template [`label`](https://en.wiktionary.org/wiki/Template:label).
    Labels {
        /// The labels.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        labels: Vec<Cow<'a, str>>,
    },

    /// Link.
    ///
    /// Parsed from wiki text starting with `[[`.
    Link {
        /// The target the link refers to.
        target: Cow<'a, str>,

        /// The text to display for the link.
        text: Cow<'a, str>,
    },

    /// Non-gloss definition, from the template [`non-gloss definition`](https://en.wiktionary.org/wiki/Template:non-gloss_definition).
    NonGlossDefinition {
        /// The text to display.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        value: Vec<Flowing<'a>>,
    },

    /// Indication of a reference.
    ///
    /// Parsed from the extension tag `ref`. The content if the reference is not parsed. This element is added to the output just to indicate the existence of a reference.
    Reference,

    /// Link to another dictionary entry, from the template [`mention`](https://en.wiktionary.org/wiki/Template:mention).
    Term {
        /// The language of the entry the link refers to.
        language: Cow<'a, str>,

        /// The term the link refers to.
        term: Cow<'a, str>,
    },

    /// Chunk of plain text.
    Text {
        /// The text to display.
        value: Cow<'a, str>,
    },

    /// Element that could not be recognized.
    Unknown {
        /// The wiki text of the element.
        value: Cow<'a, str>,
    },

    /// Unordered list.
    UnorderedList {
        /// The list items of the list.
        items: Vec<Vec<Flowing<'a>>>,
    },
}

/// Identifier for a language.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    /// Czech
    Cs,

    /// German
    De,

    /// English
    En,

    /// Esperanto
    Eo,

    /// Spanish
    Es,

    /// French
    Fr,

    /// Italian
    It,

    /// Dutch
    Nl,

    /// Portuguese
    Pt,

    /// Russian
    Ru,

    /// Swedish
    Sv,
}

/// Dictionary entry for a single language.
#[derive(Debug, Deserialize, Serialize)]
pub struct LanguageEntry<'a> {
    /// Whether the subsection `Anagrams` is present in the section.
    pub anagrams: bool,

    /// Entries for each numbered etymology for this language.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub etymology_entries: Vec<EtymologyEntry<'a>>,

    /// Entry for the etymology that is directly in the language entry.
    pub etymology_entry: EtymologyEntry<'a>,

    /// Whether the subsection `Further reading` is present in the section.
    pub further_reading: bool,

    /// The language of the entry.
    pub language: Language,
}

/// Output of parsing a page.
#[derive(Debug, Deserialize, Serialize)]
pub struct Output<'a> {
    /// The dictionary entries by language.
    ///
    /// Parsed from the sections with the name of the language as title.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub language_entries: Vec<LanguageEntry<'a>>,

    /// Warnings from the parser telling that something is not well-formed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<Warning>,
}

/// Part of speech.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Pos {
    /// Adjective
    Adjective,

    /// Adverb
    Adverb,

    /// Article
    Article,

    /// Conjunction
    Conjunction,

    /// Interjection
    Interjection,

    /// Noun
    Noun,

    /// Numeral
    Numeral,

    /// Particle
    Particle,

    /// Phrase
    Phrase,

    /// Preposition
    Preposition,

    /// Pronoun
    Pronoun,

    /// Proper noun
    ProperNoun,

    /// Verb
    Verb,
}

/// The entry for a part of speech within the entry for a language.
///
/// Parsed from the section with the part of speech as its heading.
#[derive(Debug, Deserialize, Serialize)]
pub struct PosEntry<'a> {
    /// Whether the subsection `Antonyms` is present in the section.
    pub antonyms: bool,

    /// Definitions of the entry.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub definitions: Vec<Definition<'a>>,

    /// Whether the subsection `Derived terms` is present in the section.
    pub derived_terms: bool,

    /// Details about the template for displaying the word head for the entry, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<Template<'a>>,

    /// Whether the subsection `Hypernyms` is present in the section.
    pub hypernyms: bool,

    /// Whether the subsection `Hyponyms` is present in the section.
    pub hyponyms: bool,

    /// Details about each template for displaying an inflection for the entry.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inflection: Vec<Template<'a>>,

    /// Part of speech of the entry.
    pub pos: Pos,

    /// Whether the subsection `Related terms` is present in the section.
    pub related_terms: bool,

    /// Whether the subsection `Synonyms` is present in the section.
    pub synonyms: bool,

    /// Whether the subsection `Translations` is present in the section.
    pub translations: bool,

    /// Content of the subsection `User notes` within the section, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_notes: Option<Vec<Flowing<'a>>>,
}

/// Details about a template.
#[derive(Debug, Deserialize, Serialize)]
pub struct Template<'a> {
    /// The name of the template.
    pub name: Cow<'a, str>,

    /// The named parameters to the template by name.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub named_parameters: HashMap<Cow<'a, str>, Cow<'a, str>>,

    /// The unnamed parameters to the template in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unnamed_parameters: Vec<Cow<'a, str>>,
}

/// Warning from the parser telling that something is not well-formed.
///
/// When a warning occurs, it's not guaranteed that the text near the warning is parsed correctly. Usually the data that could not be unambiguously parsed due to the warning is excluded from the output, to make sure the output doesn't contain incorrectly parsed data.
#[derive(Debug, Deserialize, Serialize)]
pub struct Warning {
    /// The byte position in the wiki text where the warning ends.
    pub end: usize,

    /// The language of the language section in which the warning occurred, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Language>,

    /// An identifier for the kind of warning.
    pub message: WarningMessage,

    /// The byte position in the wiki text where the warning starts.
    pub start: usize,
}

/// Identifier for a kind of warning from the parser.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningMessage {
    /// The element is a duplicate of something that comes before it.
    ///
    /// This may be a heading that contains the same text as a previous heading in the same section, or a parameter that has the same name as a previous parameter to the same template.
    Duplicate,

    /// The element is missing some required content.
    Empty,

    /// The section following the heading is missing some required content.
    SectionEmpty,

    /// The element is recognized but not represented in the output.
    ///
    /// The element conveys meaningful information, but this information has not been parsed and is not represented in the output. In contrast to other warnings, this warning does not indicate there is anything wrong with the wiki text. It just indicates that the wiki text contains additional information that is not represented in the output. The element is recognized as valid in the position it occurs, but its content is not parsed, and nothing can be said about whether the content is valid.
    Supplementary,

    /// The element is not recognized.
    ///
    /// This may be because of the type of the element itself or because of anything inside it.
    Unrecognized,

    /// The value of the element conflicts with information occurring before it.
    ///
    /// This can mean for example that a specified language within a section doesn't match the language specified for the section as a whole.
    ValueConflicting,

    /// The element is recognized, but its value is not.
    ///
    /// On lists it means that a list with this kind is valid in this position, but something about the list items contained in the list is not recognized.
    ///
    /// On templates it means that a template with this name is valid in this position, but something about the parameters of the template is not recognized.
    ///
    /// On template parameters it means that a parameter with this name (or lack of name) is valid in this position, but something about the value of the parameter is not recognized.
    ValueUnrecognized,
}

/// Parses an article from the English language version of Wiktionary into structured data.
///
/// `wiki_text` is the wiki text of the article. `nodes` is the sequence of nodes obtained by parsing the wiki text with the crate [Parse Wiki Text](https://github.com/portstrom/parse_wiki_text).
#[must_use]
pub fn parse<'a>(wiki_text: &'a str, nodes: &[Node<'a>]) -> Output<'a> {
    let mut context = Context {
        language: None,
        warnings: vec![],
        wiki_text,
    };
    let mut language_entries = vec![];
    let mut node_index = 0;
    while let Some(node) = nodes.get(node_index) {
        match node {
            Node::Heading {
                level,
                nodes: heading_child_nodes,
                ..
            } if *level < 3 => {
                if *level < 2 {
                    add_warning(&mut context, node, WarningMessage::Unrecognized);
                    break;
                }
                if let Some(heading_text) = parse_text(heading_child_nodes) {
                    if let Some(language) = Language::from_name(&heading_text) {
                        node_index += 1;
                        context.language = Some(language);
                        node_index += language::parse_language(
                            &mut context,
                            node,
                            &nodes[node_index..],
                            &mut language_entries,
                            language,
                        );
                        context.language = None;
                        continue;
                    }
                }
            }
            Node::Template { name, .. } => {
                if let Some(name) = parse_text(name) {
                    if &name == "also" {
                        node_index += 1;
                        add_warning(&mut context, node, WarningMessage::Supplementary);
                        continue;
                    }
                }
            }
            _ => {}
        }
        node_index += 1;
        add_warning(&mut context, node, WarningMessage::Unrecognized);
    }
    Output {
        language_entries,
        warnings: context.warnings,
    }
}

impl Language {
    /// Returns the language corresponding to the given language name if any.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "Czech" => Language::Cs,
            "Dutch" => Language::Nl,
            "English" => Language::En,
            "Esperanto" => Language::Eo,
            "French" => Language::Fr,
            "German" => Language::De,
            "Italian" => Language::It,
            "Portuguese" => Language::Pt,
            "Russian" => Language::Ru,
            "Spanish" => Language::Es,
            "Swedish" => Language::Sv,
            _ => return None,
        })
    }

    /// Returns the language code for the language.
    pub fn language_code(self) -> &'static str {
        match self {
            Language::Cs => "cs",
            Language::Nl => "nl",
            Language::En => "en",
            Language::Eo => "eo",
            Language::Fr => "fr",
            Language::De => "de",
            Language::It => "it",
            Language::Pt => "pt",
            Language::Ru => "ru",
            Language::Es => "es",
            Language::Sv => "sv",
        }
    }
}
