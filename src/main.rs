#![warn(clippy::all, clippy::pedantic)]
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use serde_json::json;
use serde_json::Value;

use chrono::Datelike;
use chrono::NaiveDate;
use chrono::Utc;

use strum_macros::Display;
use strum_macros::EnumString;

use colored::*;

use inquire::{
    formatter::MultiOptionFormatter,
    validator::Validation, Confirm, DateSelect, InquireError, MultiSelect, Select, Text,
};


#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumString)]
enum TaskStatus {
    Incomplete,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    name: String,
    description: String,
    status: TaskStatus,
    date_posted: String,
    date_deadline: String,
}

#[derive(Debug)]
struct TaskManager {
    task_list: Vec<Task>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self { task_list: vec![] }
    }

    pub fn get_tasklist(&self) -> Vec<Task> {
        let c = self.task_list.clone();
        c
    }

    fn load_file(&mut self) {
        let mut file = File::open("tasks.json").expect("Failed to load file.");

        let mut f_content = String::new();
        file.read_to_string(&mut f_content).unwrap();

        let data: Value = serde_json::from_str(&f_content).unwrap();

        self.task_list = serde_json::from_value(data["task-list"].clone()).unwrap_or_default();
    }

    fn check_savefile(&mut self) {
        let fpath = Path::new("tasks.json");

        if !fpath.exists() {
            File::create("tasks.json").unwrap();
            self.save_tofile();
        }
    }

    fn save_tofile(&mut self) {
        let mut savjson = json!({});

        savjson = json!({
            "task-list":self.task_list
        });

        let pretty_savjson = serde_json::to_string_pretty(&savjson).unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("tasks.json")
            .expect("WOW! This sure is a huge chaining!");

        file.write_all(&pretty_savjson.into_bytes()).unwrap();
    }

    fn save_task(&mut self, task: Task) {
        self.check_savefile();
        self.task_list.push(task);
        self.save_tofile();
    }

    fn load(&mut self) {
        self.check_savefile();
        self.load_file();
    }

    fn delete_tasks(&mut self, rm_list: Vec<String>) {
        for rm_name in rm_list.iter() {
            let _ = &self
                .task_list
                .retain(|task| if task.name.eq(rm_name) { false } else { true });
        }

        self.check_savefile();
        self.save_tofile();
    }


    fn get_days_diff(
        &self,
        naive_date_a: chrono::NaiveDate,
        naive_date_b: chrono::NaiveDate,
    ) -> i64 {
        let diff_duration = naive_date_a.signed_duration_since(naive_date_b);
        let days = diff_duration.num_days();
        days
    }
}

struct Interface {
    tm: TaskManager,
}

impl Interface {
    pub fn new() -> Self {
        Self {
            tm: TaskManager::new(),
        }
    }

    fn clear_screen(&self) {
        // !
        print!("\x1B[2J\x1B[1;1H");
    }

    fn task_create(&mut self) {
        const TITLE_CHAR_LIMIT: u8 = 30;
        const DESCRIPTION_CHAR_LIMIT: u8 = 100;

        let task_name = Text::new("Task Name:")
            .with_validator(|t: &str| {
                if t.len() > TITLE_CHAR_LIMIT.into() {
                    Ok(Validation::Invalid(
                        format!(
                            "Task name must be {} characters or less. Current: {}.",
                            TITLE_CHAR_LIMIT,
                            t.len()
                        )
                        .into(),
                    ))
                } else {
                    Ok(Validation::Valid)
                }
            })
            .prompt();

        let task_desc = Text::new("Task Description:")
            .with_validator(|t: &str| {
                if t.len() > DESCRIPTION_CHAR_LIMIT.into() {
                    Ok(Validation::Invalid(
                        format!(
                            "Task description must be {} characters or less. Current: {}.",
                            DESCRIPTION_CHAR_LIMIT,
                            t.len()
                        )
                        .into(),
                    ))
                } else {
                    Ok(Validation::Valid)
                }
            })
            .prompt();

        let current_status = TaskStatus::Incomplete;

        let task_date_posted = Utc::now().date_naive(); //Year-Month-Day

        //Pre-declare
        let mut deadline_choose: String = String::new();

        let deadline_ask = Confirm::new("Does the task have a deadline?")
            .with_default(true)
            .prompt();

        match deadline_ask {
            Ok(true) => {
                deadline_choose = DateSelect::new("Choose Task Deadline:")
                    .with_default(Utc::now().date_naive())
                    .with_min_date(Utc::now().date_naive())
                    .with_max_date(chrono::NaiveDate::from_ymd_opt(
                        Utc::now().date_naive().year().saturating_add(1),
                        12,
                        31,
                    ).unwrap())
                    .with_week_start(chrono::Weekday::Mon)
                    .with_help_message("Use Arrow Keys to move the cursor around")
                    .prompt()
                    .unwrap()
                    .to_string();
            }
            Ok(false) => {
                deadline_choose = "0000-00-00".to_string();
            }
            Err(e) => {
                panic!("{}",e);
            }
        }

        let n_task = Task {
            name: task_name.unwrap_or_default().to_string(),
            description: task_desc.unwrap_or_default().to_string(),
            status: current_status,
            date_posted: task_date_posted.to_string(),
            date_deadline: deadline_choose,
        };

        self.tm.save_task(n_task);
    }

