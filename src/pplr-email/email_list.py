"""pplr email: recent mail, or what is in the inbox, with each Message-ID to copy.

Reads mail-recent's JSON (Mail's index, read-only) from stdin and prints each
message, marking a sender who is in pplr with their pplr:// marker. The
Message-ID is what `pplr contact log PERSON --email ID` takes.
"""

import datetime
import json
import os
import re
import sys

MAILTO = re.compile(r"mailto:([^)\s>]+)", re.I)


def people_by_address():
    """address -> pplr:// marker, from each About's Email lines"""
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
                    out[a.lower()] = p["marker"]
    return out


def main():
    rows = json.load(sys.stdin)
    who = people_by_address()
    for r in rows:
        dt = datetime.datetime.fromisoformat(r["date"])
        marker = who.get((r.get("address") or "").lower())
        print(f"{dt:%a %d %b %H:%M}  to {r['account']}")
        print(f"  From: {r['from']}" + (f"   {marker}" if marker else ""))
        print(f"  Subj: {r['subject']}")
        print(f"  ID:   {r['message_id']}")
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
