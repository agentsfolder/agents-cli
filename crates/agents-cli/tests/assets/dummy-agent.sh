#!/bin/sh
set -eu
printf "%s\n" "$@" > run-args.txt
exit 42
