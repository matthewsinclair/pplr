"""pplr tags: each person's tags, from About/tags.yaml, against the vocabulary.

About/tags.yaml is the only place a person's tags live:

    tags:
      - fintech                      # a bare word is a hand tag
      - {tag: cto, source: auto, at: 2026-09-25}
      - {tag: bcg, source: inferred, evidence: "BCG DV 2017-2020, Profile.pdf"}

source is hand (yours: never overwritten), auto (from their profile: replaced
by a re-run) or inferred (a relationship, from evidence: replaced by a re-run
unless you make it hand). A tag's facet comes from the vocabulary,
$PPLR_DATA/_pplr/vocabulary.yaml, so it is not repeated here.

Usage (via bin/pplr_tags):
    tags.py check                 every tags.yaml against the vocabulary
    tags.py show PERSON           a person's tags, by facet
    tags.py render [--dry-run]    the "- Tags:" line in each About, from tags.yaml
    tags.py apply FILE [--dry-run]
                                  write tags.yaml from a JSON list of
                                  {person, auto: [tag], inferred: [{tag, evidence}], hand: [tag]};
                                  hand tags already in the file are kept
"""

import datetime
import glob
import json
import os
import re
import sys

import yaml

PEOPLE = os.environ.get("PPLR_DIR") or os.environ.get("PPLR_DATA") or os.path.expanduser("~/Dropbox/Career/People")
VOCAB = os.path.join(PEOPLE, "_pplr", "vocabulary.yaml")
FACETS = ["role", "function", "sector", "org", "relationship", "place"]
RENDER_NOTE = "<!-- from tags.yaml: pplr tags render -->"
TAG_RE = re.compile(r"^[a-z0-9]+$")


def vocabulary():
    v = yaml.safe_load(open(VOCAB))
    facet_of, fold = {}, {}
    for f in FACETS:
        for tag, d in (v.get(f) or {}).items():
            facet_of[tag] = f
            fold[tag] = [tag]
            for a in (d or {}).get("also", []):
                fold[a] = [tag]
    for old, tags in (v.get("split") or {}).items():
        fold[old] = list(tags)
    return facet_of, fold


def person_dirs():
    return sorted(d for d in glob.glob(os.path.join(PEOPLE, "[A-Z]", "*, *")) if os.path.isdir(d))


def key_of(d):
    return os.path.relpath(d, PEOPLE)


def find_person(name):
    name = name.split("/", 1)[-1]
    d = os.path.join(PEOPLE, name[0].upper(), name)
    if not os.path.isdir(d):
        sys.exit(f"pplr tags: no such person: {name}")
    return d


def tags_file(d):
    return os.path.join(d, "About", "tags.yaml")


def read_tags(d):
    """[{tag, source, ...}], a bare word read as a hand tag"""
    f = tags_file(d)
    if not os.path.exists(f):
        return []
    data = yaml.safe_load(open(f)) or {}
    out = []
    for t in data.get("tags") or []:
        out.append({"tag": str(t), "source": "hand"} if not isinstance(t, dict) else dict(t))
    return out


def write_tags(d, tags, name):
    lines = [f"# Tags for {name}. A bare word is your own tag; auto and inferred tags",
             "# are rewritten by pplr, hand tags never are. Vocabulary: _pplr/vocabulary.yaml",
             "tags:"]
    facet_of, _ = vocabulary()
    order = {f: i for i, f in enumerate(FACETS)}
    for t in sorted(tags, key=lambda t: (order.get(facet_of.get(t["tag"]), 9), t["source"] != "hand")):
        if t["source"] == "hand":
            lines.append(f"  - {t['tag']}")
        else:
            extra = "".join(f", {k}: {json.dumps(v, ensure_ascii=False) if k == 'evidence' else v}"
                            for k, v in t.items() if k not in ("tag", "source"))
            lines.append(f"  - {{tag: {t['tag']}, source: {t['source']}{extra}}}")
    open(tags_file(d), "w").write("\n".join(lines) + "\n")


def display_name(d):
    surname, first = os.path.basename(d).split(", ", 1)
    return f"{first} {surname}"


