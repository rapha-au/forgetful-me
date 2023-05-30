use inquire::{
    formatter::MultiOptionFormatter, validator::Validation, Confirm, DateSelect, InquireError,
    MultiSelect, Select, Text,
};

use std::collections::HashMap;

use chrono::Datelike;
use chrono::NaiveDate;
use chrono::Utc;

use colored::*;

use crate::Task;
use crate::TaskManager;
use crate::TaskStatus;

pub struct Interface {
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
        let deadline_choose: String;

        let deadline_ask = Confirm::new("Does the task have a deadline?")
            .with_default(true)
            .prompt();

        match deadline_ask {
            Ok(true) => {
                deadline_choose = DateSelect::new("Choose Task Deadline:")
                    .with_default(Utc::now().date_naive())
                    .with_min_date(Utc::now().date_naive())
                    .with_max_date(
                        chrono::NaiveDate::from_ymd_opt(
                            Utc::now().date_naive().year().saturating_add(1),
                            12,
                            31,
                        )
                        .unwrap(),
                    )
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
                panic!("{}", e);
            }
        }

        let mut n_id = 0;
        if self.tm.is_first_task() == false {
            n_id = self.tm.get_last_id() + 1;
        }

        let n_task = Task {
            id: n_id,
            name: task_name.unwrap_or_default().to_string(),
            description: task_desc.unwrap_or_default().to_string(),
            status: current_status,
            date_posted: task_date_posted.to_string(),
            date_deadline: deadline_choose,
        };

