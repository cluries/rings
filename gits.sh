#!/usr/bin/env bash


commit_all() {
    git add -A
    summary=$(git status -s)
    git commit -m "Changes: $summary"
    git push origin master
}

commit_all