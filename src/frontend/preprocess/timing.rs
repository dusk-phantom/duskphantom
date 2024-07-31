pub fn process(input: &str) -> String {
    // Replace `(\W)starttime\(\)` with `$1_sysy_starttime($LN)`
    let regex = regex::Regex::new(r"(\W)starttime\(\)").unwrap();
    let replaced = regex.replace_all(input, "${1}_sysy_starttime($$LN)");

    // Replace `(\W)stoptime\(\)` with `$1_sysy_stoptime($LN)`
    let regex = regex::Regex::new(r"(\W)stoptime\(\)").unwrap();
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
        int main() {
            starttime();
            starttime();
            starttime();x1starttime();starttime();
            stoptime();stoptime();_stoptime();
        }
        "#;
        assert_snapshot!(process(code), @r###"

                int main() {
                    _sysy_starttime(3);
                    _sysy_starttime(4);
                    _sysy_starttime(5);x1starttime();_sysy_starttime(5);
                    _sysy_stoptime(6);_sysy_stoptime(6);_stoptime();
                }
                
        "###);
    }
}
