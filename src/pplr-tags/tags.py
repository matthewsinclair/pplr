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
    tags.py render [--dry-run]    the "- Tags:" line in each About, linking each tag to
                                  its page, and the tag pages in $PPLR_DATA/_tags/
    tags.py index                 .index/tags_index.json: everyone with name, marker,
                                  role, company, picture and tags by facet
    tags.py edit PERSON +tag -tag add or remove hand tags, checked against the vocabulary
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
TAGS_DIR = "_tags"          # generated tag pages, under the people root
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
    """Each tag links to its page: About/ is two folders below the people root"""
    order = {f: i for i, f in enumerate(FACETS)}
    tags = sorted(read_tags(d), key=lambda t: order.get(facet_of.get(t["tag"]), 9))
    links = " ".join(f"[#{t['tag']}](../../../{TAGS_DIR}/{t['tag']}.md)" for t in tags)
    return f"- Tags:     {links} {RENDER_NOTE}" if tags else None


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
    if not dry:
        pages(facet_of)
        index()
    return 0


def header_field(s, field):
    m = re.search(r"^- " + field + r":[ \t]*(.*)$", s, re.M)
    if not m:
        return ""
    v = m.group(1).strip()
    t = re.match(r"\[(.*?)\]\(", v)
    return (t.group(1) if t else v).strip()


def md_link_target(path):
    return "<" + path + ">" if re.search(r"[ ()<>]", path) else path


def pages(facet_of):
    """_tags/index.md (every tag by facet, with counts) and _tags/<tag>.md
    (everyone with the tag, and the tags seen with it). Generated: rewritten
    in full each time, and files for tags no longer used are removed."""
    out = os.path.join(PEOPLE, TAGS_DIR)
    os.makedirs(out, exist_ok=True)
    v = yaml.safe_load(open(VOCAB))
    people = {}                                 # key -> {name, line, tags}
    for d in person_dirs():
        tags = [t["tag"] for t in read_tags(d) if t["tag"] in facet_of]
        if not tags:
            continue
        about = next(iter(sorted(glob.glob(os.path.join(d, "About", "*(About).md")))), None)
        s = open(about, encoding="utf-8").read() if about else ""
        role, company = header_field(s, "Role"), header_field(s, "Company")
        places = [t for t in tags if facet_of[t] == "place"]
        what = " at ".join(x for x in (role, company) if x)
        rel = os.path.relpath(about or d, out)
        people[key_of(d)] = {"name": display_name(d), "link": md_link_target(rel), "what": what,
                             "places": places, "tags": tags}
    by_tag = {}
    for k, p in people.items():
        for t in p["tags"]:
            by_tag.setdefault(t, []).append(k)
    stamp = datetime.date.today().isoformat()
    note = f"<!-- generated by pplr tags render, {stamp}: edit tags in each person's About/tags.yaml -->"
    written = set()
    for tag, keys in by_tag.items():
        keys.sort(key=lambda k: people[k]["name"].split(" ")[-1].lower() + people[k]["name"].lower())
        f = facet_of[tag]
        d = (v.get(f) or {}).get(tag) or {}
        lines = [note, f"# #{tag}", "", f"{f.capitalize()}" + (f": {d['note']}" if d.get("note") else "")
                 + f". {len(keys)} {'person' if len(keys) == 1 else 'people'}. [All tags](index.md)", ""]
        for k in keys:
            p = people[k]
            extra = " · ".join(x for x in (p["what"], " ".join(f"#{t}" for t in p["places"])) if x)
            lines.append(f"- [{p['name']}]({p['link']})" + (f": {extra}" if extra else ""))
        with_it = {}
        for k in keys:
            for t in people[k]["tags"]:
                if t != tag:
                    with_it[t] = with_it.get(t, 0) + 1
        common = sorted(with_it.items(), key=lambda x: (-x[1], x[0]))[:20]
        if common:
            lines += ["", "## Seen with", "", " · ".join(f"[#{t}]({t}.md) {n}" for t, n in common)]
        open(os.path.join(out, f"{tag}.md"), "w", encoding="utf-8").write("\n".join(lines) + "\n")
        written.add(f"{tag}.md")
    idx = [note, "# Tags", "", f"{len(by_tag)} tags across {len(people)} people. Each tag lists everyone who has it.", ""]
    for f in FACETS:
        here = sorted((t for t in by_tag if facet_of[t] == f), key=lambda t: (-len(by_tag[t]), t))
        if here:
            idx += [f"## {f.capitalize()}", "", " · ".join(f"[#{t}]({t}.md) {len(by_tag[t])}" for t in here), ""]
    open(os.path.join(out, "index.md"), "w", encoding="utf-8").write("\n".join(idx))
    written.add("index.md")
    for old in os.listdir(out):
        if old.endswith(".md") and old not in written:
            os.remove(os.path.join(out, old))
    print(f"pplr tags render: {len(by_tag)} tag pages and an index in {out}")


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


