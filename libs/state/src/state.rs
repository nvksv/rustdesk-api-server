use std::{
    default::Default,
    collections::HashMap,
    time::SystemTime,
    sync::atomic::{AtomicU64, Ordering},
    path::Path,
};
use tokio::sync::RwLock;
use utils::{AddressBook, Token};
use crate::{
    UserId, SessionId,
    database::Database, 
    bearer::AuthenticatedUser,
    password::UserPasswordInfo,
};
use crate::ui;

pub struct ApiState {
    last_maintenance_time: AtomicU64,
    access_tokens: RwLock<HashMap<Token, AccessTokenInfo>>,
    sessions: RwLock<SessionsState>,
    users: RwLock<HashMap<UserId, UserInfo>>,
    address_books: RwLock<HashMap<UserId, AddressBookInfo>>,
    db: Database,
}

#[derive(Debug, Clone)]
pub struct AccessTokenInfo {
    pub session_id: SessionId,
    pub user_id: UserId,
}

#[derive(Debug, Default)]
struct SessionsState {
    counter: SessionId,
    sessions: HashMap<SessionId, SessionInfo>,
}

#[derive(Debug, Default)]
pub struct UserInfo {
    sessions_count: usize,
    pub username: String,
    pub admin: bool,
}

#[derive(Debug, Default)]
struct SessionInfo {
    #[allow(dead_code)]
    user_id: UserId,
}

#[derive(Debug, Clone)]
pub struct AddressBookInfo {
    modified: bool,
    remove_after_flush: bool,
    pub address_book: AddressBook,
}

const MAINTENANCE_INTERVAL_IN_SECS: u64 = 60;

fn secs_from_epoch() -> u64 {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}

impl ApiState {
    pub async fn new_with_db<P: AsRef<Path>>( db_filename: P ) -> Self {
        let db = Database::open( db_filename ).await;
        Self { 
            last_maintenance_time: AtomicU64::new(0),
            access_tokens: Default::default(), 
            sessions: Default::default(), 
            users: Default::default(), 
            address_books: Default::default(), 
            db 
        }
    }

    pub async fn maintenance_flush_address_books(&self) {
        let mut state_address_books = self.address_books.write().await;
        
        let mut values: Vec<(UserId, AddressBook)> = vec![];

        for (user_id, address_book_info) in state_address_books.iter_mut() {
            if !address_book_info.modified {
                continue;
            }

            values.push((*user_id, address_book_info.address_book.clone()));
            address_book_info.modified = false;
        }

        if !values.is_empty() {
            tracing::debug!("Need to update_address_books");
            self.db.update_address_books(values).await;
        }
    }

    pub async fn maintenance(&self) {
        self.maintenance_flush_address_books().await;
    }

    pub async fn check_maintenance(&self) {
        tracing::debug!("check_maintenance...");

        let now = secs_from_epoch();
        let last_mt = self.last_maintenance_time.load(Ordering::Relaxed);

        if now >= (last_mt + MAINTENANCE_INTERVAL_IN_SECS) {
            self.last_maintenance_time.store(now, Ordering::Relaxed);
            tracing::debug!("check_maintenance NOW");
            self.maintenance().await;
        }
    }

