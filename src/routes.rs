///route definitions and handlers
use axum::{
    Json,
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post}, //hhtp method routes
};
use sqlx::SqlitePool;
use uuid::Uuid;
use validator::Validate;

use crate::models::{CreateTask, ListQuery, TagBody, TagRecord, UpdateTask};
use crate::{errors::AppError, models::DatabaseTask};

pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        //route for post (create) a new task, get on all tasks
        .route("/tasks", post(create_task).get(list_tasks))
        //route for get, put (update), and delete on single task
        .route(
            "/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
        //add tag to task
        .route("/tasks/{id}/tags", post(add_tag))
        //remove tag from task
        .route("/tasks/{id}/tags/{tag}", delete(remove_tag))
        //attaching db connection pool as shared state so all route handlers can access
        .with_state(pool)
}

///handlers
// i. Create a new task- POST /tasks
async fn create_task(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateTask>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    let task = body.into_task();

    // Create bindings for temporary values
    let task_id = task.id.to_string();
    let task_status = task.status.to_string();
    let task_priority = task.priority.to_string();
    let task_due_date = task.due_date.map(|d| d.to_rfc3339());
    let task_created_at = task.created_at.to_rfc3339();
    let task_updated_at = task.updated_at.to_rfc3339();

    sqlx::query!(
        "INSERT INTO tasks (id, title, description, status, priority, due_date, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        task_id,
        task.title,
        task.description,
        task_status,
        task_priority,
        task_due_date,
        task_created_at,
        task_updated_at,
    ).execute(&pool).await?;

    for tag in &task.tags {
        sqlx::query!(
            "INSERT INTO task_tags (task_id, tag) VALUES (?, ?)",
            task_id,
            tag
        )
        .execute(&pool)
        .await?;
    }

    Ok((axum::http::StatusCode::CREATED, Json(task)))
}

//ii. Retrieve a single task- GET /tasks/:id
async fn get_task(
    Path(id): Path<Uuid>,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let Some(database_task) = sqlx::query_as::<_, DatabaseTask>("SELECT * FROM tasks WHERE id = ?")
        .bind(id.to_string())
        .fetch_optional(&pool) //expecting 0 or 1 row
        .await?
    else {
        return Err(AppError::NotFound);
    };

    let tags = sqlx::query_as::<_, TagRecord>("SELECT tag FROM task_tags WHERE task_id = ?")
        .bind(&database_task.id)
        .fetch_all(&pool)
        .await?
        .into_iter()
        .map(|record| record.tag)
        .collect::<Vec<String>>();

    let task = database_task.into_task(tags)?;

    Ok(Json(task))
}

// iii. List tasks with filtering and sorting- GET /tasks
async fn list_tasks(
    State(pool): State<SqlitePool>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, AppError> {
    let mut sql = String::from("SELECT * FROM tasks");
    let mut conditions = Vec::new();
    let mut params: Vec<String> = Vec::new();

    // Filtering
    if let Some(ref status) = query.status {
        conditions.push("status = ?");
        params.push(status.clone());
    }
    if let Some(ref priority) = query.priority {
        conditions.push("priority = ?");
        params.push(priority.clone());
    }
    if let Some(ref title) = query.title {
        conditions.push("title LIKE ?");
        params.push(format!("%{title}%"));
    }
    if let Some(ref tag) = query.tag {
        conditions.push("id IN (SELECT task_id FROM task_tags WHERE tag = ?)");
        params.push(tag.clone());
    }
    if let Some(ref due_before) = query.due_before {
        conditions.push("due_date <= ?");
        params.push(due_before.to_rfc3339());
    }
    if let Some(ref due_after) = query.due_after {
        conditions.push("due_date >= ?");
        params.push(due_after.to_rfc3339());
    }
    if let Some(ref created_before) = query.created_before {
        conditions.push("created_at <= ?");
        params.push(created_before.to_rfc3339());
    }
    if let Some(ref created_after) = query.created_after {
        conditions.push("created_at >= ?");
        params.push(created_after.to_rfc3339());
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    // Sorting
    if let Some(sort_by) = &query.sort_by {
        let allowed_fields = [
            "title",
            "priority",
            "status",
            "due_date",
            "created_at",
            "updated_at",
        ];
        if allowed_fields.contains(&sort_by.as_str()) {
            let order = match query.sort_order.as_deref() {
                Some("desc") => "DESC",
                _ => "ASC",
            };
            sql.push_str(&format!(" ORDER BY {sort_by} {order}"));
        }
    }

    // Build and execute query
    let mut query_builder = sqlx::query_as::<_, DatabaseTask>(&sql);
    for param in &params {
        query_builder = query_builder.bind(param);
    }

    let database_tasks = query_builder.fetch_all(&pool).await?;

    let mut tasks = Vec::new();

    for database_task in database_tasks {
        let tags = sqlx::query_as::<_, TagRecord>("SELECT tag FROM task_tags WHERE task_id = ?")
            .bind(&database_task.id)
            .fetch_all(&pool)
            .await?
            .into_iter()
            .map(|record| record.tag)
            .collect::<Vec<String>>();

        let task = database_task.into_task(tags)?;
        tasks.push(task);
    }

    Ok(Json(tasks))
}

// iv. Update task details- PUT /tasks/:id
async fn update_task(
    Path(id): Path<Uuid>,
    State(pool): State<SqlitePool>,
    Json(body): Json<UpdateTask>,
) -> Result<impl IntoResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;
    let now = chrono::Utc::now();
    let now_fmt = now.to_rfc3339();
    let id_str = id.to_string();
    let status_str = body.status.map(|p| p.to_string());
    let priority_str = body.priority.map(|p| p.to_string());
    let due_fmt = body.due_date.map(|d| d.to_rfc3339());

    sqlx::query!(
        "UPDATE tasks SET title = ?, description = ?, status = ?, priority = ?, due_date = ?, updated_at = ? WHERE id = ?",
        body.title,
        body.description,
        status_str,
        priority_str,
        due_fmt,
        now_fmt,
        id_str,
    ).execute(&pool).await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}

// v. Delete a task- DELETE /tasks/:id
async fn delete_task(
    Path(id): Path<Uuid>,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let id_str = id.to_string();
    sqlx::query!("DELETE FROM tasks WHERE id = ?", id_str)
        .execute(&pool)
        .await?;
    sqlx::query!("DELETE FROM task_tags WHERE task_id = ?", id_str)
        .execute(&pool)
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

// vi. Add a tag to a task
async fn add_tag(
    State(pool): State<SqlitePool>,
    Path(id): Path<Uuid>,
    Json(body): Json<TagBody>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "INSERT INTO task_tags (task_id, tag) VALUES (?, ?)",
        id,
        body.tag
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::CREATED)
}

// vii. Remove a tag from a task
async fn remove_tag(
    State(pool): State<SqlitePool>,
    Path((id, tag)): Path<(Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "DELETE FROM task_tags WHERE task_id = ? AND tag = ?",
        id,
        tag
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}