def index():
    """One JSON index of everyone, for pplr search and the CMS: the engine's
    people list (key, marker, About, picture) joined with each tags.yaml"""
    facet_of, _ = vocabulary()
    listing = os.environ.get("PPLR_PEOPLE_JSON")
    people = json.load(open(listing)) if listing else [{"key": key_of(d), "marker": "", "about": "", "picture": None}
                                                       for d in person_dirs()]
    out = []
    for p in people:
        d = os.path.join(PEOPLE, p["key"])
        about = p.get("about") or ""
        s = open(about, encoding="utf-8").read() if about.endswith(".md") and os.path.exists(about) else ""
        tags = [t["tag"] for t in read_tags(d) if t["tag"] in facet_of]
        out.append({
            "name": os.path.basename(d), "display": display_name(d), "path": p["key"],
            "about": os.path.relpath(about, PEOPLE) if about else "", "marker": p.get("marker", ""),
            "role": header_field(s, "Role"), "company": header_field(s, "Company"),
            "picture": os.path.relpath(p["picture"], PEOPLE) if p.get("picture") else None,
            "tags": tags,
            "facets": {f: [t for t in tags if facet_of[t] == f] for f in FACETS if any(facet_of[t] == f for t in tags)},
        })
    os.makedirs(os.path.join(PEOPLE, ".index"), exist_ok=True)
    doc = {"generated_at": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
           "people_count": len(out), "people": out}
    json.dump(doc, open(os.path.join(PEOPLE, ".index", "tags_index.json"), "w"), indent=1, ensure_ascii=False)
    print(f"pplr tags index: {len(out)} people in .index/tags_index.json")
    return 0


def edit(name, changes):
    """+tag adds a hand tag (or makes an auto or inferred one yours); -tag removes it"""
    facet_of, fold = vocabulary()
    d = find_person(name)
    tags = read_tags(d)
    for c in changes:
        op, tag = c[0], c[1:].lower()
        if op not in "+-" or not tag:
            sys.exit(f"pplr tag: say +tag or -tag, not {c!r}")
        if op == "+":
            if tag not in facet_of:
                hint = f"; the vocabulary folds it into {', '.join(fold[tag])}" if tag in fold else ""
                sys.exit(f"pplr tag: {tag} is not in the vocabulary{hint}")
            tags = [t for t in tags if t["tag"] != tag] + [{"tag": tag, "source": "hand"}]
        else:
            tags = [t for t in tags if t["tag"] != tag]
    write_tags(d, tags, display_name(d))
    return show(name)


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
    if cmd == "index":
        return index()
    if cmd == "edit" and len(rest) >= 2:
        return edit(rest[0], rest[1:])
    if cmd == "apply" and rest:
        return apply(rest[0], dry)
    print(f"pplr tags: unknown command {cmd!r}"); return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
