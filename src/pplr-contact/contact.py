"""pplr contact: when you last spoke to each person, and when to get in touch next.

About/contact.yaml holds it, one per person:

    last: {date: 2026-01-28, via: meeting, link: "pplr://b/bray-martin/Meetings/20260128 Catch-up"}
    next: 2026-04-28
    cadence: 90

last is the latest contact: a meeting (found by scan, from the dated Meetings
folders) or an email, call or message you log. next is when to get in touch:
last plus the cadence, unless you set it. cadence is in days.

Who is on the contact roster, and how often, comes from the person's tags and
$PPLR_DATA/_pplr/cadence.yaml:

    roster: {contact: 180, vip: 90, career: 180}   # on the roster, with a default
    every: {3month: 90, 6month: 182, 12month: 365} # a cadence tag overrides it

So contact + 12month is once a year, and vip alone is once a quarter. A cadence
in contact.yaml overrides both. Off the roster: last is kept, and no next is set.

A Meetings folder dated after today is a booked meeting: that person is not due.

About/contact.md is a read-only view of contact.yaml, written whenever pplr
writes contact.yaml, and by render for everyone (idempotent). Edit the yaml.

Usage (via bin/pplr_contact):
    contact.py scan [--dry-run]     last from each person's newest meeting, and next from the cadence
    contact.py due [DAYS] [--all]   who to get in touch with: next on or before today (plus DAYS);
                                    --all lists everyone with a next date
    contact.py show PERSON          a person's contact.yaml, with their cadence
    contact.py log PERSON [DATE] [--via email|call|message|meeting] [--email MESSAGE-ID]
                   [--link URL] [--note TEXT]
                                    record a contact (DATE defaults to today); next moves on;
                                    --email links the message (pplr email shows the ids)
    contact.py render               About/contact.md from contact.yaml, for everyone
    contact.py next PERSON WHEN     set next: a date (2026-11-01) or from today (+2w, +10d, +3m)
    contact.py context PERSON       what to write from: role, latest update, last contact and its notes
"""

import datetime
import glob
import json
import os
import re
import sys

import yaml
from urllib.parse import quote

PEOPLE = os.environ.get("PPLR_DIR") or os.environ.get("PPLR_DATA") or os.path.expanduser("~/Dropbox/Career/People")
CADENCE = os.path.join(PEOPLE, "_pplr", "cadence.yaml")
MEETING_RE = re.compile(r"^(\d{4})(\d{2})(\d{2})\b")
VIAS = ("meeting", "email", "call", "message")
TODAY = datetime.date.today()
if os.environ.get("PPLR_TODAY"):
    TODAY = datetime.date.fromisoformat(os.environ["PPLR_TODAY"])


def person_dirs():
    return sorted(d for d in glob.glob(os.path.join(PEOPLE, "[A-Z]", "*, *")) if os.path.isdir(d))


def key_of(d):
    return os.path.relpath(d, PEOPLE)


def find_person(name):
    name = name.split("/", 1)[-1]
    d = os.path.join(PEOPLE, name[:1].upper(), name)
    if not os.path.isdir(d):
        sys.exit(f"Error: no such person: {name}")
    return d


def display_name(d):
    surname, first = os.path.basename(d).split(", ", 1)
    return f"{first} {surname}"


def markers():
    """key -> pplr:// marker, from the engine (one rule for markers)"""
    f = os.environ.get("PPLR_PEOPLE_JSON")
    return {p["key"]: p["marker"] for p in json.load(open(f))} if f else {}


def contact_file(d):
    return os.path.join(d, "About", "contact.yaml")


def as_date(v):
    if isinstance(v, datetime.date):
        return v
    return datetime.date.fromisoformat(str(v)) if v else None


def read_contact(d):
    f = contact_file(d)
    data = (yaml.safe_load(open(f)) or {}) if os.path.exists(f) else {}
    last = data.get("last") or {}
    if last:
        last = dict(last, date=as_date(last.get("date")))
    return {"last": last, "next": as_date(data.get("next")), "cadence": data.get("cadence")}


def write_contact(d, c):
    lines = [f"# Contact with {display_name(d)}: last spoke, and when to get in touch next.",
             "# pplr contact scan fills last from Meetings; pplr contact log records an email or call."]
    if c["last"]:
        extra = "".join(f", {k}: {json.dumps(c['last'][k], ensure_ascii=False)}"
                        for k in ("link", "note") if c["last"].get(k))
        lines.append(f"last: {{date: {c['last']['date']}, via: {c['last'].get('via', 'meeting')}{extra}}}")
    if c["next"]:
        lines.append(f"next: {c['next']}")
    if c["cadence"]:
        lines.append(f"cadence: {c['cadence']}")
    open(contact_file(d), "w").write("\n".join(lines) + "\n")
    render_one(d, c)


def md_link(label, target):
    return f"[{label}](<{target}>)" if re.search(r"[ ()<>]", target) else f"[{label}]({target})"


def nice(date):
    return f"{date.day} {date:%b %Y}"


