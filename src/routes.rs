///route definitions and handlers

use axum::{
    http::StatusCode,
    extract::{Path, State, Query},
    response::IntoResponse,
    routing::{get, post, put, delete}, //hhtp method routes
    Json, Router,
};
use sqlx::{Row,SqlitePool};
use uuid::Uuid;
use serde::Deserialize;
use validator::Validate;	
use std::str::FromStr;

use crate::errors::AppError;
use crate::models::{Priority, Status, CreateTask, Task, UpdateTask, ListQuery, TagBody};

pub fn create_router(pool: SqlitePool) -> Router {
    Router::new()
        //route for post (create) a new task, get on all tasks
        .route("/tasks", post(create_task).get(list_tasks))
        //route for get, put (update), and delete on single task
        .route("/tasks/:id", get(get_task).put(update_task).delete(delete_task))
        //add tag to task
        .route("/tasks/:id/tags", post(add_tag))
        //remove tag from task
        .route("/tasks/:id/tags/:tag", delete(remove_tag))
        //attaching db connection pool as shared state so all route handlers can access
        .with_state(pool)
}

///handlers
// i. Create a new task- POST /tasks
async fn create_task(State(pool): State<SqlitePool>, Json(body): Json<CreateTask>) -> Result<impl IntoResponse, AppError> {
    body.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;
    let task = body.into_task();

    sqlx::query!(
        "INSERT INTO tasks (id, title, description, status, priority, due_date, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        task.id.to_string(),
        task.title,
        task.description,
        task.status.to_string(),
        task.priority.to_string(),
        task.due_date.map(|d| d.to_rfc3339()),
        task.created_at.to_rfc3339(),
        task.updated_at.to_rfc3339(),
    ).execute(&pool).await?;

    for tag in &task.tags {
        sqlx::query!("INSERT INTO task_tags (task_id, tag) VALUES (?, ?)", task.id.to_string(), tag)
            .execute(&pool)
            .await?;
    }

    Ok((axum::http::StatusCode::CREATED, Json(task)))
}

//ii. Retrieve a single task- GET /tasks/:id
async fn get_task(Path(id): Path<Uuid>, State(pool): State<SqlitePool>) -> Result<impl IntoResponse, AppError> {
    let row = sqlx::query!("SELECT * FROM tasks WHERE id = ?", id.to_string())
        .fetch_optional(&pool) //expecting 0 or 1 row
        .await?;

    match row {
        Some(row) => {
            let tags = sqlx::query!("SELECT tag FROM task_tags WHERE task_id = ?", row.id)
                .fetch_all(&pool)
                .await?
                .into_iter()
                .filter_map(|r| r.tag) 
                .collect::<Vec<String>>();

            let due_date = match row.get::<Option<String>, _>("due_date") {
                    Some(s) => Some(s.parse()?),
                    None => None,
                };

            let task = Task {
                id: row.id.parse()?,
                title: row.title,
                description: row.description,
                status: row.status.parse()?,
                priority: row.priority.parse()?,
                due_date,
                created_at: row.created_at.parse()?,
                updated_at: row.updated_at.parse()?,
                tags,
            };
            Ok(Json(task));
        },  
        None => Err(AppError::NotFound),
    }
}

// iii. List tasks with filtering and sorting- GET /tasks
async fn list_tasks(State(pool): State<SqlitePool>, Query(query): Query<ListQuery>) -> Result<impl IntoResponse, AppError> {
    let mut sql = sqlx::query!("SELECT * FROM tasks");
    let mut conditions = Vec::new();
    let mut args: Vec<(usize, String)> = Vec::new();
    let mut bind_index = 1;

    // Filtering
    if let Some(ref status) = query.status {
        conditions.push(format!("status = ?{}", bind_index));
        args.push((bind_index, status.clone()));
        bind_index += 1;
    }
    if let Some(ref priority) = query.priority {
        conditions.push(format!("priority = ?{}", bind_index));
        args.push((bind_index, priority.clone()));
        bind_index += 1;
    }
    if let Some(ref title) = query.title {
        conditions.push(format!("title LIKE ?{}", bind_index));
        args.push((bind_index, format!("%{}%", title)));
        bind_index += 1;
    }
    if let Some(ref tag) = query.tag {
        conditions.push(format!("id IN (SELECT task_id FROM task_tags WHERE tag = ?{})", bind_index));
        args.push((bind_index, tag.clone()));
        bind_index += 1;
    }
    if let Some(ref due_before) = query.due_before {
        conditions.push(format!("due_date <= ?{}", bind_index));
        args.push((bind_index, due_before.to_rfc3339()));
        bind_index += 1;
    }
    if let Some(ref due_after) = query.due_after {
        conditions.push(format!("due_date >= ?{}", bind_index));
        args.push((bind_index, due_after.to_rfc3339()));
        bind_index += 1;
    }
    if let Some(ref created_before) = query.created_before {
        conditions.push(format!("created_at <= ?{}", bind_index));
        args.push((bind_index, created_before.to_rfc3339()));
        bind_index += 1;
    }
    if let Some(ref created_after) = query.created_after {
        conditions.push(format!("created_at >= ?{}", bind_index));
        args.push((bind_index, created_after.to_rfc3339()));
        bind_index += 1;
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    // Sorting
    if let Some(sort_by) = &query.sort_by {
        let allowed_fields = ["title", "priority", "status", "due_date", "created_at", "updated_at"];
        if allowed_fields.contains(&sort_by.as_str()) {
            let order = match query.sort_order.as_deref() {
                Some("desc") => "DESC",
                _ => "ASC",
            };
            sql.push_str(&format!(" ORDER BY {} {}", sort_by, order));
        }
    }

    // Final query execution
    let mut query_builder = sqlx::query(&sql);
    for (_, val) in &args {
        query_builder = query_builder.bind(val);
    }

    let rows = query_builder.fetch_all(&pool).await?;

    let mut tasks = Vec::new();

    for row in rows {
        let task_id: String = row.get("id");
        let tags = sqlx::query!("SELECT tag FROM task_tags WHERE task_id = ?", task_id)
            .fetch_all(&pool)
            .await?
            .into_iter()
            .filter_map(|r| r.tag) // skips None, unwraps Some
            .collect::<Vec<String>>();

        let due_date = match row.get::<Option<String>, _>("due_date") {
            Some(s) => Some(s.parse()?),
            None => None,
            };

        let task = Task {
            id: task_id.parse()?,
            title: row.get("title"),
            description: row.get("description"),
            status: row.get::<String, _>("status").parse()?,
            priority: row.get::<String, _>("priority").parse()?,
            due_date,
            created_at: row.get::<String, _>("created_at").parse()?,
            updated_at: row.get::<String, _>("updated_at").parse()?,
            tags,
        };

        tasks.push(task);
    }

    Ok(Json(tasks))
}

// iv. Update task details- PUT /tasks/:id
async fn update_task(Path(id): Path<Uuid>, State(pool): State<SqlitePool>, Json(body): Json<UpdateTask>) -> Result<impl IntoResponse, AppError> {
    body.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;
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
async fn delete_task(Path(id): Path<Uuid>, State(pool): State<SqlitePool>) -> Result<impl IntoResponse, AppError> {
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
    Path(id): Path<String>,
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
async fn remove_tag(State(pool): State<SqlitePool>, Path((id,tag)): Path<(String,String)>) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        "DELETE FROM task_tags WHERE task_id = ? AND tag = ?",
        id,
        tag
    )
    .execute(&pool)
    .await?;

    Ok(StatusCode::NO_CONTENT)
}
