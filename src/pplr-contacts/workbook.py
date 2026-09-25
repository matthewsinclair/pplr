"""Render `pplr-contacts plan` JSON as a workbook for review.

Sheets: Merged (one row per pplr person, the proposal first and the
reviewer's columns beside it), pplr, Contacts, and How to review.
The reviewer edits Decision, Card, Dupes and Notes on Merged; the card refs
(C0001...) resolve through the Contacts sheet of the same workbook.

Usage: workbook.py PLAN.json OUT.xlsx
"""

import json
import sys

from openpyxl import Workbook
from openpyxl.formatting.rule import FormulaRule
from openpyxl.styles import Alignment, Font, PatternFill
from openpyxl.worksheet.datavalidation import DataValidation

DECISIONS = ["Add", "Update", "Link only", "No change", "Skip"]
REVIEW_FILL = PatternFill("solid", fgColor="FFF4D6")
LOW_FILL = PatternFill("solid", fgColor="F9D9A8")
MEDIUM_FILL = PatternFill("solid", fgColor="FDF1C4")
HEADER_FONT = Font(bold=True)
WRAP = Alignment(wrap_text=True, vertical="top")
TOP = Alignment(vertical="top")


def lines(values):
    return "\n".join(v for v in values if v)


def sheet(wb, title, headers, rows, widths, wrap=()):
    ws = wb.create_sheet(title)
    ws.append(headers)
    for cell in ws[1]:
        cell.font = HEADER_FONT
    for row in rows:
        ws.append(row)
    for i, w in enumerate(widths, start=1):
        ws.column_dimensions[ws.cell(row=1, column=i).column_letter].width = w
    for row in ws.iter_rows(min_row=2):
        for cell in row:
            cell.alignment = WRAP if cell.column in wrap else TOP
    ws.freeze_panes = "B2"
    ws.auto_filter.ref = ws.dimensions
    return ws


