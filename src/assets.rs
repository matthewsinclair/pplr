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
