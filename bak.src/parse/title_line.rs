/// Grammar rule: title_line = whitespace{3,} title whitespace{3,} author whitespace* (newline | EOI)

#[derive(Debug, Clone)]
pub struct TitleLine {
    pub title: String,
    pub author: String,
}

pub fn parse(line: &str) -> Option<TitleLine> {
    if line.len() <= 6 {
        return None; // Too short for meaningful title line
    }

    let leading_spaces = line.len() - line.trim_start().len();
    if leading_spaces < 3 {
        return None; // Need at least 3 leading spaces
    }

    let content = line.trim_start();

    // Look for 3+ spaces within the content
    if let Some(large_gap_start) = find_large_gap(content, 3) {
        let title = content[..large_gap_start].trim().to_string();
        let author = content[large_gap_start..].trim().to_string();

        if !title.is_empty() && !author.is_empty() {
            return Some(TitleLine { title, author });
        }
    }

    None
}

/// Find position of 3+ consecutive spaces
fn find_large_gap(s: &str, min_spaces: usize) -> Option<usize> {
    let chars: Vec<char> = s.chars().collect();
    let mut space_count = 0;
    let mut gap_start = 0;

    for (i, &ch) in chars.iter().enumerate() {
        if ch == ' ' {
            if space_count == 0 {
                gap_start = i;
            }
            space_count += 1;
        } else {
            if space_count >= min_spaces {
                return Some(gap_start);
            }
            space_count = 0;
        }
    }

    if space_count >= min_spaces {
        Some(gap_start)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_title_line() {
        // Valid title line with proper spacing
        let result = parse("        Amazing Grace        Bach");
        assert!(result.is_some());
        let title_line = result.unwrap();
        assert_eq!(title_line.title, "Amazing Grace");
        assert_eq!(title_line.author, "Bach");

        // Valid title line with different spacing
        let result = parse("   Fugue in D minor              Bach");
        assert!(result.is_some());
        let title_line = result.unwrap();
        assert_eq!(title_line.title, "Fugue in D minor");
        assert_eq!(title_line.author, "Bach");

        // Invalid - not enough leading spaces
        let result = parse("  Amazing Grace        Bach");
        assert!(result.is_none());

        // Invalid - not enough spaces between title and author
        let result = parse("        Amazing Grace  Bach");
        assert!(result.is_none());

        // Invalid - too short
        let result = parse("   A   B");
        assert!(result.is_none());
    }
}