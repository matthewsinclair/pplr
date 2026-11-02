//! The tree's indexes (ST0001 design 3.1); WP04 ports `count`, WP05 the rest.

use crate::Result;
use crate::config::Config;
use crate::tree::{is_letter, names};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

/// `pplr count`: the people in `.index/index.md` when there is one (each line
/// holding a `[...]`), else each "Surname, First" folder under a letter.
pub fn run_count(_args: &[String], config: &Config) -> Result<ExitCode> {
  println!("{}", count(&config.data));
  Ok(ExitCode::SUCCESS)
}

pub fn count(root: &Path) -> usize {
  if let Ok(index) = fs::read_to_string(root.join(".index/index.md")) {
    // grep -o '\[.*\]\(.*\)': in a basic regular expression \( \) only groups,
    // so a line counts when a [ has a ] after it
    return index
      .lines()
      .filter(|line| {
        line
          .find('[')
          .is_some_and(|open| line[open..].contains(']'))
      })
      .count();
  }
  names(root)
    .into_iter()
    .filter(|letter| is_letter(letter) && root.join(letter).is_dir())
    .map(|letter| {
      names(&root.join(&letter))
        .into_iter()
        .filter(|name| name.contains(", ") && root.join(&letter).join(name).is_dir())
        .count()
    })
    .sum()
}

/// The index's view of the tree, as `pplr_json` and `pplr_index` walked it:
/// every folder at the top but `_*`, `.*` and `bin` is a letter, and every
/// folder in a letter a person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Letter {
  pub name: String,
  pub people: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
  /// The folder name, "Surname, First".
  pub name: String,
  /// "First Surname": the stem of the files in About/.
  pub stem: String,
  pub about: String,
  pub linkedin: String,
  pub picture: String,
  pub profile: String,
  /// Each meeting folder (not `_*`) and its notes file.
  pub meetings: Vec<(String, String)>,
}

/// The extensions a picture is probed for, in order.
const PICTURE_EXTENSIONS: [&str; 3] = ["jpg", "png", "jpeg"];

pub fn letters(root: &Path) -> Vec<Letter> {
  names(root)
    .into_iter()
    .filter(|n| !n.starts_with(['_', '.']) && n != "bin" && root.join(n).is_dir())
    .map(|letter| Letter {
      people: names(&root.join(&letter))
        .into_iter()
        .filter(|n| !n.starts_with('.') && root.join(&letter).join(n).is_dir())
        .map(|name| entry(root, &letter, &name))
        .collect(),
      name: letter,
    })
    .collect()
}

fn entry(root: &Path, letter: &str, name: &str) -> Entry {
  // ${person_name%%,*} and ${person_name#*, }
  let surname = name.split(',').next().unwrap_or(name);
  let first = name.split_once(", ").map_or(name, |(_, f)| f);
  let stem = format!("{first} {surname}");
  let dir = format!("./{letter}/{name}");
  let about_dir = root.join(letter).join(name).join("About");
  let picture = PICTURE_EXTENSIONS
    .iter()
    .map(|ext| format!("{stem} (Picture).{ext}"))
    .find(|file| about_dir.join(file).is_file())
    .unwrap_or_else(|| format!("{stem} (Picture).jpg"));
  let meetings_dir = root.join(letter).join(name).join("Meetings");
  let meetings = names(&meetings_dir)
    .into_iter()
    .filter(|m| !m.starts_with(['_', '.']) && meetings_dir.join(m).is_dir())
    .map(|id| {
      let notes = [
        format!("{id}.md"),
        "Summary.md".to_owned(),
        "Notes.md".to_owned(),
      ]
      .into_iter()
      .find(|f| meetings_dir.join(&id).join(f).is_file())
      .unwrap_or_else(|| format!("{id}.md"));
      let details = format!("{dir}/Meetings/{id}/{notes}");
      (id, details)
    })
    .collect();
  Entry {
    name: name.to_owned(),
    about: format!("{dir}/About/{stem} (About).md"),
    linkedin: format!("{dir}/About/{stem} (LinkedIn).webloc"),
    picture: format!("{dir}/About/{picture}"),
    profile: format!("{dir}/About/{stem} (Profile).pdf"),
    stem,
    meetings,
  }
}

/// `pplr json`: the index as `pplr_json` echoed it, line for line.
pub fn run_json(_args: &[String], config: &Config) -> Result<ExitCode> {
  print!("{}", echoed_json(&letters(&config.data)));
  Ok(ExitCode::SUCCESS)
}

