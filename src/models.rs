use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Debug, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Url {
    #[serde(skip_deserializing)]
    pub id: i64,
    pub path: String,
    pub title: String,
    pub body: String,
    pub icon: String,
    pub redirect: String,
    #[serde(skip_deserializing)]
    pub hits: i64,
    #[serde(skip_deserializing, serialize_with = "fmt_naive")]
    pub created_at: NaiveDateTime,
    #[serde(skip_deserializing, serialize_with = "fmt_naive")]
    pub updated_at: NaiveDateTime,
}

fn fmt_naive<S>(dt: &NaiveDateTime, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let dt = DateTime::<Utc>::from_utc(*dt, Utc);
    s.serialize_str(&dt.to_rfc3339())
}

impl Url {
    pub async fn from_path(path: &str, pool: &crate::db::Pool) -> Result<Option<Self>> {
        let r = sqlx::query_as!(
            Url,
            r#"
                SELECT * FROM urls
                WHERE `path` = ?
                LIMIT 1
            "#,
            path
        )
        .fetch_optional(pool)
        .await
        .context("fetch from path failed")?;
        Ok(r)
    }
    pub async fn list(pool: &crate::db::Pool) -> Result<Vec<Self>> {
        let r = sqlx::query_as!(
            Url,
            r#"
                SELECT * FROM urls
            "#
        )
        .fetch_all(pool)
        .await
        .context("fetch all failed")?;
        Ok(r)
    }
    pub async fn insert(self, pool: &crate::db::Pool) -> Result<()> {
        let now = Utc::now().naive_utc();
        sqlx::query!(
            r#"
                INSERT INTO urls (
                    `path`, `title`, `body`, `icon`, `redirect`,
                    `created_at`, `updated_at`
                )
                VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            self.path,
            self.title,
            self.body,
            self.icon,
            self.redirect,
            now,
            now,
        )
        .execute(pool)
        .await
        .context("insert failed")?;
        Ok(())
    }
    pub async fn delete(id: i64, pool: &crate::db::Pool) -> Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM urls
                WHERE id = ?
            "#,
            id
        )
        .execute(pool)
        .await
        .context("delete failed")?;
        Ok(())
    }
    pub async fn update(&self, pool: &crate::db::Pool) -> Result<()> {
        let now = Utc::now().naive_utc();
        sqlx::query!(
            r#"
                UPDATE urls
                SET
                    `path` = ?, `title` = ?, `body` = ?, `icon` = ?, `redirect` = ?,
                    `updated_at` = ?
                WHERE id = ?
            "#,
            self.path,
            self.title,
            self.body,
            self.icon,
            self.redirect,
            self.id,
            now
        )
        .execute(pool)
        .await
        .context("update failed")?;
        Ok(())
    }
    pub async fn incr(id: i64, pool: &crate::db::Pool) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE urls
                SET
                    `hits` = `hits` + 1
                WHERE id = ?
            "#,
            id
        )
        .execute(pool)
        .await
        .context("incr failed")?;
        Ok(())
    }
}

impl Url {
    pub fn html(&self) -> String {
        format!(
            r#"""<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="shortcut icon" href="{}" type="image/x-icon">
    <title>{}</title>
</head>

<body>
<div>{}</div>
</body>
</html>"""#,
            self.icon, self.title, self.body
        )
    }
}
