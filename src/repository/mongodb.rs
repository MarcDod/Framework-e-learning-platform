// Documentation was created by ChatGPT
use std::io::{self, Write};

use chrono::NaiveDateTime;
use mongodb::{options::FindOptions, error::Error, Client, Collection, bson::{doc, Document, Bson, DateTime}, results::DeleteResult};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{models::{task::{TaskDoc, TaskDocIdString}, util::{PagingSchema, OrderDir, State, state_from_str}, answer::{AnswerDocIdWithString, AnswerDoc}}};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaDocWithIdString {
    #[serde(rename = "_id")]
    pub id: String,
    pub task_type: String,
    pub task_schema: Document,
    pub solution_schema: Document,
}

#[derive(ToSchema, Serialize, Deserialize, Debug, Clone)]
pub struct SchemaDoc {
    pub id: Uuid,
    pub task_type: String,
    pub task_schema: Document,
    pub solution_schema: Document,
}


#[derive(Clone)]
pub struct MongoDbRepo {
    tasks: Collection<TaskDocIdString>,
    solutions: Collection<AnswerDocIdWithString>,
    schemas: Collection<SchemaDocWithIdString>,
}

impl MongoDbRepo {
    pub async fn establish_connection(database_url: &str, database_name: &str) -> Self {
        let client = Client::with_uri_str(database_url)
            .await
            .expect("Error while connection Mongodb");
        let db = client.database(database_name);
        let tasks: Collection<TaskDocIdString> = db.collection("Task");
        let solutions: Collection<AnswerDocIdWithString> = db.collection("Solution");
        let schemas: Collection<SchemaDocWithIdString> = db.collection("Schemas");
        MongoDbRepo { tasks, solutions, schemas }
    }


    /// Asynchronously creates a new answer document with the provided solution.
    ///
    /// # Arguments
    ///
    /// * `solution` - A `Document` representing the solution data to be stored in the answer document.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the newly created `AnswerDoc` or an `Error` if the operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use your_crate_name::YourStructName;
    /// use bson::Document; // Ensure you import the correct BSON type.
    ///
    /// let your_struct_instance = YourStructName::new(); // Replace with your actual struct name and instance.
    /// let example_solution = Document::new(); // Replace with your actual solution data.
    ///
    /// match your_struct_instance.create_answer_doc(example_solution).await {
    ///     Ok(answer_doc) => {
    ///         println!("Created answer document: {:?}", answer_doc);
    ///         // Handle the created answer document
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Error creating answer document: {}", err);
    ///         // Handle the error
    ///     }
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if there is an issue with the MongoDB query execution or connection.
    pub async fn create_answer_doc(&self, solution: Document) -> Result<AnswerDoc, Error> {
        let new_doc = AnswerDocIdWithString { 
            id: Uuid::new_v4().to_string(), 
            solution: solution.clone(),
            updated_at: DateTime::now(),
        };
        let insert_result = self.solutions.insert_one(&new_doc, None).await;

        match insert_result {
            Ok(_) => Ok(AnswerDoc { 
                id: Uuid::parse_str(&new_doc.id).unwrap(), 
                solution, 
                updated_at: NaiveDateTime::from_timestamp_millis(new_doc.updated_at.timestamp_millis()).unwrap(),
            }),
            Err(err) => Err(err)
        }
    }

    pub async fn create_schema_doc(&self, task_schema: Document, solution_schema: Document, task_type: String) -> Result<SchemaDoc, Error> {
        let new_doc = SchemaDocWithIdString {
            id: Uuid::new_v4().to_string(),
            task_type: task_type.to_string(),
            task_schema: task_schema.clone(),
            solution_schema: solution_schema.clone(),
        };

        let insert_result = self.schemas.insert_one(&new_doc, None).await;

        match insert_result {
            Ok(_) => Ok(SchemaDoc {
                id: Uuid::parse_str(&new_doc.id).unwrap(),
                task_schema,
                task_type,
                solution_schema,
            }),
            Err(err) => Err(err)
        }
    }

