//! `pplr search <query>...`: a natural-language search by `claude` over
//! .index/tags_index.json, falling back to keywords when claude cannot answer.
//! The query goes into the prompt as text on claude's stdin, never into a
//! command line.

use crate::Result;
use crate::config::Config;
use serde_json::Value;
use std::io::{IsTerminal, Write};
use std::process::{Command, ExitCode, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const PROMPT: &str = r#"You are helping search through a professional network database. Below is a JSON index containing information about people, including their roles, companies and tags.

SEARCH QUERY: "QUERY_PLACEHOLDER"

PEOPLE DATA:
DATA_PLACEHOLDER

Your task:
1. Analyze the search query to understand what the user is looking for
2. Find the most relevant people based on:
   - Tags (single words, grouped by facet: role, function, sector, org, relationship, place)
   - Role and company information
   - Semantic similarity to the query

3. Return results ranked by relevance (most relevant first)
4. For each result, briefly explain why it matches

Respond ONLY with valid JSON in this format:
{
  "results": [
    {
      "name": "Surname, Firstname",
      "path": "relative/path",
      "role": "person's role",
      "company": "company name",
      "relevance_score": 0.95,
      "explanation": "Brief explanation of why this person matches"
    }
  ],
  "total_matches": 5
}

Include up to 10 most relevant matches. Use relevance_score from 0.0 to 1.0.
"#;

const SPINNER: &str =
  "\u{280b}\u{2819}\u{2839}\u{2838}\u{283c}\u{2834}\u{2826}\u{2827}\u{2807}\u{280f}";
const SPIN_EVERY: Duration = Duration::from_millis(100);

pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  if args.first().is_none_or(|a| a.is_empty()) {
    println!("Usage: pplr search <natural-language-query>");
    println!("Examples:");
    println!("  pplr search \"people who work in film production\"");
    println!("  pplr search \"fintech founders\"");
    println!("  pplr search \"engineers I should reconnect with\"");
    println!("  pplr search \"investors in AI\"");
    return Ok(ExitCode::FAILURE);
  }
  let query = args.join(" ");
  println!("Searching for: {query}\n");
  let dir = config.data.display().to_string();
  let index_path = config.data.join(".index/tags_index.json");
  let Ok(index) = std::fs::read_to_string(&index_path) else {
    println!("Error: Tags index not found at {}", index_path.display());
    println!("Run 'pplr reindex' to create the search index.");
    return Ok(ExitCode::FAILURE);
  };

  let reply = match ask_claude(&prompt(&query, &index)) {
    Asked::Missing => {
      println!(
        "Error: Claude CLI not found. Please install Claude CLI to use natural language search."
      );
      None
    }
    Asked::Failed(response) => {
      println!("Error calling Claude: {response}");
      None
    }
    Asked::Answered(response) => match parse_reply(&response) {
      Some(reply) => Some(reply),
      None => {
        println!("Error: Invalid JSON response from Claude");
        println!("Raw response: {response}");
        None
      }
    },
  };
  match reply {
    Some(reply) => print!("{}", render_reply(&reply, &query, &dir)),
    None => {
      println!("Falling back to basic tag search...");
      print!("{}", keyword_search(&query, &index, &dir));
    }
  }
  Ok(ExitCode::SUCCESS)
}

/// The prompt: the query as text, and the index where its placeholder line was.
pub fn prompt(query: &str, index: &str) -> String {
  PROMPT
    .replace("QUERY_PLACEHOLDER", query)
    .replace("DATA_PLACEHOLDER\n", index)
}

/// What asking claude came to: each is an outcome search handles, not a fault.
enum Asked {
  /// No claude to run.
  Missing,
  /// claude ran and failed, or could not be waited for: what it said.
  Failed(String),
  /// claude's answer, stdout and stderr together.
  Answered(String),
}

fn ask_claude(prompt: &str) -> Asked {
  let child = Command::new("claude")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn();
  let Ok(mut child) = child else {
    return Asked::Missing;
  };
  if let Some(mut stdin) = child.stdin.take() {
    // A claude that exits without reading its prompt is judged by its status
    let _ = stdin.write_all(prompt.as_bytes());
  }
  let spinning = Arc::new(AtomicBool::new(std::io::stdout().is_terminal()));
  let spinner = spinning.clone();
  let handle = std::thread::spawn(move || {
    for frame in SPINNER.chars().cycle() {
      if !spinner.load(Ordering::Relaxed) {
        break;
      }
      print!("\r\u{1f50d} Searching with Claude {frame}");
      let _ = std::io::stdout().flush();
      std::thread::sleep(SPIN_EVERY);
    }
  });
  let out = child.wait_with_output();
  spinning.store(false, Ordering::Relaxed);
  let _ = handle.join();
  println!("\r\u{1f50d} Searching with Claude... done!");
  let out = match out {
    Ok(out) => out,
    Err(e) => return Asked::Failed(e.to_string()),
  };
  let mut response = String::from_utf8_lossy(&out.stdout).into_owned();
  response.push_str(&String::from_utf8_lossy(&out.stderr));
  let response = response.trim_end_matches('\n').to_owned();
  if out.status.success() {
    Asked::Answered(response)
  } else {
    Asked::Failed(response)
  }
}

/// The JSON in claude's reply: the lines from one starting `{` to one
/// starting `}`, blank lines dropped.
pub fn parse_reply(response: &str) -> Option<Value> {
  let mut inside = false;
  let mut json = String::new();
  for line in response.lines() {
    if !inside && line.starts_with('{') {
      inside = true;
    }
    if inside {
      if !line.trim().is_empty() {
        json.push_str(line);
        json.push('\n');
      }
      if line.starts_with('}') {
        inside = false;
      }
    }
  }
  serde_json::from_str(&json).ok()
}

