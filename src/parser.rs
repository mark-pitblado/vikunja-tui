use regex::Regex;

pub struct ParsedTask {
    pub title: String,
    pub priority: Option<u8>,
}

pub fn parse_task_input(input: &str) -> ParsedTask {
    // Regex for priority pattern, e.g. !1 to !5
    let priority_re = Regex::new(r"!(\d)").unwrap();

    let mut priority = None;
    let mut title = input.to_string();

    // Capture priority if it exists
    if let Some(caps) = priority_re.captures(input) {
        if let Some(priority_match) = caps.get(1) {
            if let Ok(p) = priority_match.as_str().parse::<u8>() {
                if (1..=5).contains(&p) {
                    priority = Some(p);
                }
            }
        }
        // Remove the priority pattern from the title
        title = priority_re.replace(input, "").trim().to_string();
    }

    ParsedTask { title, priority }
}
