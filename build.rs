// SPDX-License-Identifier: GPL-3.0-or-later

#![warn(clippy::pedantic)]
#![allow(clippy::filter_map)]

use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryInto;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const PERSON: &[char] = &['\u{1f9d1}', '\u{1f468}', '\u{1f469}'];
const FAMILY_MEMBER: &[char] = &[
    '\u{200d}',
    '\u{1f468}',
    '\u{1f469}',
    '\u{1f466}',
    '\u{1f467}',
];

fn main() -> Result<()> {
    let mut f = File::create(PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("emoji.rs"))?;
    writeln!(f, "pub(crate) static EMOJI_SETS: &[&[&str]] = &[")?;
    for set in group_emoji()? {
        if set.len() == 1 {
            for emoji in set {
                writeln!(f, "    &[\"{}\"], // {}", emoji.escape_unicode(), emoji)?;
            }
        } else {
            writeln!(f, "    &[")?;
            for emoji in set {
                writeln!(f, "        \"{}\", // {}", emoji.escape_unicode(), emoji)?;
            }
            writeln!(f, "    ],")?;
        }
    }
    writeln!(f, "];")?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/emoji-data.txt");
    println!("cargo:rerun-if-changed=data/emoji-test.txt");
    Ok(())
}

fn group_emoji() -> Result<BTreeSet<BTreeSet<String>>> {
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum Group {
        Independent(String),
        ModifierBase(char),
        Person(char),
        Role(char),
        Couple,
        Family,
        HoldingHands,
        Kiss,
        Keycap,
        Time,
    }

    let modifier_bases = emoji_modifier_bases()?;
    let mut map: BTreeMap<Group, BTreeSet<String>> = BTreeMap::new();
    let mut subgroup = "";

    for line in include_str!("data/emoji-test.txt").lines() {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with("# subgroup: ") {
            subgroup = line.trim_start_matches("# subgroup: ");
        }

        let line = line.splitn(2, '#').next().unwrap().trim();
        if line.is_empty() || !line.ends_with("; fully-qualified") {
            continue;
        }
        let emoji = line
            .trim_end_matches("; fully-qualified")
            .trim()
            .split_whitespace()
            .map(parse_char)
            .collect::<Result<String>>()?;
        let first = emoji.chars().next().unwrap();
        let last = emoji.chars().rev().next().unwrap();

        if matches!(
            first,
            '\u{1f46e}' | '\u{1f693}' | '\u{1f694}' | '\u{1f6c2}' | '\u{1f6c3}'
        ) {
            continue; // acab
        }

        let group = match subgroup {
            "skin-tone" | "hair-style" | "country-flag" | "subdivision-flag" => continue,
            "person" => Group::Person(first),
            "family" => {
                if emoji.chars().all(|c| FAMILY_MEMBER.contains(&c)) {
                    Group::Family
                } else if emoji.contains('\u{1f91d}') {
                    Group::HoldingHands
                } else if emoji.contains('\u{1f48b}') {
                    Group::Kiss
                } else if emoji.contains('\u{2764}') {
                    Group::Couple
                } else {
                    match first {
                        '\u{1f46d}' | '\u{1f46b}' | '\u{1f46c}' => Group::HoldingHands,
                        '\u{1f48f}' | '\u{1f491}' | '\u{1f46a}' => continue,
                        _ => unimplemented!(
                            "unhandled {} emoji: {}",
                            subgroup,
                            emoji.escape_unicode()
                        ),
                    }
                }
            }
            _ if PERSON.contains(&first) && emoji.contains('\u{200d}') => Group::Role(last),
            "person-role" => Group::Role(first),
            "keycap" => Group::Keycap,
            "time" if ('\u{1f550}'..='\u{1f567}').contains(&first) => Group::Time,
            _ if modifier_bases.contains(&first) => Group::ModifierBase(first),
            _ => Group::Independent(emoji.clone()),
        };

        map.entry(group).or_default().insert(emoji);
    }

    Ok(map.into_iter().map(|(_, v)| v).collect())
}

fn emoji_modifier_bases() -> Result<BTreeSet<char>> {
    let mut v = BTreeSet::new();
    for line in include_str!("data/emoji-data.txt").lines() {
        let line = line.splitn(2, '#').next().unwrap().trim();
        if line.is_empty() || !line.ends_with("; Emoji_Modifier_Base") {
            continue;
        }
        let seq = line.trim_end_matches("; Emoji_Modifier_Base").trim();
        let mut iter = seq.splitn(2, "..");
        let start = parse_char(iter.next().unwrap())?;
        if let Some(end) = iter.next() {
            v.extend(start..=parse_char(end)?);
        } else {
            v.insert(start);
        }
    }
    v.extend(vec!['\u{1f9de}', '\u{1f9df}']);
    Ok(v)
}

fn parse_char(s: &str) -> Result<char> {
    Ok(u32::from_str_radix(s, 16)?.try_into()?)
}