    /// Asynchronously updates a solution in the MongoDB collection.
    ///
    /// This function takes a `Document` containing the updated solution data and the unique identifier (`Uuid`) of the solution to be updated. It updates the solution in the MongoDB collection and returns a `Result` indicating the success or failure of the operation.
    ///
    /// # Arguments
    ///
    /// - `solution`: A `Document` containing the updated solution data to be stored in the MongoDB collection.
    /// - `solution_id`: A `Uuid` representing the unique identifier of the solution to be updated.
    ///
    /// # Returns
    ///
    /// A `Result` indicating the success or failure of the solution update. If successful, it returns a `Solution` struct containing the unique identifier, updated solution data, and the timestamp of the update.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the MongoDB connection, if the specified solution does not exist, or if there is an error during the solution update process.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let db_conn = MongoDbRepo::establish_connection("database_url", "database_name").await;
    ///
    ///     // Example solution data
    ///     let updated_solution_data = Document::new(); // Replace this with your actual updated solution data.
    ///
    ///     // Example solution ID
    ///     let solution_id = Uuid::new_v4(); // Replace this with your actual solution ID.
    ///
    ///     match db_conn.update_solution(updated_solution_data, solution_id).await {
    ///         Ok(updated_solution) => {
    ///             println!("Updated Solution ID: {}", updated_solution.id);
    ///             println!("Updated At: {}", updated_solution.updated_at);
    ///             // Additional code to handle the updated solution.
    ///         },
    ///         Err(err) => eprintln!("Error: {}", err),
    ///     }
    /// }
    /// ```
    ///
    /// In this example, the function is used to asynchronously update a solution in the MongoDB collection, print the updated solution's unique identifier and timestamp, and perform additional actions based on the updated solution.
    pub async fn update_answer(&self, solution: Document, solution_id: Uuid) -> Result<AnswerDoc, Error> {
        let updated_at = DateTime::now();
        let update_result = self.solutions.update_one(
            doc!{ "_id": solution_id.to_string() },
            doc!{ "$set": { "solution": solution.clone(), "updated_at": updated_at } }, None).await;
        
        match update_result {
            Ok(_) => Ok(AnswerDoc {
                id: solution_id,
                solution,
                updated_at: NaiveDateTime::from_timestamp_millis(updated_at.timestamp_millis()).unwrap(),
            }),
            Err(err) => {
                print!("{}", err);
                io::stdout().flush().unwrap();
                return Err(err)
            },
        }
    }

    /// Asynchronously creates a new task in the MongoDB collection.
    ///
    /// This function takes a `task_type` as a `String`, a `task` as a `Document`, and a `solution` as a `Document`. It creates a new task document in the MongoDB collection and returns a `Result` indicating the success or failure of the operation.
    ///
    /// # Arguments
    ///
    /// - `task_type`: A `String` representing the type of the task.
    /// - `task`: A `Document` containing the task data to be stored in the MongoDB collection.
    /// - `solution`: A `Document` containing the associated solution data to be stored in the MongoDB collection.
    ///
    /// # Returns
    ///
    /// A `Result` indicating the success or failure of the task creation. If successful, it returns a `Task` struct containing the unique identifier, task type, task data, and the state of the task.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the MongoDB connection or if there is an error during the task creation process.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let db_conn = MongoDbRepo::establish_connection("database_url", "database_name").await;
    ///
    ///     // Example task type
    ///     let task_type = "ExampleTaskType".to_string(); // Replace this with your actual task type.
    ///
    ///     // Example task data
    ///     let task_data = Document::new(); // Replace this with your actual task data.
    ///
    ///     // Example solution data
    ///     let solution_data = Document::new(); // Replace this with your actual solution data.
    ///
    ///     match db_conn.create_task(task_type, task_data, solution_data).await {
    ///         Ok(new_task) => {
    ///             println!("New Task ID: {}", new_task.id);
    ///             println!("Task Type: {}", new_task.task_type);
    ///             println!("Task State: {:?}", new_task.state);
    ///             // Additional code to handle the new task.
    ///         },
    ///         Err(err) => eprintln!("Error: {}", err),
    ///     }
    /// }
    /// ```
    ///
    /// In this example, the function is used to asynchronously create a new task in the MongoDB collection, print the new task's unique identifier, task type, and state, and perform additional actions based on the new task.
    pub async fn create_task(&self, task_type: String, task: Document, solution: Document) -> Result<TaskDoc, Error> {
        let new_doc = TaskDocIdString {
            id: Uuid::new_v4().to_string(),
            task_type,
            task,
            state: State::Active.to_string(),
            solution
        };
        let insert_result = self.tasks.insert_one(&new_doc, None).await;

        match insert_result {
            Ok(_) => Ok(TaskDoc {
                id: Uuid::parse_str(&new_doc.id).unwrap(),
                task_type: new_doc.task_type,
                task: new_doc.task,
                state: state_from_str(&new_doc.state),
                solution: new_doc.solution,
            }),
            Err(err) => Err(err),
        }
    }

