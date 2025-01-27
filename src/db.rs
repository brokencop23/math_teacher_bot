use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqliteRow};
use std::error::Error;
use teloxide::prelude::ChatId;
use sqlx::Error as SqlxError;
use sqlx::migrate::MigrateError;
use crate::math::{Task, Operation};


#[derive(Debug)]
pub enum TaskStatus {
    NEW,
    CORRECT,
    WRONG
}

impl TaskStatus {

    pub fn to_int(&self) -> i32 {
        match self {
            TaskStatus::NEW => 0,
            TaskStatus::CORRECT => 1,
            TaskStatus::WRONG => 2
        }
    }

    pub fn from_int(val: i32) -> Option<TaskStatus> {
        match val {
            0 => Some(TaskStatus::NEW),
            1 => Some(TaskStatus::CORRECT),
            2 => Some(TaskStatus::WRONG),
            _ => None
        }
    }
}

#[derive(Debug)]
pub enum DBError {
    Sqlx(SqlxError),
    Migration(MigrateError),
    InvalidColumnValue
}

impl From<SqlxError> for DBError {
    fn from(err: SqlxError) -> Self {
        DBError::Sqlx(err)
    }
}

impl From<MigrateError> for DBError {
    fn from(err: MigrateError) -> Self {
        DBError::Migration(err)
    }
}

#[derive(Debug)]
pub struct TaskRow {
    id: i64,
    task: Task,
    status: TaskStatus
}

#[derive(Debug)]
pub struct User {
    id: i64,
    chat_id: ChatId,
    name: String,
    settings: Vec<UserSettings>
}

#[derive(Debug)]
pub struct UserSettings {
    operation: Operation,
    prob: f64,
    n_from: u64,
    n_to: u64
}

pub struct DB {
    pub pool: SqlitePool
}

impl DB {

