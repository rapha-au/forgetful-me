use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

use strum_macros::Display;
use strum_macros::EnumString;

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString, PartialEq)]
pub enum TaskStatus {
    Incomplete,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    pub date_posted: String,
    pub date_deadline: String,
}

#[derive(Debug)]
pub struct TaskManager {
    task_list: Vec<Task>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self { task_list: vec![] }
    }

    pub fn is_first_task(&mut self) -> bool {
        self.check_savefile();
        self.load_file();
        if self.get_tasklist().len() == 0 {
            return true;
        }
        return false;
    }

    pub fn get_last_id(&mut self) -> u32 {
        let mut bigger = 0;
        for t in self.task_list.iter() {
            if t.id > bigger {
                bigger = t.id
            }
        }
        bigger
    }

    pub fn get_id_from_str(&mut self, lines: Vec<String>) -> Vec<u32> {
        let mut id_vec: Vec<u32> = vec![];
        let mut id_str = String::from("");

        for l in lines {
            for c in l.chars() {
                if c.is_numeric() {
                    id_str.push(c.clone());
                }

                if c.eq(&'\n') {
                    break;
                }
            }
            let str_to_u: u32 = id_str.parse().unwrap();
            id_vec.push(str_to_u);
            id_str = "".to_string();
        }
        id_vec
    }

    fn update_ids(&mut self) {
        for i in 0..self.task_list.len() {
            self.task_list[i].id = i as u32;
        }
    }

    pub fn get_tasklist(&self) -> Vec<Task> {
        let c = self.task_list.clone();
        c
    }

    fn get_path(&mut self) -> String {
        let mut f = env::current_exe().expect("Couldn't find exe");
        f.pop();
        f.push("tasks.json");
        f.into_os_string().into_string().unwrap()
    }

    fn load_file(&mut self) {
        let p = self.get_path();
        let mut file = File::open(p).expect("Failed to load file.");

        let mut f_content = String::new();
        file.read_to_string(&mut f_content).unwrap();

        let data: Value = serde_json::from_str(&f_content).unwrap();

        self.task_list = serde_json::from_value(data["task-list"].clone()).unwrap_or_default();
    }

    fn check_savefile(&mut self) {
        let p = self.get_path();

        let fpath = Path::new(&p);

        if !fpath.exists() {
            File::create(fpath).unwrap();
            self.save_tofile();
        }
    }

    fn save_tofile(&mut self) {
        let p = self.get_path();
        let savjson: serde_json::Value;

        savjson = json!({
            "task-list":self.task_list
        });

        let pretty_savjson = serde_json::to_string_pretty(&savjson).unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(p)
            .expect("Err");

        file.write_all(&pretty_savjson.into_bytes()).unwrap();
    }

    pub fn save_task(&mut self, task: Task) {
        self.check_savefile();
        self.task_list.push(task);
        self.save_tofile();
    }

    pub fn load(&mut self) {
        self.check_savefile();
        self.load_file();
    }

    pub fn delete_tasks(&mut self, rm_list: Vec<u32>) {
        for id_num in rm_list.iter() {
            let _ = &self.task_list.retain(|task| {
                if task.id.eq(id_num) == true {
                    return false;
                }
                return true;
            });
        }

        self.update_ids();

        self.check_savefile();
        self.save_tofile();
    }

    pub fn switch_task_status(&mut self, switch_list: Vec<u32>) {
        for id_num in switch_list.iter() {
            if self.task_list[*id_num as usize].id.eq(id_num) == true {
                if self.task_list[*id_num as usize].status == TaskStatus::Complete {
                    self.task_list[*id_num as usize].status = TaskStatus::Incomplete
                } else if self.task_list[*id_num as usize].status == TaskStatus::Incomplete {
                    self.task_list[*id_num as usize].status = TaskStatus::Complete
                }
            }
        }

        self.update_ids();

        self.check_savefile();
        self.save_tofile();
    }

    pub fn get_days_diff(
        &self,
        naive_date_a: chrono::NaiveDate,
        naive_date_b: chrono::NaiveDate,
    ) -> i64 {
        let diff_duration = naive_date_a.signed_duration_since(naive_date_b);
        let days = diff_duration.num_days();
        days
    }
}
