use crate::database::DatabaseUserPasswordInfo;

pub struct UserPasswordInfo<'s> {
    password: &'s str,
}

impl<'s> UserPasswordInfo<'s> {
    pub fn from_password( password: &'s str ) -> Self {
        Self { 
            password
        }
    }

    pub fn check( &self, db_password_info: DatabaseUserPasswordInfo ) -> bool {
        db_password_info.password.eq( self.password )
    }
}