    pub async fn user_login<'s>(&self, username: &String, password_info: UserPasswordInfo<'s>) -> Option<(String, Token)> {
        let (conn, user_id, db_user_info) = match self.db.find_user_by_name(username.as_str()).await {
            (conn, Some((user_id, db_user_info))) => (conn, user_id, db_user_info),
            _ => return None,
        };

        if !db_user_info.active {
            return None;
        }

        let (conn, db_password_info) = match self.db.get_user_password(conn, user_id).await {
            (conn, Some(db_password_info)) => (conn, db_password_info),
            _ => return None,
        };

        if !password_info.check(db_password_info) {
            return None;
        }

        drop(conn);

        let access_token = Token::new_random();

        let mut state_access_tokens = self.access_tokens.write().await;
        let mut state_sessions = self.sessions.write().await;
        let mut state_users = self.users.write().await;

        state_sessions.counter += 1;
        let session_id = state_sessions.counter;

        if let Some(user_info) = state_users.get_mut(&user_id) {
            user_info.sessions_count += 1;
        } else {
            let user_info = UserInfo {
                sessions_count: 1,
                username: username.clone(),
                admin: db_user_info.admin,
            };
            state_users.insert( user_id, user_info );

            let mut state_address_books = self.address_books.write().await;
            if let Some(abi) = state_address_books.get_mut(&user_id) {
                abi.remove_after_flush = false;
            }
        }

        let session_info = SessionInfo {
            user_id,
        };

        let access_token_info = AccessTokenInfo {
            session_id,
            user_id,
        };

        let _ = state_sessions.sessions.insert(session_id, session_info);
        let _ = state_access_tokens.insert( access_token.clone(), access_token_info );

        Some((username.to_string(), access_token))
    }

    pub async fn find_session(&self, access_token: &Token) -> Option<AccessTokenInfo> {
        let state_access_tokens = self.access_tokens.read().await;

        state_access_tokens.get( access_token ).map(|t| t.clone())
    }

    pub async fn get_user_address_book(&self, user_id: UserId) -> Option<AddressBook> {
        let state_address_books = self.address_books.read().await;

        let opt_ab = state_address_books
            .get(&user_id)
            .map(|abi| abi.address_book.clone());

        if opt_ab.is_some() {
            return opt_ab;
        }

        drop(state_address_books);

        let ab = self.db.get_address_book(user_id).await?;
        let abi = AddressBookInfo {
            modified: false,
            remove_after_flush: false,
            address_book: ab.clone(),
        };

        let mut state_address_books = self.address_books.write().await;
        state_address_books.insert(user_id, abi.clone());

        Some(ab)
    }

    pub async fn set_user_address_book(&self, user_id: UserId, address_book: AddressBook) -> Option<()> {
        tracing::debug!("set_user_ab()");
        let mut state_address_books = self.address_books.write().await;

        if let Some(abi) = state_address_books.get_mut(&user_id) {
            if abi.address_book != address_book {
                abi.modified = true;
                abi.address_book = address_book;
            };
        } else {
            let abi = AddressBookInfo {
                modified: false,
                remove_after_flush: false,
                address_book,
            };
            state_address_books.insert( user_id, abi );
        }
        // tracing::debug!("set_user_ab() 2");

        // let _ = self.db.update_ab( user_id, &abi.ab ).await;

        tracing::debug!("ab done!");
        Some(())
    }

    pub async fn user_logout(&self, user: &AuthenticatedUser) -> Option<()> {
        let mut state_access_tokens = self.access_tokens.write().await;
        let mut state_sessions = self.sessions.write().await;
        let mut state_users = self.users.write().await;

        let user_info = state_users.get_mut(&user.user_id)?;
        user_info.sessions_count -= 1;

        if user_info.sessions_count == 0 {
            state_users.remove(&user.user_id);

            let mut state_address_books = self.address_books.write().await;
            if let Some(abi) = state_address_books.get_mut(&user.user_id) {
                abi.remove_after_flush = true;
            }
        }

        state_sessions.sessions.remove(&user.session_id);
        state_access_tokens.remove(&user.access_token);

        Some(())
    }

    pub async fn get_current_user_name(&self, user: &AuthenticatedUser) -> Option<String> {
        let state_users = self.users.read().await;
        state_users.get(&user.user_id).map(|ui| ui.username.clone())
    }

    pub async fn with_user_info<R>(&self, user_id: &UserId, mut f: impl FnMut(&UserInfo) -> R) -> Option<R> {
        let state_users = self.users.read().await;
        if let Some(user_info) = state_users.get(user_id) {
            Some(f(user_info))
        } else {
            None
        }
    }

    pub async fn ui_get_all_users(&self) -> Option<Vec<ui::UserInfo>> {
        self.db.ui_get_all_users().await
    }

}


