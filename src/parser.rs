use chrono::{NaiveDate, NaiveDateTime};
use regex::Regex;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct ParsedTask {
    pub title: String,
    pub priority: Option<u8>,
    pub due_date: Option<NaiveDateTime>,
}

#[derive(Debug, Error, Clone)]
pub enum ParseError {
    #[error("Invalid due date format: {0}")]
    InvalidDueDate(String),
    #[error("Invalid priority format: {0}")]
    InvalidPriority(String),
}

pub fn parse_task_input(input: &str) -> Result<ParsedTask, ParseError> {
    let mut title = input.to_string();

    // Regex for priority based on '!' followed by any number
    let priority_re = Regex::new(r"!\s*(\d+)").unwrap();
    let priority = if let Some(caps) = priority_re.captures(&title) {
        let prio_str = caps.get(1).unwrap().as_str();
        match prio_str.parse::<u8>() {
            Ok(p) if (1..=5).contains(&p) => Some(p),
            Ok(_) => {
                return Err(ParseError::InvalidPriority(prio_str.to_string()));
            }
            Err(_) => {
                return Err(ParseError::InvalidPriority(prio_str.to_string()));
            }
        }
    } else {
        None
    };

    // Remove priority from title
    title = priority_re.replace_all(&title, "").into_owned();

    // Regex for due date (e.g., "due:2023-12-31")
    let due_date_re = Regex::new(r"\b(?:due):\s*(\d{4}-\d{2}-\d{2})\b").unwrap();
    let due_date = if let Some(caps) = due_date_re.captures(&title) {
        let date_str = caps.get(1).unwrap().as_str();
        match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(date) => {
                // Append default time component
                let datetime = date.and_hms(23, 59, 59);
                Some(datetime)
            }
            Err(_) => {
                return Err(ParseError::InvalidDueDate(date_str.to_string()));
            }
        }
    } else {
        None
    };

    // Remove due date from title
    title = due_date_re.replace_all(&title, "").into_owned();

    // Trim whitespace from title
    title = title.trim().to_string();

    Ok(ParsedTask {
        title,
        priority,
        due_date,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveDateTime};

    #[test]
    fn test_parse_with_priority_only() {
        let input = "Finish the report !3";
        let parsed = parse_task_input(input).unwrap();
        assert_eq!(parsed.title, "Finish the report");
        assert_eq!(parsed.priority, Some(3));
        assert_eq!(parsed.due_date, None);
    }

    #[test]
    fn test_parse_with_due_date_and_priority() {
        let input = "Finish the report due:2023-12-31 !4";
        let parsed = parse_task_input(input).unwrap();
        assert_eq!(parsed.title, "Finish the report");
        let expected_datetime = NaiveDate::from_ymd(2023, 12, 31).and_hms(23, 59, 59);
        assert_eq!(parsed.due_date, Some(expected_datetime));
        assert_eq!(parsed.priority, Some(4));
    }

    #[test]
    fn test_parse_with_no_priority() {
        let input = "Finish the report";
        let parsed = parse_task_input(input).unwrap();
        assert_eq!(parsed.title, "Finish the report");
        assert_eq!(parsed.priority, None);
        assert_eq!(parsed.due_date, None);
    }

    #[test]
    fn test_parse_with_invalid_priority() {
        let input = "Finish the report !6";
        let parsed = parse_task_input(input);
        assert!(parsed.is_err());
        if let Err(ParseError::InvalidPriority(prio_str)) = parsed {
            assert_eq!(prio_str, "6");
        } else {
            panic!("Expected InvalidPriority error.");
        }
    }

    #[test]
    fn test_parse_with_extra_whitespace() {
        let input = "  Finish the report   ! 2   ";
        let parsed = parse_task_input(input).unwrap();
        assert_eq!(parsed.title, "Finish the report");
        assert_eq!(parsed.priority, Some(2));
        assert_eq!(parsed.due_date, None);
    }

    #[test]
    fn test_parse_with_priority_at_start() {
        let input = "!5 Finish the report";
        let parsed = parse_task_input(input).unwrap();
        assert_eq!(parsed.title, "Finish the report");
        assert_eq!(parsed.priority, Some(5));
        assert_eq!(parsed.due_date, None);
    }

    #[test]
    fn test_parse_with_invalid_due_date() {
        let input = "Finish the report due:2023-13-31";
        let parsed = parse_task_input(input);
        assert!(parsed.is_err());
        if let Err(ParseError::InvalidDueDate(date_str)) = parsed {
            assert_eq!(date_str, "2023-13-31");
        } else {
            panic!("Expected InvalidDueDate error.");
        }
    }
}
