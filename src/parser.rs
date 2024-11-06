use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct ParsedTask {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<u8>,
}

pub fn parse_task_input(input: &str) -> ParsedTask {
    let priority_re = Regex::new(r"!(\d+)\s*").unwrap();
    let description_re = Regex::new(r"\{([^}]*)\}").unwrap();

    let mut priority = None;
    let mut description = None;

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

    // Description
    let mut title = input.to_string();
    if let Some(caps) = description_re.captures(&title) {
        if let Some(desc_match) = caps.get(1) {
            description = Some(desc_match.as_str().trim().to_string());
        }
    }

    title = description_re.replace_all(&title, "").to_string();

    title = priority_re.replace_all(&title, "").to_string();

    let title = Regex::new(r"\s+")
        .unwrap()
        .replace_all(&title, " ")
        .trim()
        .to_string();

    ParsedTask {
        title,
        priority,
        description,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_description() {
        let input = "Implement feature X {This is the description of the task}";
        let expected = ParsedTask {
            title: "Implement feature X".to_string(),
            description: Some("This is the description of the task".to_string()),
            priority: None,
        };
        let result = parse_task_input(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_with_description_and_priority() {
        let input = "Fix bug in module !2 {Critical issue that needs immediate attention}";
        let expected = ParsedTask {
            title: "Fix bug in module".to_string(),
            description: Some("Critical issue that needs immediate attention".to_string()),
            priority: Some(2),
        };
        let result = parse_task_input(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_with_description_and_priority_in_any_order() {
        let input = "{Detailed description here} Update documentation !3";
        let expected = ParsedTask {
            title: "Update documentation".to_string(),
            description: Some("Detailed description here".to_string()),
            priority: Some(3),
        };
        let result = parse_task_input(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_with_multiple_descriptions() {
        let input = "Task title {First description} {Second description}";
        let expected = ParsedTask {
            title: "Task title".to_string(),
            description: Some("First description".to_string()),
            priority: None,
        };
        let result = parse_task_input(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_with_priority_in_middle() {
        let input = "Update !4 software documentation";
        let expected = ParsedTask {
            title: "Update software documentation".to_string(),
            priority: Some(4),
            description: None,
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
            description: None,
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
            description: None,
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
            description: None,
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
            description: None,
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
            description: None,
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
            description: None,
        };
        let result = parse_task_input(input);
        assert_eq!(result, expected);
    }
}
