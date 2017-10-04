use std::ascii::AsciiExt;
use rust_stemmers;
use serde::ser::{Serialize, Serializer, SerializeSeq};

pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| c.is_whitespace() || c == '-')
        .map(|s| String::from(s.to_ascii_lowercase()))
        .collect()
}

pub type PipelineFn = fn(String) -> Option<String>;

#[derive(Debug)]
pub struct Pipeline {
    queue: Vec<(String, PipelineFn)>,
}

impl Serialize for Pipeline {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.queue.len()))?;
        for &(ref name, _) in &self.queue {
            seq.serialize_element(&name)?;
        }
        seq.end()
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Pipeline {
            queue: vec![
                ("trimmer".into(), trimmer),
                ("stopWordFilter".into(), stop_word_filter),
                ("stemmer".into(), stemmer),
            ],
        }
    }
}

impl Pipeline {
    pub fn empty() -> Self {
        Pipeline { queue: vec![] }
    }

    // TODO: before() after(), etc.

    pub fn register_function(&mut self, name: String, func: PipelineFn) {
        self.queue.push((name, func));
    }

    // Could return impl Iterator<Item=String>
    pub fn run(&self, tokens: Vec<String>) -> Vec<String> {
        let mut ret = vec![];
        for token in tokens {
            let mut token = Some(token);
            for &(_, func) in &self.queue {
                if let Some(t) = token {
                    token = func(t);
                } else {
                    break;
                }
            }
            if let Some(t) = token {
                ret.push(t);
            }
        }
        ret
    }
}

fn trimmer(token: String) -> Option<String> {
    Some(token.trim_matches(|c: char| !c.is_digit(36) && c != '_').into())
}

// TODO: languages
fn stemmer(token: String) -> Option<String> {
    lazy_static! {
        static ref STEMMER: rust_stemmers::Stemmer = 
            rust_stemmers::Stemmer::create(rust_stemmers::Algorithm::English);
    }

    Some(STEMMER.stem(&token).into())
}

mod phf_set {
    include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
}

// TODO: languages
fn stop_word_filter(token: String) -> Option<String> {
    match phf_set::STOP_WORDS.contains(token.as_str()) {
        true => None,
        false => Some(token),
    }
}


