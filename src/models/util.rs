use serde_json::Value;
use utoipa::ToSchema;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use core::fmt;
use diesel_derive_enum;

#[derive(ToSchema, diesel_derive_enum::DbEnum, Debug, Clone, PartialEq, Serialize, Deserialize, Copy, Eq, Hash)]
#[ExistingTypePath = "crate::schema::sql_types::AccessType"]
pub enum AccessType {
    Read,
    Write,
    Other,
    Delete,
    Create,
}

impl ToString for AccessType {
    fn to_string(&self) -> String {
        match self {
            AccessType::Read   => String::from("Read"),
            AccessType::Write  => String::from("Write"),
            AccessType::Other  => String::from("Other"),
            AccessType::Delete => String::from("Delete"),
            AccessType::Create => String::from("Create"),
        }
    }
}

#[derive(ToSchema, diesel_derive_enum::DbEnum, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::TaskPackageType"]
pub enum TaskPackageType {
    Learning,
    Exam,
}

#[derive(ToSchema, diesel_derive_enum::DbEnum, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::Visibility"]
pub enum Visibility {
    Private,
    Public,
}

#[derive(ToSchema, diesel_derive_enum::DbEnum, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::State"]
pub enum State {
    Deleted,
    Active,
}

#[derive(ToSchema, diesel_derive_enum::DbEnum, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[ExistingTypePath = "crate::schema::sql_types::AnswerState"]
pub enum AnswerState {
    Deleted,
    Active,
    Done,
}

pub fn solution_state_from_str(state_from_string: &str) -> AnswerState {
    match state_from_string {
        "ACTIVE"    => AnswerState::Active,
        "DELETED"   => AnswerState::Deleted,
        "DONE"      => AnswerState::Done,
        _           => AnswerState::Deleted,
    } 
}

impl ToString for AnswerState {
    fn to_string(&self) -> String {
        match self {
            AnswerState::Active   => String::from("ACTIVE"),
            AnswerState::Deleted  => String::from("DELETED"),
            AnswerState::Done     => String::from("DONE"),
        }
    }
}

pub fn state_from_str(state_from_string: &str) -> State {
    match state_from_string {
        "ACTIVE"    => State::Active,
        "DELETED"   => State::Deleted,
        _           => State::Deleted,
    } 
}

impl ToString for State {
    fn to_string(&self) -> String {
        match self {
            State::Active => String::from("ACTIVE"),
            State::Deleted => String::from("DELETED")
        }
    }
}

#[derive(ToSchema, Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
pub enum OrderDir {
    ASC,
    DESC,
}

#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct ErrorSchema {
    pub message: String,
}

#[derive(ToSchema, Deserialize, Debug, Clone, Copy)]
pub struct PagingSchema {
    pub page: i32,
    pub limit: i32,
    pub order: OrderDir,
}

impl fmt::Display for ErrorSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

pub fn deserialize_option_vec_int<'de, D>(de: D) -> Result<Option<Vec<i32>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    
    // Versuche, eine Option<String> zu deserialisieren
    let s = Option::<String>::deserialize(de)?;

    // Verarbeite den String, falls er nicht None ist
    let values = match s {
        Some(s) => {
            // Zerlege die Zeichenfolge an den Kommas und parsiere die Teile zu i32
            let parsed_values: Result<Vec<i32>, _> = s.split(',')
                .map(|s| s.trim().parse::<i32>())
                .collect();

            match parsed_values {
                Ok(vals) => vals,
                Err(_) => return Err(Error::custom("Failed to parse integer values")),
            }
        }
        None => vec![], // Wenn der Wert None ist, ist die Sammlung leer
    };

    Ok(Some(values))
}

pub fn deserialize_option_vec_uuid<'de, D>(de: D) -> Result<Option<Vec<Uuid>>, D::Error>
where
    D: serde::Deserializer<'de>,
{   
    use serde::de::Error;

    let s = Option::<String>::deserialize(de)?;

    let values = match s {
        Some(s) => {
            let parsed_values: Result<Vec<Uuid>, _> = s.split(',')
                .map(|s| s.trim().parse::<Uuid>())
                .collect();

            match parsed_values {
                Ok(vals) => vals,
                Err(_) => return Err(Error::custom("Failed to parse Uuid values")),
            }
        }
        None => vec![], 
    };

    Ok(Some(values))
}

pub fn deserialize_option_vec_string<'de, D>(de: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{   
    let s = Option::<String>::deserialize(de)?;

    let values = match s {
        Some(s) => {
            s.split(',').map(|v| v.to_string()).collect::<Vec<String>>()
        }
        None => vec![], 
    };

    Ok(Some(values))
}

fn sort_json_object(obj: &Value) -> Value {
    match obj {
        Value::Object(map) => {
            let sorted_map: serde_json::Map<_, _> = map
                .iter()
                .map(|(k, v)| (k.clone(), sort_json_object(v)))
                .collect();
            Value::Object(sorted_map)
        }
        _ => obj.clone(),
    }
}

pub fn compare_json_objects_ignore_order(obj1: &Value, obj2: &Value) -> bool {
    let sorted_obj1 = sort_json_object(obj1);
    let sorted_obj2 = sort_json_object(obj2);

    sorted_obj1 == sorted_obj2
}