def render_one(d, c, defaults=None):
    """About/contact.md: contact.yaml to read (the CMS shows it); True if it changed"""
    days, why = cadence_of(d, c, defaults if defaults is not None else default_cadences())
    lines = ["<!-- Generated from contact.yaml by pplr contact: edit contact.yaml, not this file. -->",
             f"# Contact: {display_name(d)}", ""]
    last = c["last"]
    if last:
        target = last.get("link") or ""
        m = re.match(r"pplr://[a-z]/[^/]+/(Meetings/.+)$", target)
        if m:
            target = "../" + m.group(1).rstrip("/") + "/"
        what = f"{nice(last['date'])}, {last.get('via', 'meeting')}"
        lines.append(f"- Last contact: {what}" + (f": {md_link(target.rstrip('/').rsplit('/', 1)[-1] if m else ('the email' if target.startswith('message:') else 'link'), target)}" if target else ""))
        if last.get("note"):
            lines.append(f"- Note: {last['note']}")
    else:
        lines.append("- Last contact: none recorded")
    lines.append(f"- Next: {nice(c['next'])}" if c["next"] else "- Next: none")
    b = booked(d)
    if b:
        lines.append(f"- Booked: {nice(b[0])}, {md_link(b[1], '../Meetings/' + b[1] + '/')}")
    lines.append(f"- Cadence: every {days} days (from {why})" if days else "- Cadence: not on the contact roster")
    text = "\n".join(lines) + "\n"
    f = os.path.join(d, "About", "contact.md")
    if os.path.exists(f) and open(f).read() == text:
        return False
    open(f, "w").write(text)
    return True


def render():
    defaults = default_cadences()
    n = sum(render_one(d, read_contact(d), defaults) for d in person_dirs() if os.path.exists(contact_file(d)))
    print(f"pplr contact render: {n} contact.md written")
    return 0


def tags_of(d):
    f = os.path.join(d, "About", "tags.yaml")
    if not os.path.exists(f):
        return set()
    return {str(t["tag"] if isinstance(t, dict) else t) for t in (yaml.safe_load(open(f)) or {}).get("tags") or []}


def default_cadences():
    """{roster: {tag: days}, every: {tag: days}}"""
    v = (yaml.safe_load(open(CADENCE)) or {}) if os.path.exists(CADENCE) else {}
    return {k: {str(t): int(n) for t, n in (v.get(k) or {}).items()} for k in ("roster", "every")}


def cadence_of(d, c, defaults):
    """(days, why): the file's own cadence, else a cadence tag, else the shortest roster tag's"""
    if c["cadence"]:
        return int(c["cadence"]), "contact.yaml"
    tags = tags_of(d)
    for kind in ("every", "roster"):
        hits = sorted((defaults[kind][t], t) for t in tags if t in defaults[kind])
        if hits:
            return hits[0][0], f"#{hits[0][1]}"
    return None, ""


def meetings(d):
    """[(date, folder name)], oldest first"""
    out = []
    for m in glob.glob(os.path.join(d, "Meetings", "*")):
        g = MEETING_RE.match(os.path.basename(m))
        if os.path.isdir(m) and g:
            try:
                out.append((datetime.date(*map(int, g.groups())), os.path.basename(m)))
            except ValueError:
                pass
    return sorted(out)


def booked(d):
    ahead = [m for m in meetings(d) if m[0] > TODAY]
    return ahead[0] if ahead else None


def moved_on(c, days):
    """next from last and the cadence; with no contact yet, today"""
    if not days:
        return c["next"]
    return c["last"]["date"] + datetime.timedelta(days=days) if c["last"] else TODAY


def scan(dry):
    defaults, mk = default_cadences(), markers()
    wrote = created = 0
    for d in person_dirs():
        c = read_contact(d)
        before = json.dumps(c, default=str, sort_keys=True)
        past = [m for m in meetings(d) if m[0] <= TODAY]
        if past and (not c["last"] or past[-1][0] > c["last"]["date"]):
            date, folder = past[-1]
            c["last"] = {"date": date, "via": "meeting", "link": f"{mk.get(key_of(d), '')}/Meetings/{folder}".lstrip("/")}
            c["next"] = None
        days, _ = cadence_of(d, c, defaults)
        if days and not c["next"]:
            c["next"] = moved_on(c, days)
        if json.dumps(c, default=str, sort_keys=True) == before or not (c["last"] or c["next"]):
            continue
        created += not os.path.exists(contact_file(d))
        wrote += 1
        if not dry:
            write_contact(d, c)
    print(f"pplr contact scan{' (dry run)' if dry else ''}: {wrote} contact.yaml {'to write' if dry else 'written'} ({created} new)")
    return 0


def since(date):
    return f"{(TODAY - date).days} days ago" if date else "never"


def due(within, everyone):
    horizon = TODAY + datetime.timedelta(days=within)
    rows = []
    for d in person_dirs():
        c = read_contact(d)
        if not c["next"] or (not everyone and (c["next"] > horizon or booked(d))):
            continue
        rows.append((c["next"], d, c))
    for nxt, d, c in sorted(rows, key=lambda r: (r[0], r[1])):
        last = c["last"]
        was = f"last {last['date']} {last.get('via', 'meeting')} ({since(last['date'])})" if last else "no contact yet"
        b = booked(d)
        print(f"{nxt}  {key_of(d)}  {was}" + (f"  booked {b[0]}" if b else ""))
    if not rows:
        print("Nobody is due.")
    return 0


