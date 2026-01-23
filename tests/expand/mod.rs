use std::path::Path;

use crate::{cli, zsh};

#[derive(Debug)]
enum TestResult<'a> {
    Unmatched,
    Matched {
        lbuffer: &'a str,
        rbuffer: &'a str,
        placeholder: &'a str,
    },
}

fn run_test(config_dirname: &str, (lbuffer, rbuffer): (&str, &str), expected: TestResult<'_>) {
    let config_dir = Path::new(file!())
        .parent()
        .unwrap()
        .join("testdata")
        .join(config_dirname);

    let output = cli()
        .args(&["expand", "--lbuffer", lbuffer, "--rbuffer", rbuffer])
        .env("ZABRZE_CONFIG_HOME", config_dir)
        .output()
        .expect("Failed to execute expand");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(stderr, "");
    assert!(output.status.success());

    let (expected_lbuffer, expected_rbuffer, expected_placeholder) = match expected {
        TestResult::Unmatched => {
            assert_eq!(stdout, "");
            (lbuffer, rbuffer, "")
        }
        TestResult::Matched {
            lbuffer,
            rbuffer,
            placeholder,
        } => (lbuffer, rbuffer, placeholder),
    };

    let cmd = format!(
        r#"
        {stdout}

        cat <<EOF
LBUFFER=$LBUFFER
RBUFFER=$RBUFFER
__zabrze_has_placeholder=$__zabrze_has_placeholder
EOF
        "#
    );

    let result = zsh()
        .arg("-c")
        .arg(cmd)
        .envs([("LBUFFER", lbuffer), ("RBUFFER", rbuffer)])
        .env("EDITOR", "vim")
        .env("ZABRZE_TEST", "1")
        .output()
        .expect("Failed to execute zsh with expand output");
    let result_stdout = String::from_utf8_lossy(&result.stdout);
    let result_stderr = String::from_utf8_lossy(&result.stderr);

    assert!(result.status.success());

    let expected_output = format!(
        "LBUFFER={expected_lbuffer}\nRBUFFER={expected_rbuffer}\n__zabrze_has_placeholder={expected_placeholder}\n"
    );
    assert_eq!(result_stdout, expected_output);

    if !result_stderr.is_empty() {
        insta::assert_snapshot!(
            format!("{config_dirname}-{lbuffer}%{rbuffer}"),
            result_stderr
        );
    }
}

#[test]
fn test_empty() {
    let config_dirname = "empty";
    run_test(config_dirname, ("", ""), TestResult::Unmatched);
    run_test(config_dirname, ("g", ""), TestResult::Unmatched);
}