/// `pplr_json`'s own layout, strings as they are.
pub fn echoed_json(letters: &[Letter]) -> String {
  let mut out = String::from("{ \"people\": {\n");
  for (i, letter) in letters.iter().enumerate() {
    if i > 0 {
      out.push_str(",\n");
    }
    out.push_str(&format!("\"{}\": [\n", letter.name));
    for (j, p) in letter.people.iter().enumerate() {
      if j > 0 {
        out.push_str(",\n");
      }
      out.push_str(&format!(
        "{{ \"name\": \"{}\", \"about\": {{\"about\": \"{}\", \"linkedin\": \"{}\", \"picture\": \"{}\", \"profile\": \"{}\" }}, \"meetings\": [\n",
        p.name, p.about, p.linkedin, p.picture, p.profile
      ));
      for (k, (id, details)) in p.meetings.iter().enumerate() {
        if k > 0 {
          out.push_str(",\n");
        }
        out.push_str(&format!(
          "{{ \"id\": \"{id}\", \"details\": \"{details}\" }}"
        ));
      }
      out.push_str("] }");
    }
    out.push_str("\n]\n");
  }
  out.push_str("}}\n");
  out
}

/// The index as `jq .` prints it: two-space indent, `"key": value`.
pub fn jq_json(letters: &[Letter]) -> String {
  let s = |v: &str| serde_json::to_string(v).unwrap_or_default();
  let mut out = String::from("{\n  \"people\": {");
  if letters.is_empty() {
    out.push_str("}\n}\n");
    return out;
  }
  for (i, letter) in letters.iter().enumerate() {
    out.push_str(if i > 0 { ",\n" } else { "\n" });
    out.push_str(&format!("    {}: [", s(&letter.name)));
    if letter.people.is_empty() {
      out.push(']');
      continue;
    }
    for (j, p) in letter.people.iter().enumerate() {
      out.push_str(if j > 0 { ",\n" } else { "\n" });
      out.push_str(&format!(
        "      {{\n        \"name\": {},\n        \"about\": {{\n          \"about\": {},\n          \"linkedin\": {},\n          \"picture\": {},\n          \"profile\": {}\n        }},\n        \"meetings\": [",
        s(&p.name), s(&p.about), s(&p.linkedin), s(&p.picture), s(&p.profile)
      ));
      if p.meetings.is_empty() {
        out.push(']');
      } else {
        for (k, (id, details)) in p.meetings.iter().enumerate() {
          out.push_str(if k > 0 { ",\n" } else { "\n" });
          out.push_str(&format!(
            "          {{\n            \"id\": {},\n            \"details\": {}\n          }}",
            s(id),
            s(details)
          ));
        }
        out.push_str("\n        ]");
      }
      out.push_str("\n      }");
    }
    out.push_str("\n    ]");
  }
  out.push_str("\n  }\n}\n");
  out
}

/// `pplr index`: the Markdown index, dated today.
pub fn run_index(_args: &[String], config: &Config) -> Result<ExitCode> {
  print!(
    "{}",
    markdown(&letters(&config.data), &config.today().to_string())
  );
  Ok(ExitCode::SUCCESS)
}

pub fn markdown(letters: &[Letter], day: &str) -> String {
  let mut out =
    format!("---\nverblock: \"{day}:v0.1: Generated by people_index\"\n---\n# People Index\n\n");
  for letter in letters {
    out.push_str(&format!("##### {}\n", letter.name));
    for p in &letter.people {
      out.push_str(&format!(
        "[{}](<./{}/{}/About/{} (About).md>)\n",
        p.name, letter.name, p.name, p.stem
      ));
    }
    out.push('\n');
  }
  out
}

/// `pplr reindex`: .index/index.json and index.md, then the tags (on the
/// legacy `tags render` until WP06 ports it).
pub fn run_reindex(args: &[String], config: &Config) -> Result<ExitCode> {
  match args.first().map(String::as_str) {
    Some(flag) if flag == "--tags" || flag == "--stale-only" || flag.starts_with("--max-age=") => {
      println!("pplr reindex --tags is retired. Tags live in each person's About/tags.yaml:");
      println!("  pplr tag \"Surname, First\" +tag -tag   edit one person's tags");
      println!("  pplr tags check                        check everyone against the vocabulary");
      println!(
        "  pplr reindex                           rebuild indexes, tag pages and Tags lines"
      );
      return Ok(ExitCode::FAILURE);
    }
    Some("-h" | "--help") => {
      println!("Usage: pplr reindex");
      println!("  .index/index.json, .index/index.md, .index/tags_index.json, the tag pages");
      println!("  in _tags/, and the Tags line in each About (from each About/tags.yaml)");
      return Ok(ExitCode::SUCCESS);
    }
    _ => {}
  }
  let root = &config.data;
  let index = root.join(".index");
  fs::create_dir_all(&index)?;
  let tree = letters(root);
  println!("Generating: {}", index.join("index.json").display());
  crate::fs::write_atomic(&index.join("index.json"), jq_json(&tree).as_bytes())?;
  println!("Generating: {}", index.join("index.md").display());
  crate::fs::write_atomic(
    &index.join("index.md"),
    markdown(&tree, &config.today().to_string()).as_bytes(),
  )?;
  println!(
    "Generating: {}, {} and each About's Tags line",
    index.join("tags_index.json").display(),
    root.join("_tags/").display()
  );
  crate::fallback::run(&["tags".to_owned(), "render".to_owned()], config)
}
