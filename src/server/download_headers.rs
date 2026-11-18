/// Builds the Content-Disposition value so the browser saves the file with its name.
pub fn attachment_value(file_name: &str) -> String {
    format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        ascii_fallback(file_name),
        percent_encode(file_name)
    )
}

fn ascii_fallback(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || " ._-()".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn percent_encode(name: &str) -> String {
    let mut result = String::new();
    for byte in name.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            result.push(byte as char);
        } else {
            result.push_str(&format!("%{byte:02X}"));
        }
    }
    result
}
