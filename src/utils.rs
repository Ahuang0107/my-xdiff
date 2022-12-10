struct Line(Option<usize>);

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

pub fn diff_text(t1: &str, t2: &str) -> anyhow::Result<String> {
    let mut output = String::new();
    let diff = similar::TextDiff::from_lines(t1, t2);
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            output.push_str(&format!("{:-^1$}\n", "-", 80));
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sing, s) = match change.tag() {
                    similar::ChangeTag::Delete => ("-", console::Style::new().red()),
                    similar::ChangeTag::Insert => ("+", console::Style::new().green()),
                    similar::ChangeTag::Equal => (" ", console::Style::new().dim()),
                };
                output.push_str(&format!(
                    "{}{} |{}",
                    console::style(Line(change.old_index())).dim(),
                    console::style(Line(change.new_index())).dim(),
                    s.apply_to(sing).bold()
                ));
                for (emphasized, value) in change.iter_strings_lossy() {
                    if emphasized {
                        output.push_str(&format!("{}", s.apply_to(value).underlined().on_black()));
                    } else {
                        output.push_str(&format!("{}", s.apply_to(value)));
                    }
                }
                if change.missing_newline() {
                    output.push_str("\n")
                }
            }
        }
    }
    Ok(output)
}
