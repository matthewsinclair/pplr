# The golden scenarios: the one list of what tests/golden/capture runs.
#
#   scenario NAME     starts a scenario; each starts from a fresh copy of fixture/
#   env KEY=VALUE     an extra setting for this scenario's steps
#   run ARG...        one `pplr ARG...` step; a scenario's steps run in order
#
# @SANDBOX@ in an argument or setting stands for the sandbox root, as it does in
# the recorded output. The settings every scenario shares are in the capture.

# Indexes and reports
scenario count-no-index;      run count
scenario json;                run json
scenario index;               run index
scenario reindex;             run reindex
scenario reindex-then-count;  run reindex; run count
scenario meetings;            run meetings 2026; run meetings 202603; run meetings 20260101 20260531; run meetings; run meetings 2026-13
# Each search has one hit: grep -r walks in readdir order, which the port does
# not keep (design D10), so a multi-hit order would be the filesystem's
scenario grep;                run grep ledgers; run grep Ledger; run grep -t berlin; run grep --tag nomatch; run grep zzz-nothing
scenario refresh;             run refresh status; run refresh next; run refresh next 2 --older-than 30; run refresh stamp "Varga, Zsófia" linkedin; run refresh status
scenario pictures-dry-run;    run pictures --dry-run
scenario pictures;            run pictures; run pictures "Varga, Zsófia"
scenario new;                 run new Wren Ashdown
scenario new-linkedin;        run new Wren Ashdown https://www.linkedin.com/in/wren-ashdown/
scenario new-existing;        run new Chidi Okafor

# Tags
scenario tag-show;            run tag "Okafor, Chidi"
scenario tag-edit;            run tag "Varga, Zsófia" +london -berlin
scenario tag-unknown;         run tag "Varga, Zsófia" +notaword
scenario tag-marker;          run tag pplr://o/okafor-chidi
scenario tags-check;          run tags check
scenario tags-show;           run tags show "Okafor, Chidi"; run tags show "Pemberton, Rufus"
scenario tags-render;         run tags render
scenario tags-index;          run tags index
scenario tags-apply;          run tags apply @SANDBOX@/inputs/tags-apply.json
scenario tags-unknown-sub;    run tags frobnicate

# Contact cadence (PPLR_TODAY is pinned for every scenario)
scenario contact-scan-dry-run; run contact scan --dry-run
scenario contact-scan;        run contact scan; run contact scan
scenario contact-due;         run contact due; run contact due 30; run contact due --all
scenario contact-show;        run contact show "Okafor, Chidi"; run contact show "Blackwood, Esme"
scenario contact-log;         run contact log "Hartigan, Mo" 2026-06-12 --via call --note "Discussed Fund II"
scenario contact-log-email;   run email recent; run contact log --email 2
scenario contact-next;        run contact next "Varga, Zsófia" +2w; run contact next "Varga, Zsófia" 2026-09-01
scenario contact-context;     run contact context "Okafor, Chidi"
scenario contact-render;      run contact render

# Email, through a fake mail-recent that keeps its rules (fakes/mail-recent)
scenario email;               run email; run email recent --sent; run email inbox; run email --bogus

# Rename, with refs reached through a symlink
scenario rename-dry-run;      run rename "Quennell, Imogen" "Quennell-Shaw, Imogen" --dry-run --refs @SANDBOX@/refs-link
scenario rename;              run rename "Quennell, Imogen" "Quennell-Shaw, Imogen" --refs @SANDBOX@/refs-link

# Links: paths, CMS URLs and a symlinked note
scenario links-dry-run;       run links @SANDBOX@/refs
scenario links-json;          run links @SANDBOX@/refs --json
scenario links-apply;         run links @SANDBOX@/refs --apply
scenario links-symlinked;     run links @SANDBOX@/refs/Notes/linked.md --apply

# Resolve and open --print
scenario resolve;             run resolve pplr://o/okafor-chidi; run resolve pplr://t/thistle-ottoline; run resolve "pplr://T/Thistle,%20Ottoline"; run resolve pplr://o/okafor-chidi/Meetings; run resolve pplr://tag/vip; run resolve pplr://z/nobody-here
scenario open-print-files;    env PPLR_CMS_URL=; run open --print pplr://o/okafor-chidi
# The CMS is named on loopback ports so the machine's own CMS (4360) never answers:
# 9 is closed; on 47360 the replay runs a stub, and the capture's fake curl answers
scenario open-print-cms-down; env PPLR_CMS_URL=http://127.0.0.1:9/people; run open --print pplr://o/okafor-chidi
scenario open-print-cms-up;   env PPLR_CMS_URL=http://127.0.0.1:47360/people; env PPLR_GOLDEN_CMS=up; run open --print pplr://v/varga-zsofia; run open --print "pplr://q/quennell-imogen/Meetings/20260415 Architecture chat"

# Sync, against the JSON store
scenario sync-usage;          run sync
scenario sync-check;          run sync --check; run sync --check --json; run sync --check -v
scenario sync-link-dry-run;   run sync --link
scenario sync-link-apply;     run sync --link --apply --name "Pemberton, Rufus"
scenario sync-link-all-names; run sync --link --apply --all-names
scenario sync-plan;           run sync --plan --out @SANDBOX@/plan.xlsx; run sync --plan
scenario sync-plan-json;      run sync --plan --json
scenario sync-apply-plan;     run sync --plan --out @SANDBOX@/plan.xlsx; run sync --apply-plan @SANDBOX@/plan.xlsx; run sync --apply-plan @SANDBOX@/plan.xlsx --apply
scenario sync-photos;         run sync --photos; run sync --photos --apply
scenario sync-pull;           run sync --pull; run sync --pull --apply --skip "Hartigan, Mo"
scenario sync-backup;         run sync --backup
