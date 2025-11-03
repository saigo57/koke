
#[cfg(test)]
pub mod tests {
    use regex::Regex;

    pub fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    pub fn minify_html(input: &str) -> String {
        let re = Regex::new(r"\s*\n\s*").unwrap(); // 改行の前後の空白を含めて除去
        re.replace_all(input, "").into_owned()
    }
}
