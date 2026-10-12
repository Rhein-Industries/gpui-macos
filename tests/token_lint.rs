//! Ratchet for native tokens; production src only (plan Verification).
use std::{collections::BTreeMap, fs, path::Path};

const PATTERNS: &[&str] = &[
    "rems(0.8125)",
    "text_size(rems|px(<literal>))",
    "relative(1.",
    "opacity(0.45)",
    // Compatibility dimming for Custom paint; native roles are migrated, so
    // new call sites must not appear (plan D6).
    "CUSTOM_DISABLED_OPACITY",
    "rgb(0x",
    "Hsla::black()/Hsla::white()",
    "hover_fill",
    "pressed_fill",
    "control_fill",
    "is_window_active()",
    "px(<non-zero literal>)",
];

fn sources(dir: &Path, source_root: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, source_root, out);
        } else if is_production_source(&path, source_root) {
            out.push(path);
        }
    }
}

fn is_production_source(path: &Path, source_root: &Path) -> bool {
    path.extension().is_some_and(|ext| ext == "rs")
        && path != source_root.join("tokens.rs")
        && path != source_root.join("state.rs")
}

// Strip comments and quoted strings so examples in documentation aren't counted.
// Whitespace normalization catches multiline calls as well as ordinary calls.
fn strip_comments_and_strings(source: &str) -> String {
    let mut chars = source.chars().peekable();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            out.push(' ');
            for next in chars.by_ref() {
                if next == '\n' {
                    break;
                }
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            out.push(' ');
            chars.next();
            let mut depth = 1;
            while let Some(next) = chars.next() {
                if next == '/' && chars.peek() == Some(&'*') {
                    chars.next();
                    depth += 1;
                } else if next == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
            }
        } else if c == '"' {
            out.push(' ');
            while let Some(next) = chars.next() {
                if next == '\\' {
                    chars.next();
                } else if next == '"' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn code_only(source: &str) -> String {
    strip_comments_and_strings(source)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

// A test statement or impl must not hide later production code. Cut only when
// cfg(test) decorates a module, allowing intervening attributes and visibility.
fn production_code(source: &str) -> String {
    let code = strip_comments_and_strings(source);
    for (start, _) in code.match_indices('#') {
        let mut rest = code[start..].trim_start();
        if !rest.starts_with("#[") {
            continue;
        }
        let Some(end) = rest.find(']') else {
            continue;
        };
        if rest[..=end]
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            != "#[cfg(test)]"
        {
            continue;
        }
        rest = rest[end + 1..].trim_start();
        while rest.starts_with("#[") {
            let Some(end) = rest.find(']') else {
                break;
            };
            rest = rest[end + 1..].trim_start();
        }
        if let Some(after) = rest.strip_prefix("pub")
            && (after.starts_with(char::is_whitespace) || after.starts_with('('))
        {
            rest = after.trim_start();
            if rest.starts_with('(')
                && let Some(end) = rest.find(')')
            {
                rest = rest[end + 1..].trim_start();
            }
        }
        if rest
            .strip_prefix("mod")
            .is_some_and(|after| after.starts_with(char::is_whitespace))
        {
            return code[..start]
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
        }
    }
    code.chars().filter(|c| !c.is_whitespace()).collect()
}

fn literal_calls(code: &str, function: &str, nonzero: bool) -> usize {
    code.match_indices(function)
        .filter(|(start, _)| {
            // Avoid counting suffixes of unrelated functions (e.g. label_text_size).
            if *start > 0 && code.as_bytes()[start - 1].is_ascii_alphanumeric() {
                return false;
            }
            let arg = &code[start + function.len()..];
            let Some(end) = arg.find(')') else {
                return false;
            };
            let arg = &arg[..end];
            let number = arg.trim_end_matches("f32").replace('_', "");
            number
                .parse::<f64>()
                .is_ok_and(|value| !nonzero || value != 0.)
        })
        .count()
}

fn counts(code: &str) -> BTreeMap<String, usize> {
    let mut result: BTreeMap<_, _> = PATTERNS.iter().map(|p| (p.to_string(), 0)).collect();
    for pattern in PATTERNS {
        let count = match *pattern {
            "text_size(rems|px(<literal>))" => {
                literal_calls(code, "text_size(rems(", false)
                    + literal_calls(code, "text_size(px(", false)
            }
            "px(<non-zero literal>)" => literal_calls(code, "px(", true),
            "Hsla::black()/Hsla::white()" => {
                code.matches("Hsla::black()").count() + code.matches("Hsla::white()").count()
            }
            p => code.matches(p).count(),
        };
        result.insert(pattern.to_string(), count);
    }
    result
}

#[test]
fn production_token_literals_do_not_increase() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut paths = Vec::new();
    sources(&root.join("src"), &root.join("src"), &mut paths);
    paths.sort();
    let mut actual = counts("");
    for path in paths {
        let source = fs::read_to_string(&path).unwrap();
        let mut code = production_code(&source);
        if path == root.join("src/lib.rs") {
            // Native window-level geometry is intentionally physical pixels.
            // Keep this allowlist narrow: traffic lights and minimum window size.
            code = code
                .replace("traffic_light_position:Some(point(px(16.),px(16.))),", "")
                .replace("window_min_size:Some(size(px(640.),px(480.))),", "");
        }
        for (pattern, value) in counts(&code) {
            *actual.get_mut(&pattern).unwrap() += value;
        }
    }
    let baseline: BTreeMap<String, usize> =
        serde_json::from_str(include_str!("token-lint-baseline.json")).unwrap();
    assert_eq!(
        actual.keys().collect::<Vec<_>>(),
        baseline.keys().collect::<Vec<_>>()
    );
    let increases: Vec<_> = actual.iter().filter(|(p, n)| **n > baseline[*p]).collect();
    let numbers = serde_json::to_string_pretty(&actual).unwrap();
    assert!(
        increases.is_empty(),
        "Token counts increased: {increases:?}\nNew counts:\n{numbers}"
    );
    if actual != baseline {
        println!("Token counts decreased; lock in this baseline:\n{numbers}");
    }
    println!("Token lint counts:\n{numbers}");
}

#[test]
fn scanner_counts_literals_and_ignores_comments_and_strings() {
    let code = code_only(
        r#"
        // .text_size(rems(13.))
        let doc = "rgb(0xff)";
        /* px(100.) /* nested */ */
        div().text_size( rems(0.8125) ).line_height(relative(1.2))
            .w(px(0.)).h(px(2.)).bg(Hsla::black()).hover_fill();
    "#,
    );
    let result = counts(&code);
    assert_eq!(result["rems(0.8125)"], 1);
    assert_eq!(result["text_size(rems|px(<literal>))"], 1);
    assert_eq!(result["relative(1."], 1);
    assert_eq!(result["px(<non-zero literal>)"], 1);
    assert_eq!(result["rgb(0x"], 0);
    assert_eq!(result["Hsla::black()/Hsla::white()"], 1);
    assert_eq!(result["hover_fill"], 1);
}

#[test]
fn scanner_cuts_only_at_test_modules_and_skips_only_root_token_files() {
    let code = production_code(
        r#"
        #[cfg(test)] record_debug();
        #[cfg(test)] impl DebugHook { fn hook() {} }
        div().bg(rgb(0x123456));
        #[cfg(test)] module_helper();
        #[cfg(test)] #[allow(dead_code)] pub(crate) mod tests {
            div().bg(rgb(0xffffff));
        }
    "#,
    );
    assert_eq!(counts(&code)["rgb(0x"], 1);
    assert!(code.contains("module_helper"));
    assert_eq!(
        counts(&production_code(
            "#[cfg(test)] mod tests { rgb(0xffffff); }"
        ))["rgb(0x"],
        0
    );
    let root = Path::new("src");
    assert!(!is_production_source(Path::new("src/tokens.rs"), root));
    assert!(!is_production_source(Path::new("src/state.rs"), root));
    assert!(is_production_source(
        Path::new("src/nested/tokens.rs"),
        root
    ));
    assert!(is_production_source(Path::new("src/nested/state.rs"), root));
}
