use std::{
    cmp::Ordering,
    io::{self, Write},
};

use comparison::WordScore;
use pop_launcher::{PluginResponse, PluginSearchResult, Request};
use word_list::WordList;

mod comparison;
mod word_list;

fn main() {
    let mut out = io::stdout().lock();
    let mut words = Vec::<WordData>::with_capacity(5);

    match word_list::load_lists() {
        Ok((loaded, lists)) => {
            for line in io::stdin().lines().filter_map(Result::ok) {
                if let Ok(request) = serde_json::from_str::<Request>(&line) {
                    match request {
                        Request::Search(s) if (!s.contains(' ') || s.len() < "spell ___".len()) => {
                            send(&mut out, &loaded);
                        }
                        Request::Search(s) => {
                            let reference_word = &s["spell ".len()..].to_lowercase();

                            let mut iter =
                                lists.iter().enumerate().flat_map(|(list_index, list)| {
                                    list.iter().map(move |word| WordData {
                                        word,
                                        score: comparison::compare(word, reference_word),
                                        list_index,
                                    })
                                });

                            words.clear();
                            iter.by_ref().take(4).for_each(|word| words.push(word));
                            words.sort_unstable_by_key(|word| word.score);
                            if let Some(mut worst_score) = words.last().map(|word| word.score) {
                                iter.for_each(|word| {
                                    if word.score < worst_score {
                                        if let Err(index) = words.binary_search_by(|list_word| {
                                            list_word.score.cmp(&word.score).then(Ordering::Less)
                                        }) {
                                            if words[index.saturating_sub(1)].word != word.word {
                                                words.insert(index, word);
                                                worst_score = words.pop().unwrap().score;
                                            }
                                        }
                                    }
                                });
                            }
                            send(&mut out, &PluginResponse::Clear);
                            for word in words.drain(..) {
                                word.send(&mut out, &lists);
                            }
                        }
                        _ => {}
                    }
                    finish(&mut out);
                }
            }
        }
        Err(reason) => {
            send(&mut out, &reason);
            finish(&mut out);
        }
    }
}

fn send(mut out: &mut impl Write, response: &PluginResponse) {
    serde_json::to_writer(&mut out, response).unwrap();
    out.write_all(&[b'\n']).unwrap();
}

fn finish(out: &mut impl Write) {
    send(out, &PluginResponse::Finished);
    out.flush().unwrap();
}

fn generate_response(name: impl ToString, description: impl ToString) -> PluginResponse {
    PluginResponse::Append(PluginSearchResult {
        name: name.to_string(),
        description: description.to_string(),
        ..Default::default()
    })
}

#[derive(Debug)]
struct WordData<'a> {
    word: &'a str,
    score: WordScore,
    list_index: usize,
}

impl<'a> WordData<'a> {
    pub fn send(self, out: &mut impl Write, lists: &[WordList]) {
        let language = lists[self.list_index].name();
        let score = self.score;
        send(
            out,
            &generate_response(self.word, format!("{score}   •   {language}")),
        )
    }
}
