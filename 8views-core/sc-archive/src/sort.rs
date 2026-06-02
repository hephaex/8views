/// Natural sort comparison for filenames (handles numeric suffixes like page001, page002, ...).
/// Mirrors TSSTSortDescriptor logic.
pub fn natural_sort_key(s: &str) -> Vec<NaturalChunk> {
    let mut chunks = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            let num: String =
                std::iter::from_fn(|| chars.next_if(|c| c.is_ascii_digit())).collect();
            chunks.push(NaturalChunk::Num(num.parse::<u64>().unwrap_or(0)));
        } else {
            let text: String =
                std::iter::from_fn(|| chars.next_if(|c| !c.is_ascii_digit())).collect();
            chunks.push(NaturalChunk::Text(text.to_lowercase()));
        }
    }
    chunks
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NaturalChunk {
    Text(String),
    Num(u64),
}

pub fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    natural_sort_key(a).cmp(&natural_sort_key(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_natural_sort_numeric() {
        let mut names = vec!["page10.jpg", "page2.jpg", "page1.jpg", "page20.jpg"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(
            names,
            ["page1.jpg", "page2.jpg", "page10.jpg", "page20.jpg"]
        );
    }

    #[test]
    fn test_natural_sort_mixed() {
        let mut names = vec!["b.jpg", "a.jpg", "c.jpg"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, ["a.jpg", "b.jpg", "c.jpg"]);
    }

    #[test]
    fn test_natural_sort_zero_padded() {
        let mut names = vec!["001.jpg", "010.jpg", "002.jpg"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, ["001.jpg", "002.jpg", "010.jpg"]);
    }
}
