/// Extract the value of `attr="..."` from a single tag string.
pub(super) fn extract_attr<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
    let needle = format!("{}=\"", attr);
    let start = tag.find(&needle)? + needle.len();
    let end = start + tag[start..].find('"')?;
    Some(&tag[start..end])
}

/// Get the text between `<tag...>` and `</tag>`. Returns the *first* match.
pub(super) fn extract_tag_content<'a>(html: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start_tag = html.find(&open)?;
    let after_open = start_tag + html[start_tag..].find('>')? + 1;
    let end = html[after_open..].find(&close)?;
    Some(&html[after_open..after_open + end])
}

/// Find all occurrences of `<tag` in html.
/// For self-closing tags (`<img .../>` or `<img ...>`), returns the tag string.
/// For paired tags (`<a ...>...</a>`), returns tag + inner content + close tag.
pub(super) fn find_all_tags<'a>(html: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let mut results = Vec::new();
    let mut search_from = 0;

    while let Some(start) = html[search_from..].find(&open) {
        let abs_start = search_from + start;
        // Find end of opening tag
        let Some(tag_end_rel) = html[abs_start..].find('>') else {
            break;
        };
        let tag_end = abs_start + tag_end_rel;

        // Self-closing?
        if html[abs_start..=tag_end].ends_with("/>") || is_void_tag(tag) {
            results.push(&html[abs_start..=tag_end]);
            search_from = tag_end + 1;
        } else {
            // Find matching close tag
            if let Some(close_rel) = html[tag_end..].find(&close) {
                let abs_end = tag_end + close_rel + close.len();
                results.push(&html[abs_start..abs_end]);
                search_from = abs_end;
            } else {
                // No close tag — treat as self-closing
                results.push(&html[abs_start..=tag_end]);
                search_from = tag_end + 1;
            }
        }
    }

    results
}

fn is_void_tag(tag: &str) -> bool {
    matches!(
        tag,
        "img" | "br" | "hr" | "input" | "meta" | "link" | "source"
    )
}

/// Strip all HTML tags, returning only text content.
pub(super) fn strip_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}