def show(name):
    d = find_person(name)
    c = read_contact(d)
    days, why = cadence_of(d, c, default_cadences())
    print(f"{display_name(d)}")
    if c["last"]:
        print(f"  last:    {c['last']['date']} {c['last'].get('via', 'meeting')} ({since(c['last']['date'])})")
        for k in ("link", "note"):
            if c["last"].get(k):
                print(f"           {c['last'][k]}")
    else:
        print("  last:    none recorded")
    print(f"  next:    {c['next'] or 'none'}" + (f"  (booked {booked(d)[0]})" if booked(d) else ""))
    print(f"  cadence: {f'{days} days, from {why}' if days else 'none'}")
    return 0


def log(name, args):
    d = find_person(name)
    c = read_contact(d)
    date, via, link, note = TODAY, "email", None, None
    it = iter(args)
    for a in it:
        if a == "--via":
            via = next(it, "email")
        elif a == "--link":
            link = next(it, None)
        elif a == "--email":
            mid = next(it, "").strip().strip("<>")
            via, link = "email", f"message://%3C{quote(mid, safe='@.-_=')}%3E"
        elif a == "--note":
            note = next(it, None)
        else:
            date = datetime.date.fromisoformat(a)
    if via not in VIAS:
        sys.exit(f"Error: --via is one of {', '.join(VIAS)}")
    if c["last"] and c["last"]["date"] > date:
        print(f"{display_name(d)}: kept the later contact on {c['last']['date']}"); return 0
    c["last"] = {"date": date, "via": via, "link": link, "note": note}
    days, _ = cadence_of(d, c, default_cadences())
    c["next"] = moved_on(c, days) if days else None
    write_contact(d, c)
    print(f"{display_name(d)}: {via} on {date}" + (f"; next {c['next']}" if c["next"] else ""))
    return 0


def set_next(name, when):
    d = find_person(name)
    c = read_contact(d)
    m = re.fullmatch(r"\+(\d+)([dwm])", when)
    if m:
        n = int(m.group(1))
        c["next"] = TODAY + datetime.timedelta(days=n * {"d": 1, "w": 7, "m": 30}[m.group(2)])
    else:
        c["next"] = datetime.date.fromisoformat(when)
    write_contact(d, c)
    print(f"{display_name(d)}: next {c['next']}")
    return 0


def notes_of(folder):
    """the meeting's summary, else its notes, else its one note file"""
    for f in ("Summary.md", "Notes.md", os.path.basename(folder) + ".md"):
        if os.path.exists(os.path.join(folder, f)):
            return os.path.join(folder, f)
    md = sorted(glob.glob(os.path.join(folder, "*.md")))
    return md[0] if md else None


def context(name):
    d = find_person(name)
    c = read_contact(d)
    about = glob.glob(os.path.join(d, "About", "* (About).md"))
    text = open(about[0]).read() if about else ""
    print(f"# {display_name(d)}\n")
    for field in ("Role", "Company", "Headline", "Email"):
        m = re.search(r"^- " + field + r":[ \t]*(.*)$", text, re.M)
        v = re.sub(r"\[(.*?)\]\(.*?\)", r"\1", m.group(1)).strip() if m else ""
        if v:
            print(f"- {field}: {v}")
    print(f"- Tags: {' '.join(sorted(tags_of(d)))}")
    updates = re.findall(r"^\*\*Update, [^*]+\*\* .*$", text, re.M)
    if updates:
        print(f"\nLatest update: {updates[0]}")
    last = c["last"]
    print(f"\nLast contact: {f'{last['date']} {last.get('via', 'meeting')} ({since(last['date'])})' if last else 'none recorded'}")
    if last.get("note"):
        print(f"Note: {last['note']}")
    past = [m for m in meetings(d) if m[0] <= TODAY]
    if past:
        folder = os.path.join(d, "Meetings", past[-1][1])
        f = notes_of(folder)
        print(f"\n## Last meeting: {past[-1][1]}\n")
        if f:
            print(f"({os.path.relpath(f, PEOPLE)})\n")
            print("\n".join(open(f, errors="replace").read().splitlines()[:80]))
    return 0


def main(argv):
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__.split("Usage (via bin/pplr_contact):")[1]); return 0
    cmd, rest = argv[0], argv[1:]
    if cmd == "scan":
        return scan("--dry-run" in rest)
    if cmd == "due":
        days = next((int(a) for a in rest if a.isdigit()), 0)
        return due(days, "--all" in rest)
    if cmd == "show" and rest:
        return show(rest[0])
    if cmd == "log" and rest:
        return log(rest[0], rest[1:])
    if cmd == "next" and len(rest) == 2:
        return set_next(rest[0], rest[1])
    if cmd == "context" and rest:
        return context(rest[0])
    if cmd == "render":
        return render()
    print(f"pplr contact: unknown command {cmd!r}"); return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
