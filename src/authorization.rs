use postgres::{Client, NoTls};
use tables::group;
use super::CONNECTION;
use super::*;
use std::{fmt::{write, Display}, hash::{DefaultHasher, Hash, Hasher}};

#[derive(Debug, Clone)]
pub(super) struct  Group {
    pub specialization_id: i64,
    pub specialization_name: String,
    pub id: i64,
    pub year: i64,
    pub number: i64,
    pub cifr: String
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.cifr, self.year, self.number)
    }
}

#[derive(Debug, Clone)]
pub(super) struct  FullName {
    pub first_name: String,
    pub second_name: String,
    pub middle_name: String
}

impl Display for FullName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.second_name, self.first_name, self.middle_name)
    }
}

#[derive(Debug, Clone)]
pub(super) struct Student {
    pub login: String,
    pub number: i32,
    pub full_name: FullName,
    pub group: Group
}

impl Student {
    pub fn new(login: &str, number: i32, full_name: FullName, group: Group) -> Self {
        Self {
            login: login.to_owned(),
            number,
            full_name,
            group
        }
    }
}

impl Display for Student {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.full_name, self.group)
    }
}

#[derive(Debug, Clone)]
pub(super) struct Teacher {
    pub login: String,
    pub subjects: Vec<i64>
}

impl Teacher {
    pub fn new(login: &str, subjects: Vec<i64>) -> Self {
        Self {
            login: login.to_owned(),
            subjects
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
            3 => {
                let login: &str = row[0].get(0);
                let query = format!("select * from get_student_info('{}');", login);
                let row = client.query(&query, &[])?;
                
                Ok(User::Student(Student::new(
                    login, 
                    row[0].get(0),
                    FullName {
                        first_name: row[0].get(1),
                        second_name: row[0].get(2),
                        middle_name: row[0].get(3),
                    },
                    Group {
                        id: row[0].get(4),
                        cifr: row[0].get(5),
                        year: row[0].get(6),
                        number: row[0].get(7),
                        specialization_id: row[0].get(8),
                        specialization_name: row[0].get(9),
                    }
                )))
            },
            4 => {
                let login: &str = row[0].get(0);
                Ok(User::Teacher(Teacher::new(login, vec![])))
            },
            _ => unreachable!()
        }
    }
}

//pub(super) fn register_user()