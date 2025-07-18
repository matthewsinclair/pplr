#!/bin/zsh

# Search for directories with "Matthew Sinclair's conflicted copy" in the name
find . -type d -name "*Matthew Sinclair's conflicted copy*" -exec rm -rf {} +
