use postgres::{Client, NoTls};
use slint::SharedString;
use tables::group;
use super::CONNECTION;
use super::*;
use std::{fmt::{format, write, Display}, hash::{DefaultHasher, Hash, Hasher}};

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
    pub subjects: Vec<SharedString>
}

impl Teacher {
    pub fn new(login: &str, subjects: Vec<SharedString>) -> Self {
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
    let mut client = Client::connect(CONNECTION, NoTls).unwrap();

    let hashed_password = get_hashed_password(password);

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
                let query = format!("select * from get_academic_plan_for_teacher('{login}')");
                let mut subjects: Vec<SharedString> = vec![];
                for row in client.query(&query, &[])? {
                    let id: i64 = row.get(0);
                    let spec_name: &str = row.get(1);
                    let sub_name: &str = row.get(2);
                    let cert_type: &str = row.get(3);
                    let hours: i64 = row.get(4);
                    let semester: i64 = row.get(5);
                    
                    subjects.push(
                        format!("{id}; {spec_name}; {sub_name}; {cert_type}; {hours}; {semester}")
                            .as_str()
                            .into()
                    );
                }
                Ok(User::Teacher(Teacher::new(login, subjects)))
            },
            _ => unreachable!()
        }
    }
}

//pub(super) fn register_user()

pub fn get_hashed_password(password: &str) -> String {
    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    hasher.finish().to_string()
}