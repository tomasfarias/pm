extern crate shellexpand;

use std::path;
use std::fs;
use std::io::Read;
use std::io::Write;
use structopt::StructOpt;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::HashMap;
use std::fs::File;
use toml;


#[derive(Debug, StructOpt)]
#[structopt(name = "pm", about = "Manage all your projects in one place.")]
enum Cli {
    #[structopt(name = "list")]
    List {},
    #[structopt(name = "add")] 
    Add {
        #[structopt(short = "p", long = "path", parse(from_os_str))]
        path: path::PathBuf,
    },
    #[structopt(name = "rm")]
    Remove {
        #[structopt(short = "n", long = "name")]
        name: String,
    }
}

#[derive(Debug, Deserialize)]
struct Manager {
    projects: HashMap<String, Project>,
    file_name: String,
}

impl Manager {
    fn default(file_name: &String) -> Self {
        Manager {
            projects: HashMap::new(),
            file_name: file_name.to_string(),
        }
    }

    fn from_toml(file_name: &String) -> Self {
        if path::Path::new(&file_name).exists() == false {
            Self::default(&file_name)
        } else {
            match Self::read_managed_projects_from_toml(&file_name) {
                Ok(manager) => manager,
                Err(_) => Self::default(&file_name),
            }
        }
    
    }

    fn read_managed_projects_from_toml(file_name: &String) -> Result<Manager, toml::de::Error> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .append(true)
            .open(&file_name)
            .unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents);

        let project_manager = Manager{
            projects: toml::from_str(&contents)?,
            file_name: file_name.to_string(),
        };

        Ok(project_manager)
    }

    fn insert_project(&mut self, pathbuf: &std::path::PathBuf) {
        let new_project = Project{
            path: pathbuf.as_path().to_str().unwrap().to_string(),
        };
        let name = pathbuf.file_name().unwrap().to_str().unwrap().to_string();        
        self.projects.insert(name, new_project);

        self.save_projects_to_toml();
    }

    fn save_projects_to_toml(&self) {
        let projects = toml::to_string(&self.projects).unwrap();
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(&self.file_name)
            .unwrap();
        file.write_all(projects.as_bytes());
    }

    fn list_projects(&mut self) {
        for (name, project) in &self.projects {
            println!("{}: {}", name, project.path);
        }
    }

    fn remove_project(&mut self, key: &String) {
        self.projects.remove(&key.to_owned());
        self.save_projects_to_toml();
    }
}


#[derive(Debug, Deserialize, Serialize)]
struct Project {
    path: String,
}

fn main() {
    let cli = Cli::from_args();
    println!("{:?}", cli);
   
    let pm_dir = create_dir_if_not_exists("~/.pm/");
    let file_name = format!("{}managed.toml", pm_dir);
    let mut project_manager = Manager::from_toml(&file_name);
    
    match Cli::from_args() {
        Cli::List {} => project_manager.list_projects(),
        Cli::Remove { name } => project_manager.remove_project(&name),
        Cli::Add { path }=> project_manager.insert_project(&path),
        _ => eprintln!("Unkown command"),
    }
}

fn create_dir_if_not_exists(dir: &str) -> String{
    let dir = shellexpand::tilde(&dir).into_owned();

    if path::Path::new(&dir).exists() == false {
        match fs::create_dir_all(&dir) {
            Ok(_) => {},
            Err(_) => {},
        }
    }

    dir
}
