"""pplr email: recent mail, or what is in the inbox, numbered, with each Message-ID.

Reads mail-recent's JSON (Mail's index, read-only) from stdin and prints each
message with a number, marking the person it is with when they are in pplr:
the sender, or for mail you sent, the first recipient in pplr. The listing is
saved (email-last.json in $PPLR_CACHE_DIR), so `pplr contact log --email 3`
knows message 3: its Message-ID, date, subject and person.
"""

import datetime
import json
import os
import re
import sys

MAILTO = re.compile(r"mailto:([^)\s>]+)", re.I)
CACHE = os.path.join(os.environ.get("PPLR_CACHE_DIR") or os.path.expanduser("~/Library/Caches/pplr"), "email-last.json")


def people_by_address():
    """address -> (key, pplr:// marker), from each About's Email lines"""
    f = os.environ.get("PPLR_PEOPLE_JSON")
    out = {}
    for p in (json.load(open(f)) if f else []):
        try:
            text = open(p["about"], errors="replace").read()
        except (OSError, TypeError):
            continue
        for line in text.splitlines():
            if line.startswith("- Email:"):
                for a in MAILTO.findall(line):
                    out[a.lower()] = (p["key"], p["marker"])
    return out


def main():
    rows = json.load(sys.stdin)
    who = people_by_address()
    saved = []
    for n, r in enumerate(rows, 1):
        sent = (r.get("address") or "").lower() == (r.get("account") or "").lower()
        candidates = r.get("to", []) if sent else [r.get("address") or ""]
        person = next((who[a.lower()] for a in candidates if a.lower() in who), None)
        dt = datetime.datetime.fromisoformat(r["date"])
        tag = f"   {person[1]}" if person else ""
        if sent:
            print(f"[{n}] {dt:%a %d %b %H:%M}  from {r['account']}")
            print(f"  To:   {', '.join(r.get('to', []))}{tag}")
        else:
            print(f"[{n}] {dt:%a %d %b %H:%M}  to {r['account']}")
            print(f"  From: {r['from']}{tag}")
        print(f"  Subj: {r['subject']}")
        print(f"  ID:   {r['message_id']}")
        print()
        saved.append({"n": n, "date": r["date"], "message_id": r["message_id"], "subject": r["subject"],
                      "direction": "out" if sent else "in", "person": person[0] if person else None})
    os.makedirs(os.path.dirname(CACHE), exist_ok=True)
    json.dump(saved, open(CACHE, "w"), indent=1, ensure_ascii=False)
    return 0


if __name__ == "__main__":
    sys.exit(main())