    pub async fn new(url: &str) -> Result<Self, DBError> {
        let pool = SqlitePool::connect(url).await?;
        sqlx::migrate!("./src/migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn get_users(&self) -> Result<Vec<User>, DBError> {
        let users = sqlx::query(
            "SELECT
                users.id as user_id,
                chat_id,
                name,
                plus_prob,
                plus_from,
                plus_to,
                minus_prob,
                minus_from,
                minus_to,
                mul_prob,
                mul_from,
                mul_to,
                div_prob,
                div_from,
                div_to
            FROM users 
            LEFT JOIN settings
                ON users.id = settings.user_id
            ")
            .map(| r: SqliteRow | {
                let settings = vec![
                    UserSettings {
                        operation: Operation::PLUS,
                        prob: r.get("plus_prob"),
                        n_from: r.get("plus_from"),
                        n_to: r.get("plus_to")
                    },
                    UserSettings {
                        operation: Operation::MINUS,
                        prob: r.get("minus_prob"),
                        n_from: r.get("minus_from"),
                        n_to: r.get("minus_to")
                    },
                    UserSettings {
                        operation: Operation::MULTIPLY,
                        prob: r.get("mul_prob"),
                        n_from: r.get("mul_from"),
                        n_to: r.get("mul_to")
                    },
                    UserSettings {
                        operation: Operation::DIVIDE,
                        prob: r.get("div_prob"),
                        n_from: r.get("div_from"),
                        n_to: r.get("div_to")
                    }
                ];
                User {
                    id: r.get("user_id"),
                    chat_id: ChatId(r.get("chat_id")),
                    name: r.get("name"),
                    settings
                }
            })
            .fetch_all(&self.pool)
            .await?;
        Ok(users)
    }

    pub async fn get_user_by_chat(&self, chat_id: ChatId) -> Result<i64, DBError> {
        let row = sqlx::query("SELECT id FROM users WHERE chat_id=?")
            .bind(chat_id.0)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get("id"))
    }

    pub async fn new_user(&self, chat_id: ChatId) -> Result<i64, DBError> {
        let uid = sqlx::query("INSERT INTO users (chat_id) VALUES (?) RETURNING id")
            .bind(chat_id.0)
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("id");
        sqlx::query("INSERT INTO settings (user_id) VALUES (?)")
            .bind(uid)
            .execute(&self.pool)
            .await?;
        Ok(uid)
    }

    pub async fn update_user(&self, user: &User) -> Result<(), DBError> {
        sqlx::query("UPDATE users SET name=? WHERE id=?")
            .bind(&user.name)
            .bind(user.id)
            .execute(&self.pool)
            .await?;
        for setting in user.settings.iter() {
            let prefix = match setting.operation {
                Operation::PLUS => "plus",
                Operation::MINUS => "minus",
                Operation::MULTIPLY => "mul",
                Operation::DIVIDE => "div"
            };
            sqlx::query(&format!(
            "UPDATE settings
            SET {}_prob=?, {}_from=?, {}_to=?
            WHERE user_id=?"
            , prefix, prefix, prefix))
                .bind(setting.prob)
                .bind(setting.n_from as i64)
                .bind(setting.n_to as i64)
                .bind(user.id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    pub async fn add_task(&self, user_id: i64, task: &Task) -> Result<(), DBError> {
        sqlx::query(
        "INSERT INTO tasks (user_id, n_left, n_right, operation, status)
        VALUES (?, ?, ?, ?, ?)"
        )
            .bind(user_id)
            .bind(task.num_left as i64)
            .bind(task.num_right as i64)
            .bind(task.operation.to_int())
            .bind(TaskStatus::NEW.to_int())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_tasks(&self, user_id: i64) -> Result<Vec<TaskRow>, DBError> {
        let tasks = sqlx::query(
        "SELECT id, operation, status, n_left, n_right
        FROM tasks
        WHERE user_id = ?
        ")
            .bind(user_id)
            .map( | row: SqliteRow | {
                let operation = match Operation::from_int(row.get("operation")) {
                    Some(op) => op,
                    None => return Err(DBError::InvalidColumnValue)
                };
                let status = match TaskStatus::from_int(row.get("status")) {
                    Some(s) => s,
                    None => return Err(DBError::InvalidColumnValue)
                };
                Ok(TaskRow {
                    id: row.get("id"),
                    task: Task {
                        num_left: row.get::<u64, _>("n_left"),
                        num_right: row.get::<u64, _>("n_right"),
                        operation
                    },
                    status
                })
            })
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .collect::<Result<Vec<_>, DBError>>()?;
        Ok(tasks)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    async fn create_db() -> DB {
        DB::new("sqlite::memory:")
            .await
            .unwrap()
    }
    
    #[tokio::test]
    async fn test_connect() {
        let _ = create_db().await;
        assert!(true);
    }

    #[tokio::test]
    async fn test_users() {
        let db = create_db().await;
        let users = db.get_users().await.unwrap();
        assert_eq!(users.len(), 0);

        // insert user
        let uid = db.new_user(ChatId(123)).await;
        assert!(uid.is_ok());
        
        // check if more users appeared
        let users = db.get_users().await.unwrap();
        assert_eq!(users.len(), 1);

        // update user
        let user = &users[0];
        let upd_user = User {
            id: user.id,
            chat_id: user.chat_id,
            name: "Danil".to_string(),
            settings: vec![
                UserSettings {
                    operation: Operation::PLUS,
                    prob: 1.0,
                    n_from: 10,
                    n_to: 20
                }
            ]
        };
        assert!(db.update_user(&upd_user).await.is_ok());

        let user = &db.get_users().await.unwrap()[0];
        let settings = &user.settings[0];
        assert_eq!(user.name, "Danil");
        assert_eq!(settings.n_from, 10);
        assert_eq!(settings.n_to, 20);

    }

    #[tokio::test]
    async fn test_tasks() {
        let db = create_db().await;
        let _ = db.new_user(ChatId(123)).await;
        
        let users = db.get_users().await.unwrap();
        let user = &users[0];

        let new_task = Task {
            num_left: 10,
            num_right: 20,
            operation: Operation::PLUS
        };

        assert!(db.add_task(user.id, &new_task).await.is_ok());
        let tasks = db.get_tasks(user.id, None).await.unwrap();
        assert_eq!(tasks.len(), 1);
    }

}

