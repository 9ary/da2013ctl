#!/bin/bash

REV=$(git rev-list HEAD --count)
COMMIT=$(git describe --always)
BRANCH=$(git rev-parse --abbrev-ref HEAD)

source=$(cat <<EOF
const char git_rev[] = "$REV";
const char git_commit[] = "$COMMIT";
const char git_branch[] = "$BRANCH";
const char git_version[] = PROJ_NAME " r$REV ($BRANCH-$COMMIT)";
EOF
)

diff $1 <(echo "$source") > /dev/null 2>&1
if [[ $? -ne 0 ]]
then
    echo "  VERSION"
    echo "$source" > $1
fi
