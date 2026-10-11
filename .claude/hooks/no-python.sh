#!/bin/bash
# PreToolUse hook (Bash): refuse any command that runs Python or pip. This repository never runs Python, not even
# a version check: use jq, shell tools, cardgen, commander-sim or Rust. Matches python/python3/python3.x/pip/pipx/uv
# where a command starts (line start, after ; & | ( $( or a quote, optionally after sudo/env/exec or a path). A
# markdown backtick doesn't count, so commit messages and docs that merely mention Python still go through.
cmd=$(jq -r '.tool_input.command // ""')
start='(^|[;&|("'"'"']|\$\()'
prefix='[[:space:]]*((sudo|env|exec|command|nohup|time)[[:space:]]+([^[:space:]]+=[^[:space:]]*[[:space:]]+)*)*'
path='([^[:space:];&|()]*/)?'
prog='(python[0-9.]*|pip[0-9]*|pipx|uv)([[:space:]]|$|[;&|)])'
if printf '%s\n' "$cmd" | grep -qE "$start$prefix$path$prog"; then
  echo "Blocked: this repository never runs Python (not even --version or a one-liner). Use jq, grep/sed/awk, cardgen, commander-sim or a Rust test/example instead." >&2
  exit 2
fi
exit 0