    fn tasklist_print(&self) {
        let tlist = self.tm.get_tasklist();
        let mut str_tvec = vec![];

        for l in 0..tlist.len() {
            let mut days:i64 = 0;
            let mut tmp_deadline: ColoredString;

            if tlist[l].date_deadline.eq("0000-00-00") {
                tmp_deadline = tlist[l].date_deadline.clone().white();
            } else {
                let ymd_it = tlist[l].date_deadline.split("-");
                let ymd: Vec<&str> = ymd_it.collect();

                //Compare Time to get color
                days = self
                    .tm
                    .get_days_diff(
                        NaiveDate::from_ymd_opt(
                            ymd[0].parse::<i32>().unwrap(),
                            ymd[1].parse::<u32>().unwrap(),
                            ymd[2].parse::<u32>().unwrap(),
                        )
                        .unwrap(),
                        Utc::now().date_naive()
                    );

                tmp_deadline = tlist[l].date_deadline.clone().white();

                //Days to deadline
                if days >= 7 {
                    tmp_deadline = tlist[l].date_deadline.clone().green();
                } else if days < 7 && days > 0 {
                    tmp_deadline = tlist[l].date_deadline.clone().yellow();
                } else if days == 0 {
                    tmp_deadline = tlist[l].date_deadline.clone().red();
                } else if days < 0 {
                    tmp_deadline = tlist[l].date_deadline.clone().magenta();
                }

            }

            let tmp_complete_string: String = format!(
                "Name: {} \n Description: {} \n Status: {}\n Date Posted: {}\n Deadline: {}\n",
                tlist[l].name.clone(),
				tlist[l].description.clone(),
                tlist[l].status.clone(),
                tlist[l].date_posted.clone(),
                tmp_deadline
            );

            str_tvec.push(tmp_complete_string);
        }

        println!("");
        for i in 0..str_tvec.len() {
            println!("{}", str_tvec[i]);
        }
        println!("");
    }

    fn tasklist_remove(&mut self) {
        let tasklist_ref = self.tm.get_tasklist();

        let mut t_options = vec![];

        for l in 0..tasklist_ref.len() {
            t_options.push(format!("{}", tasklist_ref[l].name.clone()));
        }

        if tasklist_ref.len().eq(&0) {
            println!("Task List Empty!");
        } else if tasklist_ref.len() > 0 {
            let formatter: MultiOptionFormatter<String> =
                &|tasks| format!("Selected {} tasks", tasks.len());

            let rm_selection = MultiSelect::new("Select which entries to remove", t_options)
                .with_help_message("↑↓ to move, space to select one, → to all, ← to none, type to filter, enter to confirm")
                .with_formatter(formatter)
                .prompt();

            let rm_vec = rm_selection.unwrap_or_default();

            self.tm.delete_tasks(rm_vec);
            
        }
    }

    pub fn run(&mut self) {
        self.tm.load();

        let version = env!("CARGO_PKG_VERSION");
        print!("Forgetful Me Ver. - {}\n", version);
        println!("A simple task reminder software.\n");

        'm_loop: loop {
            let options_hash: HashMap<u8, &str> = HashMap::from([
                (0, "Add Task"),
                (1, "Remove Task"),
                (2, "View Task List"),
                (3, "Quit"),
            ]);

            let menu_options = vec![
                options_hash[&0],
                options_hash[&1],
                options_hash[&2],
                options_hash[&3],
            ];

            let menu_answer: Result<&str, InquireError> =
                Select::new("Choose an action:", menu_options).prompt();

            match menu_answer {
                Ok(choice) => {
                    if options_hash[&0].eq(choice) {
                        //ADD
                        self.task_create();
                        self.clear_screen();
                    } else if options_hash[&1].eq(choice) {
                        //REMOVE
                        self.tasklist_remove();
                        self.clear_screen();
                    } else if options_hash[&2].eq(choice) {
                        //VIEW
                        self.clear_screen();
                        self.tasklist_print();
                    } else if options_hash[&3].eq(choice) {
                        //QUIT
                        break 'm_loop;
                    } else {
                        panic!();
                    }
                }

                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
        self.clear_screen();
    }
}

fn main() {
    let mut interface = Interface::new();
    interface.run();
}

