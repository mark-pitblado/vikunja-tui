use serde::Deserialize;

// Task struct
#[derive(Clone, Deserialize, Debug)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub done: bool,
}

// TaskDetail struct with description
#[derive(Deserialize, Debug)]
pub struct TaskDetail {
    pub id: u64,
    pub title: String,
    pub done: bool,
    pub due_date: Option<String>,
    pub labels: Option<Vec<Label>>,
    pub priority: Option<i32>,
    pub description: Option<String>,
}

// Label struct
#[derive(Deserialize, Debug)]
pub struct Label {
    pub id: u64,
    pub title: String,
}
