#!/bin/bash

# Get the latest commit message
commit_message=$(git log -1 --pretty=%B HEAD)

# Check if the commit message starts with "fmt:"
if [[ $commit_message == fmt:* ]]; then
  # Get the latest commit hash
  commit_hash=$(git log -1 --pretty=format:%H)

  # Add the commit message as a comment and the commit hash to .git-blame-ignore-revs
  echo "# $commit_message" >> .git-blame-ignore-revs
  echo $commit_hash >> .git-blame-ignore-revs

  # Add .git-blame-ignore-revs to staging
  git add .git-blame-ignore-revs

  # Commit the change
  git commit -m "Add commit hash $commit_hash to .git-blame-ignore-revs with commit message"

  # Setup ignoreRevsfile
  git config --local blame.ignoreRevsFile .git-blame-ignore-revs
fi
