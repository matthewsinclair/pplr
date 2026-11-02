-- Pplr Links.app: opens pplr:// links with `pplr open` (ST0001 WP08). It is
-- built and signed with the release, and runs the pplr in its own tree: beside
-- it as bin/pplr in an installed tree, or as pplr in a dev build's target dir.
-- No path is written into it, so one signed app serves any tree.

on open location theURL
  set here to POSIX path of (path to me)
  set pplr to do shell script "d=$(dirname " & quoted form of here & "); if [ -x \"$d/bin/pplr\" ]; then echo \"$d/bin/pplr\"; else echo \"$d/pplr\"; fi"
  do shell script quoted form of pplr & " open " & quoted form of theURL
end open location

on run
  display dialog "Pplr Links opens pplr:// links. There is nothing to do here." buttons {"OK"} default button 1
end run