def check():
    facet_of, fold = vocabulary()
    problems = 0
    count = 0
    for d in person_dirs():
        if not os.path.exists(tags_file(d)):
            continue
        count += 1
        try:
            tags = read_tags(d)
        except yaml.YAMLError as e:
            print(f"{key_of(d)}: unreadable tags.yaml: {e}".splitlines()[0]); problems += 1; continue
        seen = set()
        for t in tags:
            tag = t.get("tag", "")
            if tag in seen:
                print(f"{key_of(d)}: {tag} twice"); problems += 1
            seen.add(tag)
            if not TAG_RE.match(tag):
                print(f"{key_of(d)}: {tag!r} is not a single lowercase word"); problems += 1
            elif tag not in facet_of:
                hint = f" (the vocabulary folds it into {', '.join(fold[tag])})" if tag in fold else ""
                print(f"{key_of(d)}: {tag} is not in the vocabulary{hint}"); problems += 1
            if t.get("source") not in ("hand", "auto", "inferred"):
                print(f"{key_of(d)}: {tag} has source {t.get('source')!r}"); problems += 1
            if t.get("source") == "inferred" and facet_of.get(tag) != "relationship":
                print(f"{key_of(d)}: {tag} is inferred but not a relationship tag"); problems += 1
    print(f"pplr tags check: {count} tags.yaml, {problems} problem(s)")
    return 1 if problems else 0


def show(name):
    facet_of, _ = vocabulary()
    d = find_person(name)
    tags = read_tags(d)
    for f in FACETS:
        here = [t for t in tags if facet_of.get(t["tag"]) == f]
        if here:
            print(f"{f:13} " + "  ".join(t["tag"] + ("" if t["source"] == "hand" else f" ({t['source']})") for t in here))
    other = [t["tag"] for t in tags if t["tag"] not in facet_of]
    if other:
        print(f"{'not in vocab':13} " + "  ".join(other))
    return 0


def tags_line(d, facet_of):
    order = {f: i for i, f in enumerate(FACETS)}
    tags = sorted(read_tags(d), key=lambda t: order.get(facet_of.get(t["tag"]), 9))
    return ("- Tags:     " + " ".join(f"#{t['tag']}" for t in tags) + " " + RENDER_NOTE) if tags else None


def render(dry):
    """The generated Tags line sits at the end of the About header list, before _About_"""
    facet_of, _ = vocabulary()
    changed = same = 0
    for d in person_dirs():
        about = next(iter(sorted(glob.glob(os.path.join(d, "About", "*(About).md")))), None)
        if not about:
            continue
        s = open(about, encoding="utf-8").read()
        line = tags_line(d, facet_of)
        old = re.search(r"^- Tags:.*" + re.escape(RENDER_NOTE) + r"[ \t]*\n", s, re.M)
        if old:
            new = s[:old.start()] + (line + "\n" if line else "") + s[old.end():]
        elif line:
            # after the last "- Field:" line of the first header list
            header = list(re.finditer(r"^- [A-Za-z]+:.*\n", s, re.M))
            if not header:
                continue
            last = header[0]
            for m in header[1:]:
                if m.start() == last.end():
                    last = m
                else:
                    break
            new = s[:last.end()] + line + "\n" + s[last.end():]
        else:
            new = s
        if new == s:
            same += 1
            continue
        changed += 1
        if not dry:
            open(about, "w", encoding="utf-8").write(new)
    print(f"pplr tags render{' (dry run)' if dry else ''}: {changed} About file(s) {'to change' if dry else 'changed'}, {same} already right")
    return 0


def apply(path, dry):
    facet_of, _ = vocabulary()
    today = datetime.date.today().isoformat()
    rows = json.load(open(path))
    written = 0
    for r in rows:
        d = find_person(r["person"])
        hand = {t["tag"] for t in read_tags(d) if t["source"] == "hand"} | set(r.get("hand", []))
        tags = [{"tag": t, "source": "hand"} for t in sorted(hand)]
        for t in r.get("auto", []):
            if t not in hand:
                tags.append({"tag": t, "source": "auto", "at": today})
        for t in r.get("inferred", []):
            if t["tag"] not in hand:
                tags.append({"tag": t["tag"], "source": "inferred", "evidence": t.get("evidence", "")})
        bad = [t["tag"] for t in tags if t["tag"] not in facet_of]
        if bad:
            print(f"{r['person']}: not in the vocabulary, left out: {', '.join(bad)}")
            tags = [t for t in tags if t["tag"] in facet_of]
        if not dry:
            write_tags(d, tags, display_name(d))
        written += 1
    print(f"pplr tags apply{' (dry run)' if dry else ''}: {written} tags.yaml {'to write' if dry else 'written'}")
    return 0


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__.split("Usage (via bin/pplr_tags):")[1]); return 0
    cmd, rest = argv[0], argv[1:]
    dry = "--dry-run" in rest
    rest = [a for a in rest if a != "--dry-run"]
    if cmd == "check":
        return check()
    if cmd == "show" and rest:
        return show(rest[0])
    if cmd == "render":
        return render(dry)
    if cmd == "apply" and rest:
        return apply(rest[0], dry)
    print(f"pplr tags: unknown command {cmd!r}"); return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