#[test]
fn test_basic_toml() {
    let config_dirname = "basic_toml";
    run_test(config_dirname, ("", ""), TestResult::Unmatched);
    run_test(config_dirname, (" ", ""), TestResult::Unmatched);
    run_test(
        config_dirname,
        ("g", ""),
        TestResult::Matched {
            lbuffer: "git",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("  g", ""),
        TestResult::Matched {
            lbuffer: "  git",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("g", "add"),
        TestResult::Matched {
            lbuffer: "git",
            rbuffer: "add",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("g", " add"),
        TestResult::Matched {
            lbuffer: "git",
            rbuffer: " add",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("echo g", ""),
        TestResult::Matched {
            lbuffer: "echo g",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("echo a; g", ""),
        TestResult::Matched {
            lbuffer: "echo a; git",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("seq 10 | .1", ""),
        TestResult::Matched {
            lbuffer: "seq 10 | awk '{print $1}'",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("true && g", ""),
        TestResult::Matched {
            lbuffer: "true && git",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("false || g", ""),
        TestResult::Matched {
            lbuffer: "false || git",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("false || echo g", ""),
        TestResult::Unmatched,
    );
    run_test(config_dirname, ("G", ""), TestResult::Unmatched);
    run_test(config_dirname, ("gg", ""), TestResult::Unmatched);
}

#[test]
fn test_global() {
    let config_dirname = "global";
    run_test(
        config_dirname,
        ("null", ""),
        TestResult::Matched {
            lbuffer: ">/dev/null 2>&1",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("echo a null", ""),
        TestResult::Matched {
            lbuffer: "echo a >/dev/null 2>&1",
            rbuffer: "",
            placeholder: "",
        },
    );
}

#[test]
fn test_context() {
    let config_dirname = "context";
    run_test(
        config_dirname,
        ("git c", ""),
        TestResult::Matched {
            lbuffer: "git commit",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("echo ;git c", ""),
        TestResult::Matched {
            lbuffer: "echo ;git commit",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(config_dirname, ("echo git c", ""), TestResult::Unmatched);
    run_test(
        config_dirname,
        ("git cm", ""),
        TestResult::Matched {
            lbuffer: "git commit -m '",
            rbuffer: "'",
            placeholder: "1",
        },
    );
    run_test(config_dirname, ("echo git cm", ""), TestResult::Unmatched);
    run_test(
        config_dirname,
        ("git aa", ""),
        TestResult::Matched {
            lbuffer: "git add -vA",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(config_dirname, ("echo git aa", ""), TestResult::Unmatched);
    run_test(
        config_dirname,
        ("git push -f", ""),
        TestResult::Matched {
            lbuffer: "git push --force-with-lease",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(config_dirname, ("git -f", ""), TestResult::Unmatched);
    run_test(config_dirname, ("git add -f", ""), TestResult::Unmatched);
    run_test(
        config_dirname,
        ("echo git push -f", ""),
        TestResult::Unmatched,
    );
}

#[test]
fn test_evaluate() {
    let config_dirname = "evaluate";
    run_test(
        config_dirname,
        ("view", ""),
        TestResult::Matched {
            lbuffer: "vim -R",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(config_dirname, ("echo view", ""), TestResult::Unmatched);
    run_test(
        config_dirname,
        ("ANSWER", ""),
        TestResult::Matched {
            lbuffer: "The answer is 42",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("echo ANSWER", ""),
        TestResult::Matched {
            lbuffer: "echo The answer is 42",
            rbuffer: "",
            placeholder: "",
        },
    );
}

#[test]
fn test_placeholder() {
    let config_dirname = "placeholder";
    run_test(
        config_dirname,
        ("[", ""),
        TestResult::Matched {
            lbuffer: "[ ",
            rbuffer: " ]",
            placeholder: "1",
        },
    );
    run_test(
        config_dirname,
        ("[[", ""),
        TestResult::Matched {
            lbuffer: "[[ ",
            rbuffer: " ]]",
            placeholder: "1",
        },
    );
    run_test(
        config_dirname,
        ("xargsi", ""),
        TestResult::Matched {
            lbuffer: "xargs -I{} ",
            rbuffer: "",
            placeholder: "",
        },
    );
}

#[test]
fn test_trigger_pattern() {
    let config_dirname = "trigger_pattern";
    run_test(
        config_dirname,
        ("..", ""),
        TestResult::Matched {
            lbuffer: "cd ..",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("../..", ""),
        TestResult::Matched {
            lbuffer: "cd ../..",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("../../..", ""),
        TestResult::Matched {
            lbuffer: "cd ../../..",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(config_dirname, ("..a", ""), TestResult::Unmatched);
    run_test(
        config_dirname,
        ("yes | ./a.ts", ""),
        TestResult::Matched {
            lbuffer: "yes | deno run ./a.ts",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("yes | ./ab.ts", ""),
        TestResult::Matched {
            lbuffer: "yes | deno run ./ab.ts",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("cat a | .1", ""),
        TestResult::Matched {
            lbuffer: "cat a | awk '{ print $1 }'",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("cat a | .2", ""),
        TestResult::Matched {
            lbuffer: "cat a | awk '{ print $2 }'",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("cat a | .123", ""),
        TestResult::Matched {
            lbuffer: "cat a | awk '{ print $123 }'",
            rbuffer: "",
            placeholder: "",
        },
    );
}

#[test]
fn test_conditional() {
    let config_dirname = "conditional";
    run_test(
        config_dirname,
        ("cond1", ""),
        TestResult::Matched {
            lbuffer: "TRUE",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("cond2", ""),
        TestResult::Matched {
            lbuffer: "cond2",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("cond3", ""),
        TestResult::Matched {
            lbuffer: "FALLBACK",
            rbuffer: "",
            placeholder: "",
        },
    );
}

#[test]
fn test_action() {
    let config_dirname = "action";
    run_test(
        config_dirname,
        ("apt i", ""),
        TestResult::Matched {
            lbuffer: "sudo apt install -y",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("npm i", ""),
        TestResult::Matched {
            lbuffer: "npm install",
            rbuffer: "",
            placeholder: "",
        },
    );
}

#[test]
fn test_capture() {
    let config_dirname = "capture";
    run_test(
        config_dirname,
        (".1", ""),
        TestResult::Matched {
            lbuffer: "awk '{ print $1 }'",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        (".99", ""),
        TestResult::Matched {
            lbuffer: "awk '{ print $99 }'",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("./main.py", ""),
        TestResult::Matched {
            lbuffer: "python3 ./main.py",
            rbuffer: "",
            placeholder: "",
        },
    );
}

#[test]
fn test_multi_files() {
    let config_dirname = "multi_files";
    run_test(
        config_dirname,
        ("A", ""),
        TestResult::Matched {
            lbuffer: "apple",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("B", ""),
        TestResult::Matched {
            lbuffer: "banana",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("Y", ""),
        TestResult::Matched {
            lbuffer: "yogurt",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("Z", ""),
        TestResult::Matched {
            lbuffer: "zinc",
            rbuffer: "",
            placeholder: "",
        },
    );
}

#[test]
fn test_abort_on_error() {
    let config_dirname = "abort_on_error";
    run_test(
        config_dirname,
        ("simple", ""),
        TestResult::Matched {
            lbuffer: "SUCCESS",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("success", ""),
        TestResult::Matched {
            lbuffer: "SUCCESS",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("fail", ""),
        TestResult::Matched {
            lbuffer: "fail",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("fail_success", ""),
        TestResult::Matched {
            lbuffer: "FAIL_SUCCESS",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("unknown", ""),
        TestResult::Matched {
            lbuffer: "unknown",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("no_abort", ""),
        TestResult::Matched {
            lbuffer: "FAIL",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("placeholder_success_success", ""),
        TestResult::Matched {
            lbuffer: "SUCCESS",
            rbuffer: "SUCCESS",
            placeholder: "1",
        },
    );
    run_test(
        config_dirname,
        ("placeholder_fail_success", ""),
        TestResult::Matched {
            lbuffer: "placeholder_fail_success",
            rbuffer: "",
            placeholder: "",
        },
    );
    run_test(
        config_dirname,
        ("placeholder_success_fail", ""),
        TestResult::Matched {
            lbuffer: "placeholder_success_fail",
            rbuffer: "",
            placeholder: "",
        },
    );
}
