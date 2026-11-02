//! What ships inside the binary (ST0001 design D6), from the repo's one
//! source, `templates/`. The data tree's `_pplr/` overrides it.

/// The Finder icons `applyicons` sets, by file name.
pub const ICONS: [(&str, &[u8]); 3] = [
  ("About.pdf", include_bytes!("../templates/icons/About.pdf")),
  (
    "Calendar.pdf",
    include_bytes!("../templates/icons/Calendar.pdf"),
  ),
  (
    "Meeting.pdf",
    include_bytes!("../templates/icons/Meeting.pdf"),
  ),
];

/// The person template `new` copies, by path inside the person's folder.
pub const PERSON_FILES: [(&str, &[u8]); 2] = [
  (
    "About/Firstname Surname (About).md",
    include_bytes!("../templates/person/About/Firstname Surname (About).md"),
  ),
  (
    "Meetings/20YYMMDD Meeting/20YYMMDD Meeting.md",
    include_bytes!("../templates/person/Meetings/20YYMMDD Meeting/20YYMMDD Meeting.md"),
  ),
];

/// The template's folders, the empty Client/ among them.
pub const PERSON_DIRS: [&str; 4] = ["About", "Client", "Meetings", "Meetings/20YYMMDD Meeting"];

/// Where the data tree overrides the person template.
pub const PERSON_OVERRIDE: &str = "_pplr/templates/person";
