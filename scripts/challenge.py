#!/usr/bin/env python3

# Run all challenges whose solution code has changed since the last commit.

import argparse
import subprocess
import os.path
import re

def build_solver(day):
    cmd = ["cargo", "build", "--release", "--bin", day]
    subprocess.run(cmd, cwd="advent").check_returncode()

def solve_challenge(day, args):
    challenge_file = "{}/{}/challenge".format(args.challenge, day)
    solver = "advent/target/release/{}".format(day)

    with open(challenge_file, 'rb') as challenge:
        subprocess.run(solver, stdin=challenge, text="UTF-8").check_returncode()

def main(args):
    # First: use git to list all files changed since the last commit.
    # Using a naive strategy (if the file `src/bin/dayXY.rs` or any file in `src/bin/dayXY/` changes),
    # re-run any updated solvers.

    standalone_pat = re.compile(r"(day\d+).rs")
    directory_pat = re.compile(r"advent/src/bin/(day\d+)")

    # We use HEAD^ since this script is intended for merge-protected branches; i.e. the tip of the branch
    # will always be a merge commit.
    git_diff_cmd = ["git", "diff", "--name-only", "HEAD^"]

    def extract_day(path):
        (dirname, basename) = os.path.split(path)
        if dirname == "advent/src/bin":
            m = standalone_pat.match(basename)
            if m:
                return m.group(1)
        else:
            m = directory_pat.match(dirname)
            if m:
                return m.group(1)

    git_diff = subprocess.run(git_diff_cmd, capture_output=True, text="UTF-8")
    git_diff.check_returncode()
    for path in git_diff.stdout.split('\n'):
        day = extract_day(path)
        if day:
            print("Running solver for {}".format(day), flush=True)
            build_solver(day)
            solve_challenge(day, args)

if "__main__" == __name__:
    parser = argparse.ArgumentParser()
    parser.add_argument("-c", "--challenge",
                        default="/opt/advent-of-code-2020-secret",
                        help="Root directory of challenge files")
    args = parser.parse_args()
    main(args)