        self.tm.save_task(n_task);
    }

    pub fn get_colored_tasks(&mut self) -> HashMap<&str, u128> {
        let mut colored_tasks: HashMap<&str, u128> = Default::default();
        colored_tasks.insert("GREEN", 0);
        colored_tasks.insert("YELLOW", 0);
        colored_tasks.insert("RED", 0);
        colored_tasks.insert("MAGENTA", 0);

        let tlist = self.tm.get_tasklist();

        for l in 0..tlist.len() {
            if tlist[l].status == TaskStatus::Incomplete {
                let days: i64;
                let mut tmp_deadline: ColoredString;
                if tlist[l].date_deadline.eq("0000-00-00") {
                    tmp_deadline = tlist[l].date_deadline.clone().white();
                } else {
                    let ymd_it = tlist[l].date_deadline.split("-");
                    let ymd: Vec<&str> = ymd_it.collect();

                    days = self.tm.get_days_diff(
                        NaiveDate::from_ymd_opt(
                            ymd[0].parse::<i32>().unwrap(),
                            ymd[1].parse::<u32>().unwrap(),
                            ymd[2].parse::<u32>().unwrap(),
                        )
                        .unwrap(),
                        Utc::now().date_naive(),
                    );
                    tmp_deadline = tlist[l].date_deadline.clone().white();
                    //Days to deadline
                    if days >= 7 {
                        colored_tasks.entry("GREEN").and_modify(|t| {
                            *t = *t + 1;
                        });
                    } else if days < 7 && days > 0 {
                        colored_tasks.entry("YELLOW").and_modify(|t| {
                            *t = *t + 1;
                        });
                    } else if days == 0 {
                        colored_tasks.entry("RED").and_modify(|t| {
                            *t = *t + 1;
                        });
                    } else if days < 0 {
                        colored_tasks.entry("MAGENTA").and_modify(|t| {
                            *t = *t + 1;
                        });
                    }
                }
            }
        }

        colored_tasks
    }

    pub fn get_complete_tasks(&mut self) -> usize {
        let mut complete_tasks: usize = 0;
        let tlist = self.tm.get_tasklist();
        for task in tlist.iter() {
            if task.status == TaskStatus::Complete {
                complete_tasks += 1;
            }
        }
        complete_tasks
    }
    
    pub fn get_incomplete_tasks(&mut self) -> usize {
        let mut incomplete_tasks: usize = 0;
        let tlist = self.tm.get_tasklist();
        for task in tlist.iter() {
            if task.status == TaskStatus::Incomplete {
                incomplete_tasks += 1;
            }
        }
        incomplete_tasks
    }

    fn tasklist_print_incomplete(&self) {
        let tlist = self.tm.get_tasklist();
        let mut str_tvec = vec![];
        for l in 0..tlist.len() {
            if tlist[l].status == TaskStatus::Incomplete {
                let days: i64;
                let mut tmp_deadline: ColoredString;
                if tlist[l].date_deadline.eq("0000-00-00") {
                    tmp_deadline = tlist[l].date_deadline.clone().white();
                } else {
                    let ymd_it = tlist[l].date_deadline.split("-");
                    let ymd: Vec<&str> = ymd_it.collect();
                    //Compare Time to get color
                    days = self.tm.get_days_diff(
                        NaiveDate::from_ymd_opt(
                            ymd[0].parse::<i32>().unwrap(),
                            ymd[1].parse::<u32>().unwrap(),
                            ymd[2].parse::<u32>().unwrap(),
                        )
                        .unwrap(),
                        Utc::now().date_naive(),
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
                    "ID:{}\nName: {} \n Description: {} \n Status: {}\n Date Posted: {}\n Deadline: {}\n",
                    tlist[l].id.clone(),
                    tlist[l].name.clone(),
                    tlist[l].description.clone(),
                    tlist[l].status.clone(),
                    tlist[l].date_posted.clone(),
                    tmp_deadline
                );

                str_tvec.push(tmp_complete_string);
            }
        }

        println!("");
        for i in 0..str_tvec.len() {
            println!("{}", str_tvec[i]);
        }
        println!("");

        if str_tvec.len() == 0 {
            println!("No Incomplete Tasks!");
        }
    }

    fn tasklist_print_completed(&self) {
        let tlist = self.tm.get_tasklist();
        let mut str_tvec = vec![];
        for l in 0..tlist.len() {
            if tlist[l].status == TaskStatus::Complete {
                let days: i64;
                let mut tmp_deadline: ColoredString;
                if tlist[l].date_deadline.eq("0000-00-00") {
                    tmp_deadline = tlist[l].date_deadline.clone().white();
                } else {
                    let ymd_it = tlist[l].date_deadline.split("-");
                    let ymd: Vec<&str> = ymd_it.collect();
                    //Compare Time to get color
                    days = self.tm.get_days_diff(
                        NaiveDate::from_ymd_opt(
                            ymd[0].parse::<i32>().unwrap(),
                            ymd[1].parse::<u32>().unwrap(),
                            ymd[2].parse::<u32>().unwrap(),
                        )
                        .unwrap(),
                        Utc::now().date_naive(),
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
                    "ID:{}\nName: {} \n Description: {} \n Status: {}\n Date Posted: {}\n Deadline: {}\n",
                    tlist[l].id.clone(),
                    tlist[l].name.clone(),
                    tlist[l].description.clone(),
                    tlist[l].status.clone(),
                    tlist[l].date_posted.clone(),
                    tmp_deadline
                );

                str_tvec.push(tmp_complete_string);
            }
        }

        println!("");
        for i in 0..str_tvec.len() {
            println!("{}", str_tvec[i]);
        }
        println!("");

        if str_tvec.len() == 0 {
            println!("No Complete Tasks!");
        }
    }

    fn tasklist_print_all(&self) {
        let tlist = self.tm.get_tasklist();
        let mut str_tvec = vec![];

        for l in 0..tlist.len() {
            let days: i64;
            let mut tmp_deadline: ColoredString;

            if tlist[l].date_deadline.eq("0000-00-00") {
                tmp_deadline = tlist[l].date_deadline.clone().white();
            } else {
                let ymd_it = tlist[l].date_deadline.split("-");
                let ymd: Vec<&str> = ymd_it.collect();

                //Compare Time to get color
                days = self.tm.get_days_diff(
                    NaiveDate::from_ymd_opt(
                        ymd[0].parse::<i32>().unwrap(),
                        ymd[1].parse::<u32>().unwrap(),
                        ymd[2].parse::<u32>().unwrap(),
                    )
                    .unwrap(),
                    Utc::now().date_naive(),
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
                "ID:{}\nName: {} \n Description: {} \n Status: {}\n Date Posted: {}\n Deadline: {}\n",
                tlist[l].id.clone(),
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
        let tasklist_ref = self.tm.get_tasklist().clone();

        let mut t_options = vec![];

        for l in 0..tasklist_ref.len() {
            let t_rm_str = format!(
                "ID:{}\nName:{}\nDescription:{}\nStatus:{}",
                tasklist_ref[l].id.clone(),
                tasklist_ref[l].name.clone(),
                tasklist_ref[l].description.clone(),
                tasklist_ref[l].status.clone()
            );
            t_options.push(t_rm_str);
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

            let id_vec = self.tm.get_id_from_str(rm_vec.clone());

            self.tm.delete_tasks(id_vec);
        }
    }

    fn tasklist_mark(&mut self) {
        let tasklist_ref = self.tm.get_tasklist().clone();

        let mut t_options = vec![];

        for l in 0..tasklist_ref.len() {
            let t_rm_str = format!(
                "ID:{}\nName:{}\nDescription:{}\nStatus:{}",
                tasklist_ref[l].id.clone(),
                tasklist_ref[l].name.clone(),
                tasklist_ref[l].description.clone(),
                tasklist_ref[l].status.clone()
            );
            t_options.push(t_rm_str);
        }

        if tasklist_ref.len().eq(&0) {
            println!("Task List Empty!");
        } else if tasklist_ref.len() > 0 {
            let formatter: MultiOptionFormatter<String> =
                &|tasks| format!("Selected {} tasks", tasks.len());

            let switch_selection = MultiSelect::new("Select which entries to switch marking", t_options)
                .with_help_message("↑↓ to move, space to select one, → to all, ← to none, type to filter, enter to confirm")
                .with_formatter(formatter)
                .prompt();

            let switch_vec = switch_selection.unwrap_or_default();

            let id_vec = self.tm.get_id_from_str(switch_vec.clone());

            self.tm.switch_task_status(id_vec);
        }
    }

    fn ask_tasklist(&mut self) {
        let which_print_hash = HashMap::from([
            (0, "Print All Tasks"),
            (1, "Print Incomplete Tasks"),
            (2, "Print Complete"),
        ]);

        let which_print_vec = vec![
            which_print_hash[&0],
            which_print_hash[&1],
            which_print_hash[&2],
        ];

        let which_print: Result<&str, InquireError> =
            Select::new("Choose an action:", which_print_vec).prompt();

        match which_print {
            Ok(which) => {
                if which_print_hash[&0].eq(which) {
                    self.tasklist_print_all();
                } else if which_print_hash[&1].eq(which) {
                    self.tasklist_print_incomplete();
                } else if which_print_hash[&2].eq(which) {
                    self.tasklist_print_completed();
                }
            }

            Err(_) => {
                println!("There was an error, please try again");
            }
        }
    }

    pub fn run(&mut self) {
        self.tm.load();

        let version = env!("CARGO_PKG_VERSION");
        print!("Forgetful Me Ver. - {}\n", version);
        println!("A simple task reminder software.\n");

        println!(
            "\nCompleted tasks:{}, Incomplete tasks:{}",
            self.get_complete_tasks().clone(),
            self.get_incomplete_tasks().clone()
        );

        {
            let colort = self.get_colored_tasks().clone();

            println!(
                "\n\tGreen: {}\n\tYellow: {}\n\tRed: {}\n\tMagenta: {}",
                colort["GREEN"], colort["YELLOW"], colort["RED"], colort["MAGENTA"]
            );
        }

        print!("\n");

        'm_loop: loop {
            let options_hash: HashMap<u8, &str> = HashMap::from([
                (0, "Add Task"),
                (1, "Remove Task"),
                (2, "Mark Task Incomplete/Complete"),
                (3, "View Task List"),
                (4, "Status"),
                (5, "Quit"),
            ]);

            let menu_options = vec![
                options_hash[&0],
                options_hash[&1],
                options_hash[&2],
                options_hash[&3],
                options_hash[&4],
                options_hash[&5],
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
                        //MARK ENTRIES AS COMPLETE OR INCOMPLETE
                        self.clear_screen();
                        self.tasklist_mark();
                    } else if options_hash[&3].eq(choice) {
                        //VIEW
                        self.clear_screen();
                        //Decide which is going to be printed
                        self.ask_tasklist();
                    } else if options_hash[&4].eq(choice) {
                        //STATUS
                        self.clear_screen();
                        println!(
                            "\nCompleted tasks:{}, Incomplete tasks:{}\n",
                            self.get_complete_tasks().clone(),
                            self.get_incomplete_tasks().clone()
                        );
                        let colort = self.get_colored_tasks().clone();
                        println!(
                            "\n\tGreen: {}\n\tYellow: {}\n\tRed: {}\n\tMagenta: {}",
                            colort["GREEN"], colort["YELLOW"], colort["RED"], colort["MAGENTA"]
                        );
                        println!("\n");
                    } else if options_hash[&5].eq(choice) {
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
