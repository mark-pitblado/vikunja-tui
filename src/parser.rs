use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct ParsedTask {
    pub title: String,
    pub priority: Option<u8>,
}

pub fn parse_task_input(input: &str) -> ParsedTask {
    let priority_re = Regex::new(r"!(\d+)\s*").unwrap();

    let mut priority = None;

    // Priority
    for caps in priority_re.captures_iter(input) {
        if let Some(priority_match) = caps.get(1) {
            if let Ok(p) = priority_match.as_str().parse::<u8>() {
                if (1..=5).contains(&p) && priority.is_none() {
                    priority = Some(p);
                }
            }
        }
    }

    let title = priority_re.replace_all(&input, "").to_string();

    let title = Regex::new(r"\s+")
        .unwrap()
        .replace_all(&title, " ")
        .trim()
        .to_string();

    ParsedTask { title, priority }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_priority_in_middle() {
        let input = "Update !4 software documentation";
        let expected = ParsedTask {
            title: "Update software documentation".to_string(),
            priority: Some(4),
        };
        let result = parse_task_input(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_with_extra_spaces_after_priority() {
        let input = "Fix bugs !2    in the code";
        let expected = ParsedTask {
            title: "Fix bugs in the code".to_string(),
            priority: Some(2),
        };
        let result = parse_task_input(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_with_multiple_spaces_between_words() {
        let input = "Write   tests !3 for the   parser";
        let expected = ParsedTask {
            title: "Write tests for the parser".to_string(),
            priority: Some(3),
        };
        let result = parse_task_input(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_with_priority_at_end_and_extra_spaces() {
        let input = "Deploy to production   !5   ";
        let expected = ParsedTask {
            title: "Deploy to production".to_string(),
            priority: Some(5),
        };
        let result = parse_task_input(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_with_priority_at_start_no_space() {
        let input = "!2Prepare presentation slides";
        let expected = ParsedTask {
            title: "Prepare presentation slides".to_string(),
            priority: Some(2),
        };
        let result = parse_task_input(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_with_multiple_priorities_and_spaces() {
        let input = "  !1  !2 Organize    team building !3 event ";
        let expected = ParsedTask {
            title: "Organize team building event".to_string(),
            priority: Some(1),
        };
        let result = parse_task_input(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_with_invalid_priority_and_spaces() {
        let input = "Check logs !8    immediately";
        let expected = ParsedTask {
            title: "Check logs immediately".to_string(),
            priority: None,
        };
        let result = parse_task_input(input);
        assert_eq!(result, expected);
    }
}
