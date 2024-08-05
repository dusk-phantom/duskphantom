pub fn process(input: &str) -> String {
    // Replace `starttime\(\)` with `_sysy_starttime($LN)` if character before is not connected
    let regex = regex::Regex::new(r"([^A-Za-z0-9_]|^)starttime\(\)").unwrap();
    let replaced = regex.replace_all(input, "${1}_sysy_starttime($$LN)");

    // Replace `stoptime\(\)` with `_sysy_stoptime($LN)` if character before is not connected
    let regex = regex::Regex::new(r"([^A-Za-z0-9_]|^)stoptime\(\)").unwrap();
    let replaced = regex.replace_all(&replaced, "${1}_sysy_stoptime($$LN)");

    // Replace `$LN` with real line number
    replaced
        .split('\n')
        .enumerate()
        .map(|(ix, line)| line.replace("$LN", (ix + 1).to_string().as_str()))
        .collect::<Vec<String>>()
        .join("\n")
}

// Unit tests
#[cfg(test)]
pub mod tests_timing {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_timing() {
        let code = r#"
        starttime();
        starttime();
        starttime();x1starttime();starttime();
        stoptime();stoptime();_stoptime();
        "#
        .trim()
        .split('\n')
        .map(|s| s.trim())
        .collect::<Vec<_>>()
        .join("\n");
        assert_snapshot!(process(&code), @r###"
        _sysy_starttime(1);
        _sysy_starttime(2);
        _sysy_starttime(3);x1_sysy_starttime(3);_sysy_starttime(3);
        _sysy_stoptime(4);_sysy_stoptime(4);__sysy_stoptime(4);
        "###);
    }
}