def main(plan_path, out_path):
    plan = json.load(open(plan_path))
    people = {p["key"]: p for p in plan["people"]}
    cards = {c["ref"]: c for c in plan["cards"]}

    wb = Workbook()
    wb.remove(wb.active)

    # Merged: the reviewer's columns (B to E) sit beside the person
    merged = []
    for r in plan["rows"]:
        p = people[r["person"]]
        c = cards.get(r.get("card"))
        merged.append([
            r["person"], r["decision"], r.get("card", ""), ", ".join(r["dupes"]), "",
            r["confidence"], r["why"], lines(r["changes"]),
            c["name"] if c else "",
            p["company"], c["organization"] if c else "",
            p["role"], c["jobTitle"] if c else "",
            lines(p["emails"]), lines(c["emails"]) if c else "",
            lines(p["phones"]), lines(c["phones"]) if c else "",
            p["linkedin"], lines(c["linkedin"]) if c else "",
            c["account"] if c else "",
            lines(f'{k["ref"]} {k["name"]} ({k["score"]}: {k["why"]})' for k in r["candidates"]),
        ])
    ws = sheet(wb, "Merged",
               ["Person", "Decision", "Card", "Dupes", "Notes", "Confidence", "Why", "Changes",
                "Card name", "Company (pplr)", "Company (card)", "Role (pplr)", "Role (card)",
                "Emails (pplr)", "Emails (card)", "Phones (pplr)", "Phones (card)",
                "LinkedIn (pplr)", "LinkedIn (card)", "Card account", "Other candidates"],
               merged,
               [26, 12, 9, 12, 28, 11, 34, 44, 22, 20, 20, 24, 24, 28, 28, 16, 16, 20, 20, 10, 44],
               wrap={7, 8, 14, 15, 16, 17, 19, 21})
    last = ws.max_row
    for col in "BCDE":
        for row in range(2, last + 1):
            ws[f"{col}{row}"].fill = REVIEW_FILL
    dv = DataValidation(type="list", formula1='"' + ",".join(DECISIONS) + '"', allow_blank=False,
                        showErrorMessage=True, errorTitle="Decision", error="One of: " + ", ".join(DECISIONS))
    ws.add_data_validation(dv)
    dv.add(f"B2:B{last}")
    ws.conditional_formatting.add(f"F2:F{last}", FormulaRule(formula=['$F2="Low"'], fill=LOW_FILL))
    ws.conditional_formatting.add(f"F2:F{last}", FormulaRule(formula=['$F2="Medium"'], fill=MEDIUM_FILL))

    sheet(wb, "pplr",
          ["Person", "Given", "Family", "Company", "Role", "Emails", "Phones", "LinkedIn", "pplr URL", "Photo", "Folder"],
          [[p["key"], p["given"], p["family"], p["company"], p["role"], lines(p["emails"]), lines(p["phones"]),
            p["linkedin"], p["marker"], "yes" if p.get("picture") else "", p["dir"]]
           for p in plan["people"]],
          [26, 14, 16, 22, 30, 30, 16, 22, 30, 7, 50], wrap={6, 7})

    sheet(wb, "Contacts",
          ["Card", "Name", "Company", "Role", "Emails", "Phones", "LinkedIn", "Account", "Groups",
           "pplr URL", "Proposed for", "Card id"],
          [[c["ref"], c["name"], c["organization"], c["jobTitle"], lines(c["emails"]), lines(c["phones"]),
            lines(c["linkedin"]), c["account"], ", ".join(c["groups"]), c.get("marker") or "",
            c.get("proposedFor") or "", c["id"]]
           for c in plan["cards"]],
          [8, 24, 22, 24, 30, 16, 20, 9, 16, 24, 26, 44], wrap={5, 6, 7})

    counts = {}
    for r in plan["rows"]:
        counts[(r["confidence"], r["decision"])] = counts.get((r["confidence"], r["decision"]), 0) + 1
    how = wb.create_sheet("How to review")
    text = [
        f"pplr and Apple Contacts: sync plan, {plan['generated']}",
        "",
        "Review the Merged sheet. Change only the four shaded columns; leave the rest as they are.",
        "  Decision: Add (a new iCloud card), Update (bring the card up to date from pplr), Link only (the pplr URL and group, no field changes), No change, or Skip (leave this person out of Contacts).",
        "  Card: the card to update or link, as a ref from the Contacts sheet (C0123). Change it if the plan picked the wrong card; clear it for Add.",
        "  Dupes: other cards that are the same person, comma-separated refs. They are listed for merging, never deleted without asking.",
        "  Notes: anything else, eg keep the card's name.",
        "",
        "Rows run least certain first: Low, then Medium, then High. Low and Medium rows are shaded in the Confidence column.",
        "Update never removes anything from a card. pplr wins on name, company and role; its emails, phones and LinkedIn are added beside the card's own. Addresses, birthdays, notes and photos on existing cards are left alone.",
        "Add makes an iCloud card from pplr with its photo, and every card touched gets the pplr URL and the PPLR group.",
        "Nothing is written until the reviewed workbook is applied, after a .vcf backup of every card.",
        "",
        "Proposals:",
    ] + [f"  {n:4d}  {c} / {d}" for (c, d), n in sorted(counts.items(), key=lambda kv: (["Low", "Medium", "High"].index(kv[0][0]), kv[0][1]))]
    for line in text:
        how.append([line])
    how.column_dimensions["A"].width = 140
    for row in how.iter_rows():
        row[0].alignment = Alignment(wrap_text=True, vertical="top")
    how["A1"].font = Font(bold=True, size=13)

    wb.save(out_path)


def decisions(xlsx_path, out_path):
    """The reviewed Merged sheet as [{person, decision, card_id}], card refs
    resolved through the same workbook's Contacts sheet"""
    from openpyxl import load_workbook
    wb = load_workbook(xlsx_path, read_only=True)
    ids = {}
    for row in wb["Contacts"].iter_rows(min_row=2, values_only=True):
        if row and row[0]:
            ids[row[0]] = row[11]
    out = []
    for row in wb["Merged"].iter_rows(min_row=2, values_only=True):
        if not row or not row[0]:
            continue
        person, decision, card = row[0], (row[1] or "").strip(), (row[2] or "").strip()
        if decision not in DECISIONS:
            sys.exit(f"{person}: Decision must be one of {', '.join(DECISIONS)}, not {decision!r}")
        if decision in ("Update", "Link only") and card not in ids:
            sys.exit(f"{person}: {decision} needs a Card ref from the Contacts sheet, not {card!r}")
        out.append({"person": person, "decision": decision, "card": ids.get(card) if card else None})
    json.dump(out, open(out_path, "w"), indent=1)


if __name__ == "__main__":
    if sys.argv[1] == "--decisions":
        decisions(sys.argv[2], sys.argv[3])
    else:
        main(sys.argv[1], sys.argv[2])
