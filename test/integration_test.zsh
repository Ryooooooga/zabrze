#!/usr/bin/env zsh
result=0

try() {
    local lbuffer="$1"
    local rbuffer="$2"
    local expected_name="$3"
    local expected_lbuffer="$4"
    local expected_rbuffer="$5"
    local expected_placeholder="$6"

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

    if [ "$name" != "$expected_name" ]; then
        echo "name not matched (expected: '$expected_name', actual: '$name')"
        result=1
    fi

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

export ZABRZE_CONFIG_HOME="${0:a:h}"
export ZABRZE_TEST=1
export EDITOR=vim

#   lbuffer         rbuffer     name    LBUFFER                         RBUFFER         placeholder
try "g"             ""          "git"   "git"                           ""              ""
try "  g"           ""          "git"   "  git"                         ""              ""
try "g"             "add"       "git"   "git"                           "add"           ""
try "g"             " add"      "git"   "git"                           " add"          ""
try "echo g"        ""          ""      "echo g"                        ""              ""
try "echo a; g"     ""          "git"   "echo a; git"                   ""              ""
try "cat a | .1"    ""          ".1"    "cat a | awk '{ print \$1 }'"   ""              ""
try "view"          "a.txt"     "view"  "vim -R"                        "a.txt"         ""
try "echo ANSWER"   ""          "42"    "echo answer is 42"             ""              ""
try "ANSWER"        ""          "42"    "answer is 42"                  ""              ""
try "git aa"        ""          ""      "git add -vA"                   ""              ""
try "echo git aa"   ""          ""      "echo git aa"                   ""              ""
try "git -f"        ""          ""      "git -f"                        ""              ""
try "git push -f"   ""          ""      "git push --force-with-lease"   ""              ""
try "git cm"        ""          ""      "git commit -m '"               "'"             "1"
try "git cm"        " -v"       ""      "git commit -m '"               "' -v"          "1"
try "apt install"   "zsh"       ""      "sudo apt install -y"           "zsh"           ""
try "["             ""          ""      "[ "                            " ]"            "1"
try "[["            ""          ""      "[[ "                           " ]]"           "1"
try "xargsi"        ""          ""      "xargs -I{} "                   ""              ""
try ".."            ""          ""      "cd .."                         ""              ""
try "../.."         ""          ""      "cd ../.."                      ""              ""
try "../../.."      ""          ""      "cd ../../.."                   ""              ""
try "; placeholder" ""          ""      "; ab"                          "cd placeholder" "1"
try "yes | ./a.ts"  ""          ""      "yes | deno run ./a.ts"         ""              ""
try "yes | ./ab.ts" ""          ""      "yes | deno run ./ab.ts"        ""              ""
try "cond"          ""          ""      "conditional abbrev"            ""              ""
try "cond2"         ""          ""      "cond2"                         ""              ""
try "cond3"         ""          ""      "conditional fallback"          ""              ""
try "2"             ""          ""      "otherfile"                     ""              ""

if [ "$result" -ne 0 ]; then
    echo "test failed!!" >/dev/stderr
    exit 1
fi

echo "test passed!!"
