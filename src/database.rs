use std::path::Path;
use sqlx::{QueryBuilder, sqlite::{Sqlite, SqlitePool, SqliteConnectOptions, SqliteJournalMode}, pool::PoolConnection};
use crate::{
    AddressBook,
    state::{UserId}
};

pub struct Database {
    pool: SqlitePool,
}

pub struct DatabaseConnection {
    conn: PoolConnection<Sqlite>
}

pub struct DatabaseUserInfo {
    pub active: bool,
}

pub struct DatabaseUserPasswordInfo {
    pub password: String,
}

macro_rules! unwrap_or_return_tuple {
    ($first:expr, $opt:expr) => {
        match $opt {
            Some(v) => v,
            None => return ($first, None),
        }
    }
}

impl Database {
    pub async fn open<P: AsRef<Path>>( db_filename: P ) -> Self {
        let db_opts = SqliteConnectOptions::new()
            .filename(db_filename.as_ref())
            .journal_mode(SqliteJournalMode::Wal)
            .create_if_missing(true);
    
        let pool = SqlitePool::connect_with(db_opts).await.unwrap();
    
        Self {
            pool
        }
    }

    // #[allow(dead_code)]
    // async fn init_db(&self) {
    //     let mut conn = self.pool.acquire().await.unwrap();

    //     sqlx::query!(r#"
    //         DROP INDEX IF EXISTS "index_address_books_id";
    //         DROP TABLE IF EXISTS "address_books";
    //         DROP INDEX IF EXISTS "index_passwords_id";
    //         DROP TABLE IF EXISTS "passwords";
    //         DROP INDEX IF EXISTS "index_users_username";
    //         DROP INDEX IF EXISTS "index_users_id";
    //         DROP TABLE IF EXISTS "users";

    //         CREATE TABLE "users" (
    //             "user_id"	INTEGER NOT NULL,
    //             "active"	BOOLEAN NOT NULL,
    //             "username"	TEXT NOT NULL,
    //             PRIMARY KEY("user_id")
    //         );
            
    //         CREATE UNIQUE INDEX "index_users_id" ON "users" (
    //             "user_id"
    //         );
            
    //         CREATE INDEX "index_users_username" ON "users" (
    //             "username"
    //         );

    //         CREATE TABLE "passwords" (
    //             "user_id"	INTEGER NOT NULL,
    //             "password"	TEXT NOT NULL,
    //             PRIMARY KEY("user_id"),
    //             FOREIGN KEY("user_id") REFERENCES "users"("user_id")
    //         );
            
    //         CREATE UNIQUE INDEX "index_passwords_id" ON "passwords" (
    //             "user_id"
    //         );

    //         CREATE TABLE "address_books" (
    //             "user_id"	INTEGER NOT NULL,
    //             "ab"	TEXT NOT NULL,
    //             FOREIGN KEY("user_id") REFERENCES "users"("user_id"),
    //             PRIMARY KEY("user_id")
    //         );
            
    //         CREATE UNIQUE INDEX "index_address_books_id" ON "address_books" (
    //             "user_id"
    //         );
    //     "#)
    //     .execute(&mut conn)
    //     .await
    //     .unwrap();
    // }

    pub async fn find_user_by_name(&self, username: &str) -> (DatabaseConnection, Option<(UserId, DatabaseUserInfo)>) {
        let mut conn = DatabaseConnection { conn: self.pool.acquire().await.unwrap() };

        let res = unwrap_or_return_tuple!(conn, sqlx::query!(r#"
            SELECT
                user_id,
                active
            FROM
                users
            WHERE
                username = ?
        "#, username)
        .fetch_one(&mut conn.conn)
        .await
        .ok());

        let user_id: UserId = res.user_id;
        let dbi = DatabaseUserInfo {
            active: res.active,
        };

        (conn, Some((user_id, dbi)))
    }

    pub async fn get_user_password( &self, mut conn: DatabaseConnection, user_id: UserId ) -> (DatabaseConnection, Option<DatabaseUserPasswordInfo>) {
        let res = unwrap_or_return_tuple!(conn, sqlx::query!(r#"
            SELECT
                password
            FROM
                passwords
            WHERE
                user_id = ?
        "#, user_id)
        .fetch_one(&mut conn.conn)
        .await
        .ok());

        let dbpi = DatabaseUserPasswordInfo { 
            password: res.password 
        };

        (conn, Some(dbpi))
    }

    pub async fn get_address_book(&self, user_id: UserId) -> Option<AddressBook> {
        let mut conn = self.pool.acquire().await.unwrap();

        let res = sqlx::query!(r#"
            SELECT
                ab
            FROM
                address_books
            WHERE
                user_id = ?
        "#, user_id)
        .fetch_one(&mut conn)
        .await
        .ok()?;

        let ab = AddressBook {
            ab: res.ab,
        };

        Some(ab)
    }

    pub async fn update_address_books(&self, mut values: Vec<(UserId, AddressBook)>) -> Option<()> {
        let mut tx = self.pool.begin().await.unwrap();

        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            // Note the trailing space; most calls to `QueryBuilder` don't automatically insert
            // spaces as that might interfere with identifiers or quoted strings where exact
            // values may matter.
            "INSERT OR REPLACE INTO address_books (user_id, ab) "
        );
        
        let values_count = values.len() as u64;

        // Note that `.into_iter()` wasn't needed here since `users` is already an iterator.
        query_builder.push_values(values.drain(..), |mut b, value| {
            b
            .push_bind(value.0)
            .push_bind(value.1.ab);
        });
        
        let query = query_builder.build();

        let res = query
        .execute(&mut tx)
        .await
        .ok()?
        .rows_affected();

        if res != values_count {
            return None;
        }

        tx.commit().await.ok()?;

        Some(())
    }
}