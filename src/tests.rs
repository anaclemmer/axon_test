#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::models::{CreateTask, Priority, Status};
    use chrono::{TimeZone, Utc};
    use validator::Validate;

    #[test]
    fn test_create_task_conversion() {
        let input = CreateTask {
            title: "Test".to_string(),
            description: "Test desc".to_string(),
            status: Status::Todo,
            priority: Priority::Medium,
            due_date: Some(Utc.with_ymd_and_hms(2025, 8, 1, 12, 0, 0).unwrap()),
            tags: vec!["a".to_string(), "b".to_string()],
        };

        let task = input.into_task();

        assert_eq!(task.title, "Test");
        assert_eq!(task.status, Status::Todo);
        assert_eq!(task.priority, Priority::Medium);
        assert_eq!(task.tags.len(), 2);
        assert!(task.due_date.is_some());
    }

    #[test]
    fn test_create_task_validation_fails() {
        let invalid = CreateTask {
            title: "".to_string(),
            description: "".to_string(),
            status: Status::Todo,
            priority: Priority::Low,
            due_date: None,
            tags: vec![],
        };

        let result = invalid.validate();
        assert!(result.is_err());
    }
}
