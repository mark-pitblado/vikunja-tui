use crate::models::{Task, TaskDetail};
use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub async fn fetch_tasks(
    instance_url: &str,
    api_key: &str,
    page: usize,
) -> Result<Vec<Task>, reqwest::Error> {
    let client = Client::new();
    let url = format!("{}/api/v1/tasks/all?page={}", instance_url, page);

    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?
        .json::<Vec<Task>>()
        .await?;

    Ok(res)
}

pub async fn fetch_task_detail(
    instance_url: &str,
    api_key: &str,
    task_id: u64,
) -> Result<TaskDetail, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("{}/api/v1/tasks/{}", instance_url, task_id);

    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if res.status().is_success() {
        let task_detail = res.json::<TaskDetail>().await?;
        Ok(task_detail)
    } else {
        let error_text = res.text().await?;
        Err(format!("Error fetching task detail: {}", error_text).into())
    }
}

pub async fn create_new_task(
    instance_url: &str,
    api_key: &str,
    task_title: &str,
    description: Option<&str>,
    priority: Option<u8>,
) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let url = format!("{}/api/v1/projects/1/tasks", instance_url);

    let mut task_data = json!({
        "title": task_title
    });

    if let Some(desc) = description {
        task_data["description"] = json!(desc);
    }

    if let Some(priority_value) = priority {
        task_data["priority"] = json!(priority_value);
    }

    let res = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&task_data)
        .send()
        .await?;

    if res.status().is_success() {
        Ok(())
    } else {
        let error_text = res.text().await?;
        Err(format!("Error creating task: {}", error_text).into())
    }
}