pub fn render_reply(reply: &Value, query: &str, dir: &str) -> String {
  let total = reply
    .get("total_matches")
    .and_then(Value::as_i64)
    .unwrap_or(0);
  if total == 0 {
    return format!(
      "No matches found for: {query}\n\nTry different keywords or check 'pplr help' for search tips.\n"
    );
  }
  let text = |v: &Value, key: &str| {
    v.get(key)
      .and_then(Value::as_str)
      .unwrap_or_default()
      .to_owned()
  };
  let mut out = format!("Found {total} relevant matches:\n\n");
  for result in reply
    .get("results")
    .and_then(Value::as_array)
    .into_iter()
    .flatten()
  {
    let name = text(result, "name");
    out.push_str(&format!(
      "\u{1f464} [{name}](<{dir}/{}/About/{name} \\(About\\).md>)",
      text(result, "path")
    ));
    for (key, prefix) in [("role", "\n   "), ("company", "\n   at ")] {
      let value = text(result, key);
      if !value.is_empty() {
        out.push_str(&format!("{prefix}{value}"));
      }
    }
    out.push_str(&format!("\n   \u{2192} {}", text(result, "explanation")));
    if let Some(score) = result
      .get("relevance_score")
      .and_then(Value::as_f64)
      .filter(|s| *s != 0.0)
    {
      out.push_str(&format!(
        "\n   \u{1f3af} Relevance: {}%",
        (score * 100.0).floor()
      ));
    }
    out.push_str("\n\n");
  }
  out
}

/// The fallback: everyone whose tags, role, company or name hold the query.
pub fn keyword_search(query: &str, index: &str, dir: &str) -> String {
  let mut out = String::from("Using fallback search (basic keyword matching)...\n\n");
  let q = query.to_ascii_lowercase();
  let people = serde_json::from_str::<Value>(index).ok();
  let text = |v: &Value, key: &str| {
    v.get(key)
      .and_then(Value::as_str)
      .unwrap_or_default()
      .to_owned()
  };
  let mut blocks = Vec::new();
  for person in people
    .as_ref()
    .and_then(|p| p.get("people"))
    .and_then(Value::as_array)
    .into_iter()
    .flatten()
  {
    let tags: Vec<String> = person
      .get("tags")
      .and_then(Value::as_array)
      .into_iter()
      .flatten()
      .filter_map(Value::as_str)
      .map(str::to_owned)
      .collect();
    let hit = tags.iter().any(|t| t.contains(&q))
      || ["role", "company", "display"]
        .iter()
        .any(|key| text(person, key).to_ascii_lowercase().contains(&q));
    if !hit {
      continue;
    }
    let mut block = format!(
      "\u{1f464} [{}](<{dir}/{}>)",
      text(person, "display"),
      text(person, "about")
    );
    for (key, prefix) in [("role", "\n   "), ("company", "\n   at ")] {
      let value = text(person, key);
      if !value.is_empty() {
        block.push_str(&format!("{prefix}{value}"));
      }
    }
    if !tags.is_empty() {
      block.push_str(&format!("\n   #{}", tags.join(" #")));
    }
    blocks.push(block);
  }
  if blocks.is_empty() {
    out.push_str("No matches found.\nTry different keywords or run 'pplr reindex' to refresh the search index.\n");
  } else {
    out.push_str(&blocks.join("\n\n"));
    out.push('\n');
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn the_query_is_text_in_the_prompt_whatever_it_holds() {
    let query = "x/y & $(rm -rf ~) \"quoted\" s/a/b/";
    let p = prompt(query, "{\"people\": []}");
    assert!(p.contains(&format!("SEARCH QUERY: \"{query}\"")));
    assert!(p.contains("PEOPLE DATA:\n{\"people\": []}\nYour task:"));
  }

  #[test]
  fn a_reply_is_read_between_its_braces() {
    let reply = parse_reply("thinking...\n{\n  \"results\": [],\n\n  \"total_matches\": 0\n}\nbye");
    assert_eq!(
      reply.and_then(|r| r.get("total_matches").cloned()),
      Some(Value::from(0))
    );
    assert_eq!(parse_reply("no json here"), None);
  }

  #[test]
  fn results_render_with_relevance_and_the_fallback_matches_keywords() {
    let reply: Value = serde_json::from_str(
      r#"{"results": [{"name": "Kemp, Jon", "path": "K/Kemp, Jon", "role": "CTO", "company": "", "relevance_score": 0.957, "explanation": "a CTO"}], "total_matches": 1}"#,
    )
    .unwrap_or_default();
    assert_eq!(
      render_reply(&reply, "cto", "/p"),
      "Found 1 relevant matches:\n\n\u{1f464} [Kemp, Jon](</p/K/Kemp, Jon/About/Kemp, Jon \\(About\\).md>)\n   CTO\n   \u{2192} a CTO\n   \u{1f3af} Relevance: 95%\n\n"
    );
    let index = r#"{"people": [{"display": "Jon Kemp", "about": "K/x.md", "role": "CTO", "company": "Acme", "tags": ["fintech"]}, {"display": "Zed", "about": "Z/z.md", "role": "", "company": "", "tags": []}]}"#;
    assert_eq!(
      keyword_search("FinTech", index, "/p"),
      "Using fallback search (basic keyword matching)...\n\n\u{1f464} [Jon Kemp](</p/K/x.md>)\n   CTO\n   at Acme\n   #fintech\n"
    );
    assert!(keyword_search("nobody", index, "/p").contains("No matches found."));
  }
}
