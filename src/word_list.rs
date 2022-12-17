use std::{collections::HashMap, fmt::Write, fs::File, io::Read, path::PathBuf};

use pop_launcher::PluginResponse;
use serde::Deserialize;

use crate::generate_response;

const PATH: &str = "./.local/share/pop-launcher/plugins/spell_check";

#[derive(Debug, Deserialize)]
struct Config {
    word_lists: HashMap<String, ListData>,
}

#[derive(Debug, Deserialize)]
struct ListData {
    name: String,
    path: String,
}

/// Loads lists, then returns (a description of loaded lists, the list)
#[allow(clippy::result_large_err)]
pub fn load_lists() -> Result<(PluginResponse, Vec<WordList>), PluginResponse> {
    const CFG_FILE: &str = "config.toml";
    let mut cfg_path = PathBuf::with_capacity(PATH.len() + CFG_FILE.len() + 4);
    cfg_path.push(PATH);
    cfg_path.push(CFG_FILE);
    match File::open(&cfg_path) {
        Ok(mut config) => {
            let mut buf = Vec::new();
            match config.read_to_end(&mut buf) {
                Ok(len) => match toml::from_slice::<Config>(&buf[..len]) {
                    Ok(config) => {
                        let mut lists = Vec::with_capacity(config.word_lists.len());
                        for ListData { name, path } in config.word_lists.into_values() {
                            let words = load_list(name, &path)?;
                            lists.push(words);
                        }
                        let mut dictionaries = String::new();
                        let mut first = true;
                        for list in &lists {
                            if !first {
                                dictionaries.push('\n');
                            }
                            first = false;
                            let name = &list.name;
                            let len = list.len;
                            write!(&mut dictionaries, "- {name} ({len} words)").unwrap();
                        }
                        Ok((generate_response("Dictionaries:", dictionaries), lists))
                    }
                    Err(e) => Err(generate_response(
                        "Unable to parse config.toml",
                        format!(
                            "Actual error:
  {e}
Example:
  [word_lists.en_us]
  name = \"U.S. English\"
  path = \"wordlists/en_us\"

  [word_lists.es_mex]
  name = \"Mexican Spanish\"
  path = \"wordlists/es_mex\""
                        ),
                    )),
                },
                Err(e) => Err(generate_response(
                    "Unable to open config.toml",
                    format!("Actual error:\n  {e}"),
                )),
            }
        }
        Err(e) => Err(generate_response(
            "Unable to open config.toml",
            format!("Make sure it is placed in the folder\n  {PATH}\nActual error:\n  {e}"),
        )),
    }
}

#[allow(clippy::result_large_err)]
fn load_list(name: String, list: &str) -> Result<WordList, PluginResponse> {
    let mut path = PathBuf::with_capacity(PATH.len() + list.len() + 4);
    path.push(PATH);
    path.push(list);
    match File::open(path) {
        Ok(mut file) => {
            let mut buf = [0; 1024];
            let mut dashes_found = 0;
            let mut list = 'outer: loop {
                match file.read(&mut buf) {
                    Ok(0) => return Err(generate_response(
                        format!("Word list \"{name}\" not formatted correctly"),
                        "Words should be seperated by newlines and the word list should start after \"---\"
Example:
  Word list I made
  ---
  word1
  word2
  word3")),
                    Ok(len) => {
                        for (index, byte) in buf[..len].iter().enumerate() {
                            match (dashes_found, byte) {
                                (0 | 1 | 2, b'-') => dashes_found += 1,
                                (3, b'\n') => break 'outer String::from_utf8(
                                    buf[(index + 1)..len].to_vec()
                                ).map_err(|e| generate_response(
                                    format!("Word list \"{name}\" is not valid utf-8"),
                                    format!("Actual error:\n  {e}"))
                                )?,
                                _ => dashes_found = 0,
                            }
                        }
                    }
                    Err(e) => {
                        return Err(generate_response(
                            format!("Unable to read word list \"{name}\""),
                            format!("Actual error:\n  {e}"),
                        ))
                    }
                }
            };
            file.read_to_string(&mut list).map_err(|e| {
                generate_response(
                    format!("Unable to read word list \"{name}\""),
                    format!("Actual error:\n  {e}"),
                )
            })?;
            let words = list.to_lowercase();
            let len = words.lines().count();
            Ok(WordList { name, words, len })
        }
        Err(e) => Err(generate_response(
            format!("Unable to open word list \"{name}\""),
            format!("Actual error:\n  {e}"),
        )),
    }
}

#[derive(Debug)]
pub struct WordList {
    name: String,
    words: String,
    len: usize,
}

impl WordList {
    pub fn iter(&self) -> impl Iterator<Item = &str> + '_ {
        self.words.lines()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
