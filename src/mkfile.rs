use std::{collections::HashMap, path::PathBuf};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub enum ConcreteTarget {
    Deep(PathBuf),
    Shallow(PathBuf),
}

impl ConcreteTarget {
    pub fn exists(&self) -> bool {
        match self {
            ConcreteTarget::Deep(path) => path.exists(),
            ConcreteTarget::Shallow(path) => path.exists(),
        }
    }
    pub fn pathbuf(&self) -> &PathBuf {
        match self {
            ConcreteTarget::Deep(path) => path,
            ConcreteTarget::Shallow(path) => path,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Target {
    Concrete(ConcreteTarget),
    Virtual(String),
}

pub type UpdateCommand = String;

#[derive(Debug, PartialEq)]
pub struct Rule {
    dependencies: Vec<Target>,
    commands: Vec<UpdateCommand>,
}

impl Target {
    pub fn parse(text: &str) -> Self {
        if let Some(text) = text.strip_prefix('$') {
            Target::Virtual(text.to_string())
        } else if let Some(text) = text.strip_prefix('^') {
            Target::Concrete(ConcreteTarget::Deep(PathBuf::from(text)))
        } else {
            Target::Concrete(ConcreteTarget::Shallow(PathBuf::from(text)))
        }
    }
}

#[derive(Debug)]
pub struct MkFile {
    rules: HashMap<Target, Rule>,
}

impl MkFile {
    pub fn parse(text: &str) -> Self {
        lazy_static! {
            static ref RULE_RE: Regex =
                Regex::new(r"([^\s]+)\s*:([^\n]*)((\n[ \t]+[^\n]+)*)").unwrap();
        }

        let mut rules = HashMap::new();

        for cap in RULE_RE.captures_iter(text) {
            let target = Target::parse(&cap[1]);
            let dependencies = cap[2].split_whitespace().map(Target::parse).collect();
            let commands = cap[3]
                .split('\n')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            let rule = Rule {
                dependencies,
                commands,
            };

            rules.insert(target, rule);
        }

        MkFile { rules }
    }

    pub fn dependencies(&self, target: &Target) -> &Vec<Target> {
        &self.rules[target].dependencies
    }

    pub fn commands(&self, target: &Target) -> &Vec<UpdateCommand> {
        &self.rules[target].commands
    }

    pub fn has_target(&self, target: &Target) -> bool {
        self.rules.contains_key(target)
    }
}

#[cfg(test)]
mod test {
    use insta::assert_debug_snapshot;

    use super::*;

    #[test]
    fn test_parse() {
        let test_input = include_str!("test_input.mk");
        let rules = MkFile::parse(test_input);

        assert_debug_snapshot!(rules);
    }
}
