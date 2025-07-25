///data types
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, FromRow)] //for json, turning query results into Tasks to map to db
#[serde(rename_all = "camelCase")] //to output proper json fields
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: Status, //Todo, In Progress, Done,
    pub priority: Priority, //Low, Medium, High,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>, //multiple
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "TEXT")] //Make sure db expects text
pub enum Status {
    Todo,
    InProgress,
    Done,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Status::Todo => "Todo",
            Status::InProgress => "InProgress",
            Status::Done => "Done",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Status {
    type Err = ParseStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "todo" => Ok(Status::Todo),
            "inprogress" => Ok(Status::InProgress),
            "done" => Ok(Status::Done),
            _ => Err(ParseStatusError),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "TEXT")]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Priority {
    type Err = ParsePriorityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            _ => Err(ParsePriorityError),
        }
    }
}



#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateTask { //created by client
    #[validate(length(min = 1))] //make sure title isn't empty
    pub title: String,

    #[validate(length(min = 1))] //make sure description isn't empty
    pub description: String,

    pub status: Status,
    pub priority: Priority,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

impl CreateTask {
    pub fn into_task(self) -> Task { //turns CreateTask (from client) into full Task for db (id and times)
        let now = Utc::now();
        Task {
            id: Uuid::new_v4(),
            title: self.title,
            description: self.description,
            status: self.status,
            priority: self.priority,
            due_date: self.due_date,
            created_at: now,
            updated_at: now,
            tags: self.tags,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTask {
    #[validate(length(min = 1))]
    pub title: Option<String>,

    #[validate(length(min = 1))]
    pub description: Option<String>,

    pub status: Option<Status>,
    pub priority: Option<Priority>,
    pub due_date: Option<DateTime<Utc>>,

    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery { //query parameters for getting task list
    //filtering
    pub status: Option<String>,
    pub priority: Option<String>,
    pub title: Option<String>,
    pub tag: Option<String>,
    pub due_before: Option<DateTime<Utc>>,
    pub due_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
    //sorting
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, //asc or desc
}

#[derive(Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TagBody { //info contained in tag
    #[validate(length(min = 1))]
    pub tag: String,
}


