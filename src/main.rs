use std::io::{self, Write};

use pop_launcher::{PluginResponse, PluginSearchResult, Request};

mod word_list;

fn main() {
    let mut out = io::stdout().lock();

    match word_list::load_lists() {
        Ok((loaded, list)) => {
            for line in io::stdin().lines().filter_map(Result::ok) {
                if let Ok(request) = serde_json::from_str::<Request>(&line) {
                    match request {
                        Request::Search(s) if !s.contains(' ') => {
                            send(&mut out, &loaded);
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