    /// Asynchronously fetches the schema associated with the provided task type.
    ///
    /// # Arguments
    ///
    /// * `task_type` - A `String` representing the task type for which the schema is to be fetched.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing an `Option` with the fetched `SchemaDoc` or `None` if no schema is found.
    /// Returns an `Error` if the operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use your_crate_name::YourStructName;
    /// let your_struct_instance = YourStructName::new(); // Replace with your actual struct name and instance.
    /// let task_type = "example_task_type".to_string();
    ///
    /// match your_struct_instance.fetch_schema(task_type).await {
    ///     Ok(Some(schema)) => {
    ///         println!("Fetched schema: {:?}", schema);
    ///         // Handle the fetched schema
    ///     }
    ///     Ok(None) => {
    ///         println!("No schema found for the specified task type.");
    ///         // Handle the case where no schema is found
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Error fetching schema: {}", err);
    ///         // Handle the error
    ///     }
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if there is an issue with the MongoDB query execution or connection.
    pub async fn fetch_schema(&self, task_type: String) -> Result<Option<SchemaDoc>, Error> {
        let filter_doc = doc!{ "task_type": task_type.to_string() };

        match self.schemas.find_one(filter_doc, None).await {
            Ok(Some(schema)) => Ok(Some(
                SchemaDoc {
                    id: Uuid::parse_str(&schema.id).unwrap(),
                    task_schema: schema.task_schema,
                    task_type: schema.task_type,
                    solution_schema: schema.solution_schema,
                }
            )),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }

    }

    pub async fn fetch_task(&self, task_id: Uuid, only_active: bool) -> Result<Option<TaskDoc>, Error> {
        let mut filter_doc = doc!{ "_id": task_id.to_string() };

        if only_active {
            filter_doc = filter_doc.into_iter().chain(doc!{ "state": State::Active.to_string() }).collect()
        }
        match self.tasks.find_one(filter_doc, None).await {
            Ok(task) => {
                if let Some(task) = task {
                    Ok(Some(TaskDoc {
                        id: Uuid::parse_str(&task.id).unwrap(),
                        task_type: task.task_type,
                        task: task.task,
                        state: state_from_str(&task.state),
                        solution: task.solution,
                    }))
                } else {
                    Ok(None)
                }
            },
            Err(err) => Err(err),
        }
    }

    /// Asynchronously retrieves a task from the MongoDB collection based on the provided task ID and optional filter for active tasks.
    ///
    /// This function takes a `task_id` of type `Uuid` and an `only_active` boolean indicating whether to filter for active tasks only. It fetches a task document from the MongoDB collection based on the provided task ID and, if specified, the active state filter. The function returns a `Result` indicating the success or failure of the operation.
    ///
    /// # Arguments
    ///
    /// - `task_id`: A `Uuid` representing the unique identifier of the task to be fetched.
    /// - `only_active`: A boolean indicating whether to filter for active tasks only.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Option` representing the fetched task. If the task is found, it returns `Some(Task)` with details such as the unique identifier, task type, task data, and the state of the task. If the task is not found or an error occurs, it returns `None` or an `Error` respectively.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the MongoDB connection or if there is an error during the task retrieval process.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let db_conn = MongoDbRepo::establish_connection("database_url", "database_name").await;
    ///
    ///     // Example task ID
    ///     let task_id = Uuid::new_v4(); // Replace this with your actual task ID.
    ///
    ///     // Example flag indicating whether to fetch only active tasks
    ///     let only_active = true; // Replace this with your actual value.
    ///
    ///     match db_conn.fetch_task(task_id, only_active).await {
    ///         Ok(Some(fetched_task)) => {
    ///             println!("Fetched Task ID: {}", fetched_task.id);
    ///             println!("Task Type: {}", fetched_task.task_type);
    ///             println!("Task State: {:?}", fetched_task.state);
    ///             // Additional code to handle the fetched task.
    ///         },
    ///         Ok(None) => println!("Task not found."),
    ///         Err(err) => eprintln!("Error: {}", err),
    ///     }
    /// }
    /// ```
    ///
    /// In this example, the function is used to asynchronously fetch a task from the MongoDB collection based on the task ID and filter for active tasks. It prints details of the fetched task if it exists, prints a message if the task is not found, and handles errors.
    pub async fn fetch_amount_tasks(&self, only_active: bool) -> Result<u64, Error> {
        let mut filter_doc = doc!{ };

        if only_active {
            filter_doc = filter_doc.into_iter().chain(doc!{ "state": State::Active.to_string() }).collect()
        }

        self.tasks.count_documents(filter_doc, None).await
    }

    pub async fn fetch_amount_schemas(&self) -> Result<u64, Error> {
        let filter_doc = doc!{ };

        self.schemas.count_documents(filter_doc, None).await
    }

