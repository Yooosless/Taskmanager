use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::sync::Mutex; 
use actix_web::Responder;
use tokio::fs;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Task {
    pub title: String,
    pub body: String,
    #[serde(default = "default_completed")]
    pub completed: bool,
    #[serde(default = "default_creation_date")]
    pub creation_date: String,
    pub completion_date: Option<String>,
}

fn default_completed() -> bool {
    true
}

fn default_creation_date() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

impl Task {
    pub fn new(title: String, body: String) -> Self {
        let creation_date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        Task {
            title,
            body,
            completed: true,
            creation_date,
            completion_date: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskManager {
    pub tasks: Vec<Task>,
}

impl TaskManager {
    pub fn default() -> Self {
        TaskManager { tasks: Vec::new() }
    }

    pub async fn from_file_path(file_path: &str) -> Result<TaskManager, Box<dyn std::error::Error>> {
        let path: PathBuf = PathBuf::from(file_path);
        
        if path.exists() {
            let file_contents = fs::read_to_string(path).await?;
            let task_manager: TaskManager = serde_json::from_str(&file_contents)?;
            Ok(task_manager)
        } else {
            Ok(TaskManager { tasks: Vec::new() })
        }
    }
    

    pub async fn save(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path: PathBuf = PathBuf::from(file_path);
        let json_string = serde_json::to_string_pretty(&self)?;
        
        let mut file = fs::File::create(path).await?;
        file.write_all(json_string.as_bytes()).await?;
        
        Ok(())
    }

    pub async fn remove_task(&mut self, index: usize) {
        if self.tasks.len() == 0 {
            println!("There are no tasks in this file.");
        } else if index < self.tasks.len() {
            self.tasks.remove(index);
            println!("Now there are {} tasks in the file.", self.tasks.len());
        } else {
            println!(
                "There are only {} tasks in the file. Can't delete something that doesn't exist.",
                self.tasks.len() - 1
            );
        }
    }

    pub async fn complete_task(&mut self, index: usize) {
        if index < self.tasks.len() {
            let task = &mut self.tasks[index];
            if task.completed {
                task.completed = false;

                task.completion_date =
                    Some(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());

                println!("Task completion status updated successfully.");
            } else {
                println!("Task at index {} is already marked as completed.", index);
            }
        } else {
            println!("Invalid index. No task found at index {}.", index);
        }
    }
}

async fn init_task_manager() -> TaskManager {
    let file_path = "C:\\Users\\Admin\\Desktop\\todo.txt";
    match TaskManager::from_file_path(file_path).await {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("Error reading the file: {:?}", e);
            TaskManager { tasks: Vec::new() }
        }
    }
}
#[get("/list/{i}")]
async fn list_tasks(path: web::Path<usize>) -> HttpResponse {
    let i = path.into_inner();

    match TaskManager::from_file_path("C:\\Users\\Admin\\Desktop\\todo.txt").await {
        Ok(content) => {
            let mut response_body = String::new();

            for (index, task) in content.tasks.iter().enumerate() {
                if i >= index {
                    let task_info = format!(
                        "Task {}: Title: {}, Body: {}, Completed: {}, created at: {}\n",
                        index, task.title, task.body, task.completed, task.creation_date
                    );
                    response_body.push_str(&task_info);
                }
            }

            HttpResponse::Ok().body(response_body)
        }
        Err(e) => {
            eprintln!("Error reading the file: {:?}", e);
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

#[post("/add")]
async fn add_task(task: web::Json<Task>, data: web::Data<Mutex<TaskManager>>) -> HttpResponse {
    println!(
        "Hi you want to add a task with a title {} and body {} and is {}",
        task.title, task.body, task.completed
    );

   let new_task = Task::new(task.title.clone(), task.body.clone());

   
    let mut task_manager = data.lock().unwrap();
    task_manager.tasks.push(new_task);

    if let Err(e) = task_manager.save("C:\\Users\\Admin\\Desktop\\todo.txt").await {
        eprintln!("Error saving the file: {:?}", e);
        return HttpResponse::InternalServerError().body("Internal Server Error");
    }
    HttpResponse::Ok().body("Task added successfully")
}

#[delete("/rm/{index}")]
async fn remove_task(index: web::Path<usize>,data:web::Data<Mutex<TaskManager>>) -> impl Responder {
    let mut task_manager = data.lock().unwrap();
    task_manager.remove_task(index.into_inner()).await;

    match task_manager.save("C:\\Users\\Admin\\Desktop\\todo.txt").await {
        Ok(_) => "Task removed successfully",
        Err(e) => {
            eprintln!("Error saving the file: {:?}", e);
            "Error removing task"
        }
    }
}
#[put("/complete/{index}")]
async fn complete(
    index: web::Path<usize>,
) -> Result<HttpResponse, actix_web::Error> {
    println!("Updating task completion status for index: {}", index);

    let mut task_manager =
        match TaskManager::from_file_path("C:\\Users\\Admin\\Desktop\\todo.txt").await {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading the file: {:?}", e);
                return Err(actix_web::error::ErrorInternalServerError(e));
            }
        };

        task_manager.complete_task(index.into_inner()).await;
 
    if let Err(e) = task_manager.save("C:\\Users\\Admin\\Desktop\\todo.txt").await {
        eprintln!("Error saving the file: {:?}", e);
        return Err(actix_web::error::ErrorInternalServerError(e));
    }

    Ok(HttpResponse::Ok().body("Task completion status updated successfully"))
}




#[get("/finished")]
async fn finished_tasks() -> impl Responder {
    let mut response = String::new();

    match TaskManager::from_file_path("C:\\Users\\Admin\\Desktop\\todo.txt").await {
        Ok(content) => {
            for (index, task) in content.tasks.iter().enumerate() {
                if let Some(completion_date) = &task.completion_date {
                    let task_info = format!(
                        "Task: {index}, title: {}, Completion Date: {}\n",
                        task.title,
                        completion_date
                    );
                    response.push_str(&task_info);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading the file: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    }

    HttpResponse::Ok().body(response)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tasks=init_task_manager().await;
    let task_manager =web::Data::new(Mutex::new(tasks));
    HttpServer::new(move|| {
        App::new()
        .app_data(task_manager.clone())
            .service(list_tasks)
            .service(add_task)
            .service(remove_task)
            .service(complete)
            .service(finished_tasks)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
