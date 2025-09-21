/// Grammar rule: title_line = title whitespace{4,} author whitespace* (newline | EOI)
/// When 4+ spaces are found in a line, split into title and author

#[derive(Debug, Clone)]
pub struct TitleLine {
    pub title: String,
    pub author: String,
}

pub fn parse(line: &str) -> Option<TitleLine> {
    // Look for 4+ spaces within the line content to split into title and author
    if let Some(large_gap_start) = find_large_gap(line, 4) {
        let title = line[..large_gap_start].trim().to_string();
        let author = line[large_gap_start..].trim().to_string();

        if !title.is_empty() && !author.is_empty() {
            return Some(TitleLine { title, author });
        }
    }

    None
}

/// Find position of min_spaces or more consecutive spaces
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
        // Valid title line with 4+ spaces
        let result = parse("Amazing Grace    Bach");
        assert!(result.is_some());
        let title_line = result.unwrap();
        assert_eq!(title_line.title, "Amazing Grace");
        assert_eq!(title_line.author, "Bach");

        // Valid title line with many spaces
        let result = parse("Fugue in D minor              Bach");
        assert!(result.is_some());
        let title_line = result.unwrap();
        assert_eq!(title_line.title, "Fugue in D minor");
        assert_eq!(title_line.author, "Bach");

        // Valid with leading spaces (they're trimmed from title)
        let result = parse("   Amazing Grace    Bach");
        assert!(result.is_some());
        let title_line = result.unwrap();
        assert_eq!(title_line.title, "Amazing Grace");
        assert_eq!(title_line.author, "Bach");

        // Test case from user: "Hello            john"
        let result = parse("Hello            john");
        assert!(result.is_some());
        let title_line = result.unwrap();
        assert_eq!(title_line.title, "Hello");
        assert_eq!(title_line.author, "john");

        // Invalid - only 3 spaces between title and author
        let result = parse("Amazing Grace   Bach");
        assert!(result.is_none());

        // Invalid - only 2 spaces
        let result = parse("Amazing Grace  Bach");
        assert!(result.is_none());
    }
}