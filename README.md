# Task Management System
By: Ana Clemmer
---

This project is a simple task management system with REST API, built in Rust and using axum for the HTTP server. I learned a lot throughout this project and I'm grateful for the opportunity to expand my horizons with AXON Networks even through this preliminary test!

## Setup Instructions

0. Install:
- Rust
- SQLite
- sqlx-cli: 
    ```bash
    cargo install sqlx-cli --no-default-features --features sqlite

1. Clone and enter repo.
    
    git clone <your-repo-url>
    cd axon_test

2. Set up environment.
    
    Create an .env file with this content: DATABASE_URL=sqlite://axon_test.db

3. Run database migrations.
    
    sqlx migrate run

4. Run server (available at http://localhost:3000).
    
    cargo run


## API Documentation

### GET /tasks

    Lists all tasks with optional filters and sorting.

| Query Parameter   | Type    | Description                          |
|------------------|---------|--------------------------------------|
| `status`         | String  | Filter by status                     |
| `priority`       | String  | Filter by priority                   |
| `title`          | String  | Filter by title (like a search)      |
| `tag`            | String  | Filter by tag                        |
| `due_before`     | Date    | Filter tasks due before a date       |
| `due_after`      | Date    | Filter tasks due after a date        |
| `created_before` | Date    | Filter tasks created before a date   |
| `created_after`  | Date    | Filter tasks created after a date    |
| `sort_by`        | String  | Field to sort on (`due_date`, etc.)  |
| `sort_order`     | String  | `asc` (default) or `desc`            |

    
    Response: 200 OK + list of tasks

### POST /tasks

    Creates a new task      

    Request body (JSON):

        {
            "title": "string",
            "description": "string",
            "status": "Todo | InProgress | Done",
            "priority": "Low | Medium | High",
            "dueDate": "ISO 8601 string (optional)",
            "tags": ["string", ...]
        }  
    
    Response: 
    
        201 Created, full task object with newly generated id and timestamps

### GET /tasks/:id

    Parameter :id is task UUID

    Gets a single task

    Response:

        200 OK, task JSON
        If task doesn't exist: 404 Not Found


### PUT /tasks/:id

    Parameter :id is task UUID

    Updates existing task

    Request body:
        
        {
            "title": "Updated title",
            "description": "Updated description",
            "status": "InProgress",
            "priority": "Medium",
            "dueDate": "2025-09-01T15:00:00Z",
            "tags": ["work", "high priority"]
        }

    Response:

        If task was updated: 204 No Content
        If task doesn't exist: 404 Not Found

### DELETE /tasks/:id

    Parameter :id is task UUID

    Deletes a task

    Response:

        If task was deleted: 204 No Content
        If task doesn't exist: 404 Not Found

## Example Usage

- Create a task:

    Send a POST request to http://localhost:3000/tasks
    
        ```bash
        curl -X POST http://localhost:3000/tasks \
        -H "Content-Type: application/json" \
        -d '{
            "title": "Buy groceries",
            "description": "Milk, eggs, and bread",
            "status": "Todo",
            "priority": "Medium",
            "dueDate": "2025-08-01T12:00:00Z",
            "tags": ["personal", "errands"]
        }'

- List all tasks:

    curl http://localhost:3000/tasks

- Filter tasks by status and priority

    Send a GET request to http://localhost:3000/tasks?status=Todo&priority=High

    Will return a list of tasks with status Todo and High priority

- Filter tasks by due date range

    curl "http://localhost:3000/tasks?due_after=2025-07-01T00:00:00Z&due_before=2025-08-01T00:00:00Z"

- Sort tasks by due date

    curl "http://localhost:3000/tasks?sort_by=due_date&sort_order=desc"

- Update task by ID

    curl -X PUT http://localhost:3000/tasks/<task-id-here> \
    -H "Content-Type: application/json" \
    -d '{
        "status": "InProgress",
        "priority": "High"
    }'

- Delete a task

    curl -X DELETE http://localhost:3000/tasks/<task-id-here>

- Add a tag to a task

    curl -X POST http://localhost:3000/tasks/<task-id-here>/tags \
    -H "Content-Type: application/json" \
    -d '{
        "tag": "urgent"
    }'

- Remove a tag from a task

    curl -X DELETE http://localhost:3000/tasks/<task-id-here>/tags \
    -H "Content-Type: application/json" \
    -d '{
        "tag": "urgent"
    }'


## Assumptions/design
- Task ids are generated server-side as UUID
- Tags are stored in a separate task_tags table (many to one)
- Dates are UTC, stored as RFC3339 strings
- Validation is handled with the validator crate
- Field names are snake_case in Rust code, and camelCase in JSON 
- Enum fields are stored as strings in SQLite


