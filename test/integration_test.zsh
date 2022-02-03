#!/usr/bin/env zsh
result=0

try() {
    local lbuffer="$1"
    local rbuffer="$2"
    local expected_lbuffer="$3"
    local expected_rbuffer="$4"
    local expected_placeholder="$5"

    echo "try $lbuffer:$rbuffer"

    local out exit_code
    out="$(zabrze expand --lbuffer="$lbuffer" --rbuffer="$rbuffer")"
    exit_code="$?"
    if [ "$exit_code" -ne 0 ]; then
        echo "  zabrze expand failed with status $exit_code" >/dev/stderr
        result=1
        return
    fi

    local LBUFFER RBUFFER __zabrze_has_placeholder
    LBUFFER="$lbuffer"
    RBUFFER="$rbuffer"
    eval "$out"

    if [ "$LBUFFER" != "$expected_lbuffer" ]; then
        echo "LBUFFER not matched (expected: '$expected_lbuffer', actual: '$LBUFFER')"
        result=1
    fi

    if [ "$RBUFFER" != "$expected_rbuffer" ]; then
        echo "RBUFFER not matched (expected: '$expected_rbuffer', actual: '$RBUFFER')"
        result=1
    fi

    if [ "$__zabrze_has_placeholder" != "$expected_placeholder" ]; then
        echo "__zabrze_has_placeholder not matched (expected: '$expected_placeholder', actual: '$__zabrze_has_placeholder')"
        result=1
    fi
}

export ZABRZE_CONFIG_FILE="${0:a:h}/config.yaml"
export EDITOR=vim

#   lbuffer         rbuffer     LBUFFER                         RBUFFER         placeholder
try "g"             ""          "git"                           ""              ""
try "  g"           ""          "  git"                         ""              ""
try "g"             "add"       "git"                           "add"           ""
try "g"             " add"      "git"                           " add"          ""
try "echo g"        ""          "echo g"                        ""              ""
try "echo a; g"     ""          "echo a; git"                   ""              ""
try "cat a | .1"    ""          "cat a | awk '{ print \$1 }'"   ""              ""
try "view"          "a.txt"     "vim -R"                        "a.txt"         ""
try "echo ANSWER"   ""          "echo answer is 42"             ""              ""
try "ANSWER"        ""          "answer is 42"                  ""              ""
try "git aa"        ""          "git add -vA"                   ""              ""
try "echo git aa"   ""          "echo git aa"                   ""              ""
try "git -f"        ""          "git -f"                        ""              ""
try "git push -f"   ""          "git push --force-with-lease"   ""              ""
try "git cm"        ""          "git commit -m '"               "'"             "1"
try "git cm"        " -v"       "git commit -m '"               "' -v"          "1"
try "apt install"   "zsh"       "sudo apt install -y"           "zsh"           ""
try "["             ""          "[ "                            " ]"            "1"
try ".."            ""          "cd .."                         ""              ""
try "../.."         ""          "cd ../.."                      ""              ""
try "../../.."      ""          "cd ../../.."                   ""              ""
try "; placeholder" ""          "; ab"                          "cd placeholder" "1"
try "yes | ./a.ts"  ""          "yes | deno run ./a.ts"         ""              ""
try "yes | ./ab.ts" ""          "yes | deno run ./ab.ts"        ""              ""

if [ "$result" -ne 0 ]; then
    echo "test failed!!" >/dev/stderr
    exit 1
fi

echo "test passed!!"
