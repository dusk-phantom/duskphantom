pub fn diff(a: &str, b: &str) -> String {
    diff::lines(a, b)
        .iter()
        .map(|line| match line {
            diff::Result::Left(s) => format!("[-] {}", s),
            diff::Result::Both(s, _) => s.to_string(),
            diff::Result::Right(s) => format!("[+] {}", s),
        })
        .collect::<Vec<String>>()
        .join("\n")
}
