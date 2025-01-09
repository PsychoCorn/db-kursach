use postgres::{Client, NoTls};
use super::CONNECTION;
use super::*;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone)]
pub(super) struct Student {
    pub login: String,
    pub number: i32,
}

impl Student {
    pub fn new(login: &str, number: i32) -> Self {
        Self {
            login: login.to_owned(),
            number
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Teacher {
    pub login: String,
}

impl Teacher {
    pub fn new(login: &str) -> Self {
        Self {
            login: login.to_owned()
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum User {
    UnknownUser,
    Student(Student),
    Teacher(Teacher),
    Decan,
    Admin
}

pub(super) fn check_user(login: &str, password: &str) -> Result<User, postgres::Error> {
    let mut hasher = DefaultHasher::new();
    let mut client = Client::connect(CONNECTION, NoTls).unwrap();

    password.hash(&mut hasher);
    let hashed_password = hasher.finish().to_string();

    let query = format!("select login, password, id_role from users where users.login = '{}' and users.password = '{}'", login, hashed_password);
    let row = client.query(&query, &[])?;

    if row.is_empty() { Ok(User::UnknownUser) } else {
        let id_role = row[0].get(2);
        match id_role {
            1 => Ok(User::Admin),
            2 => Ok(User::Decan),
            3 => Ok(User::Student(Student::new(row[0].get(0), 0))),
            4 => Ok(User::Teacher(Teacher::new(row[0].get(0)))),
            _ => unreachable!()
        }
    }
}

//pub(super) fn register_user()