    /// Asynchronously retrieves a list of tasks from the MongoDB collection based on the provided pagination settings, optional task IDs, and an active state filter.
    ///
    /// This function takes a `pagination` parameter of type `PagingSchema` to define the page, limit, and ordering settings. The `task_ids` parameter is an optional vector of `Uuid` representing specific task IDs to retrieve. The `only_active` parameter is a boolean indicating whether to filter for active tasks only. The function returns a `Result` indicating the success or failure of the operation.
    ///
    /// # Arguments
    ///
    /// - `pagination`: A reference to a `PagingSchema` struct containing page, limit, and ordering settings.
    /// - `task_ids`: An optional vector of `Uuid` representing specific task IDs to retrieve.
    /// - `only_active`: A boolean indicating whether to filter for active tasks only.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `Task` representing the fetched tasks. If tasks are found, it returns `Ok(Vec<Task>)` with details such as the unique identifier, task type, task data, and the state of each task. If no tasks are found or an error occurs, it returns an empty vector or an `Error` respectively.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the MongoDB connection or if there is an error during the task retrieval process.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let db_conn = MongoDbRepo::establish_connection("database_url", "database_name").await;
    ///
    ///     // Example pagination settings
    ///     let pagination = PagingSchema {
    ///         page: 1,           // Replace this with your actual page number.
    ///         limit: 10,         // Replace this with your actual limit.
    ///         order: OrderDir::ASC, // Replace this with your actual ordering direction.
    ///     };
    ///
    ///     // Example optional vector of task IDs
    ///     let task_ids = Some(vec![Uuid::new_v4()]); // Replace this with your actual task IDs or None.
    ///
    ///     // Example flag indicating whether to fetch only active tasks
    ///     let only_active = true; // Replace this with your actual value.
    ///
    ///     match db_conn.fetch_all_tasks(&pagination, task_ids, only_active).await {
    ///         Ok(fetched_tasks) => {
    ///             for task in fetched_tasks {
    ///                 println!("Fetched Task ID: {}", task.id);
    ///                 println!("Task Type: {}", task.task_type);
    ///                 println!("Task State: {:?}", task.state);
    ///                 // Additional code to handle each fetched task.
    ///             }
    ///         },
    ///         Err(err) => eprintln!("Error: {}", err),
    ///     }
    /// }
    /// ```
    ///
    /// In this example, the function is used to asynchronously fetch a list of tasks from the MongoDB collection based on the provided pagination settings, optional task IDs, and an active state filter. It prints details of each fetched task if any are found and handles errors.
    pub async fn fetch_all_tasks(&self, pagination: &PagingSchema, task_ids: Option<Vec<Uuid>>, only_active: bool) -> Result<Vec<TaskDoc>, Error> {
        let start_position = pagination.page * pagination.limit;
        let order_dir = if pagination.order == OrderDir::ASC { 1 } else { -1 };

        let mut filter_doc = match task_ids {
            Some(v) => doc!{ 
                "_id": { "$in": v.into_iter().map(|id| Bson::String(id.to_string())).collect::<Vec<Bson>>() }, 
                
            },
            None => doc!{},
        };

        if only_active {
            filter_doc = filter_doc.into_iter().chain(doc!{ "state": State::Active.to_string() }).collect()
        }


        let find_result = self.tasks.find(filter_doc, FindOptions::builder().limit(i64::from(pagination.limit)).skip(start_position as u64).sort(doc!{
            "task_type": order_dir
        }).build()).await;

        match find_result {
            Ok(mut cursor) => {
                let mut ret_vec = vec![];

                while cursor.advance().await? {
                    let task = cursor.deserialize_current().unwrap();
                    ret_vec.push(TaskDoc {
                        id: Uuid::parse_str(&task.id).unwrap(),
                        task_type: task.task_type,
                        task: task.task,
                        state: state_from_str(&task.state),
                        solution: task.solution,
                    })
                }

                Ok(ret_vec)
            },
            Err(err) => Err(err),
        }
    }

    pub async fn fetch_all_schemas(&self, pagination: &PagingSchema, task_types: Option<Vec<String>>) -> Result<Vec<SchemaDoc>, Error> {
        let start_position = pagination.page * pagination.limit;
        let order_dir = if pagination.order == OrderDir::ASC { 1 } else { -1 };

        let mut filter_doc = match task_types {
            Some(v) => doc!{ 
                "task_type": { "$in": v.into_iter().map(|task_type| Bson::String(task_type.to_string())).collect::<Vec<Bson>>() }, 
                
            },
            None => doc!{},
        };


        let find_result = self.schemas.find(filter_doc, FindOptions::builder().limit(i64::from(pagination.limit)).skip(start_position as u64).sort(doc!{
            "task_type": order_dir
        }).build()).await;

        match find_result {
            Ok(mut cursor) => {
                let mut ret_vec = vec![];

                while cursor.advance().await? {
                    let schema_doc = cursor.deserialize_current().unwrap();
                    ret_vec.push(SchemaDoc {
                        id: Uuid::parse_str(&schema_doc.id).unwrap(),
                        task_type: schema_doc.task_type,
                        task_schema: schema_doc.task_schema,
                        solution_schema: schema_doc.solution_schema
                    })
                }

                Ok(ret_vec)
            },
            Err(err) => Err(err),
        }
    }

    /// Asynchronously marks a task as deleted in the MongoDB collection based on the provided task ID.
    ///
    /// This function takes a `task_id` parameter of type `Uuid` representing the unique identifier of the task to delete. The function returns a `Result` indicating the success or failure of the operation.
    ///
    /// # Arguments
    ///
    /// - `task_id`: A reference to a `Uuid` representing the unique identifier of the task to delete.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `u64` representing the count of modified documents. If the task is successfully marked as deleted, it returns `Ok(u64)` with the count of modified documents (should be 1). If the task is not found or an error occurs, it returns an `Error`.
    ///
    /// # Errors
    ///
    /// This function may return an error if there is an issue with the MongoDB connection or if there is an error during the task deletion process.
    ///
    /// # Example
    ///
    /// ```rust
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let db_conn = MongoDbRepo::establish_connection("database_url", "database_name").await;
    ///
    ///     // Example task ID
    ///     let task_id = Uuid::new_v4(); // Replace this with your actual task ID.
    ///
    ///     match db_conn.delete_task(&task_id).await {
    ///         Ok(modified_count) => {
    ///             if modified_count > 0 {
    ///                 println!("Task with ID {} marked as deleted.", task_id);
    ///             } else {
    ///                 println!("Task with ID {} not found or already deleted.", task_id);
    ///             }
    ///         },
    ///         Err(err) => eprintln!("Error: {}", err),
    ///     }
    /// }
    /// ```
    ///
    /// In this example, the function is used to asynchronously mark a task as deleted in the MongoDB collection based on the provided task ID. It prints a message indicating whether the task was marked as deleted or not and handles errors.
    pub async fn delete_task(&self, task_id: &Uuid) -> Result<u64, Error> {
        let update_result = self.tasks.update_one(
            doc!{ "_id": task_id.to_string() },
            doc!{ "$set": { "state": State::Deleted.to_string() }},
            None).await;

        match update_result {
            Ok(task) => Ok(task.modified_count),
            Err(err) => Err(err),
        }
    }

    /// Asynchronously deletes a solution from the database based on the provided solution ID.
    ///
    /// # Arguments
    ///
    /// * `solution_id` - A reference to the UUID representing the solution ID to be deleted.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the deletion result or an `Error` if the operation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use your_crate_name::YourStructName;
    /// use uuid::Uuid;
    ///
    /// let your_struct_instance = YourStructName::new(); // Replace with your actual struct name and instance.
    /// let solution_id = Uuid::new_v4();
    ///
    /// match your_struct_instance.delete_solution(&solution_id).await {
    ///     Ok(delete_result) => {
    ///         println!("Deleted solution. Deletion result: {:?}", delete_result);
    ///         // Handle the successful deletion
    ///     }
    ///     Err(err) => {
    ///         eprintln!("Error deleting solution: {}", err);
    ///         // Handle the error
    ///     }
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if there is an issue with the MongoDB query execution or connection.
    pub async fn delete_solution(&self, solution_id: &Uuid) -> Result<DeleteResult, Error> {
        let delete_result = self.tasks.delete_one(
            doc!{ "_id": solution_id.to_string() },
            None).await;

        match delete_result {
            Ok(deleteResult) => Ok(deleteResult),
            Err(err) => Err(err),
        }
    }

    pub async fn fetch_answer_doc(&self, solution_id: Uuid) -> Result<Option<AnswerDoc>, Error> {
        let fetch_result = self.solutions.find_one(
            doc!{ "_id": solution_id.to_string() }, None).await;
        
        match fetch_result {
            Ok(Some(solution)) => Ok(Some(AnswerDoc {
                id: solution_id,
                solution: solution.solution,
                updated_at: NaiveDateTime::from_timestamp_millis(solution.updated_at.timestamp_millis()).unwrap(),
            })),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    #[cfg(test)]
    pub async fn clear_db(&self) {
        self.tasks.drop(None).await;
        self.schemas.drop(None).await;
        self.solutions.drop(None).await;
    }
}
