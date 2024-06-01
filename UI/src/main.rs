#[macro_use]
extern crate sqlx;
extern crate dotenv;

use std::io::Write;
use database::*;
use tokio::main;
use tokio::runtime::Runtime;
use sqlx::{pool, SqlitePool};
use dotenv::dotenv;
use std::env;
use std::fs::File;
use std::future::IntoFuture;
use std::io::Read;
use std::path::Path;

mod models;
mod database;

use crate::models::{User, Credential};
use crate::database::{establish_connection, create_tables, save_user, get_user, get_credentials};
use std::process::{ExitStatus};
use tokio::process::Command;
use std::io;

use app_state_derived_lenses::{username};

use druid::widget::{Button, Flex, Label, Padding, TextBox, Scroll, List};
use druid::{AppDelegate as OtherAppDelegate, AppLauncher, Data, Handled, Lens, Selector, WidgetExt, WindowDesc};

#[derive(Clone, Data, Lens)]
struct AppState {
    name: String,
    username: String,
    fingerprint_path: String,
    site: String,
    site_username: String,
    site_password: String,
}

impl AppState {
    fn new() -> Self {
        Self {
            name: "Bioguard".into(),
            username: "".into(),
            fingerprint_path: "".into(),
            site: "".into(),
            site_username: "".into(),
            site_password: "".into(),
        }
    }
}

const LOGIN: Selector = Selector::new("login");

// Function to detect minutiae in the fingerprint image
fn detect_minutiae(image: &Vec<Vec<u8>>) -> Vec<(usize, usize)> {
    let mut minutiae = Vec::new();
    let height = image.len();
    let width = image[0].len();

    for i in 1..height-1 {
        for j in 1..width-1 {
            if image[i][j] == 1 {
                let mut count = 0;
                for x in i-1..=i+1 {
                    for y in j-1..=j+1 {
                        if image[x][y] == 1 {
                            count += 1;
                        }
                    }
                }
                if count == 3 {
                    minutiae.push((i, j));
                }
            }
        }
    }
    minutiae
}

// Function to perform minutiae matching between two fingerprint images
fn minutiae_matching(image1: &Vec<Vec<u8>>, image2: &Vec<Vec<u8>>) -> Vec<(usize, usize)> {
    let minutiae1 = detect_minutiae(image1);
    let minutiae2 = detect_minutiae(image2);

    // Simple matching algorithm: matching minutiae if they are close enough
    let mut matches = Vec::new();
    for minutia1 in &minutiae1 {
        for minutia2 in &minutiae2 {
            let distance = ((minutia1.0 as isize - minutia2.0 as isize).pow(2) +
                            (minutia1.1 as isize - minutia2.1 as isize).pow(2)) as f64;
            if distance.sqrt() < 5.0 && !matches.contains(minutia1) {
                matches.push(*minutia1);
            }
        }
    }
    matches
}

fn match_test() {
    let fingerprint_image1 = vec![
        vec![0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
        vec![0, 1, 1, 0, 0, 1, 0, 0, 0, 0],
        vec![0, 1, 0, 0, 0, 1, 0, 0, 0, 0],
        vec![0, 0, 1, 0, 0, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 0],
        vec![0, 1, 1, 0, 1, 1, 0, 0, 0, 0],
        vec![1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
        vec![0, 0, 0, 0, 1, 1, 0, 1, 0, 0],
        vec![0, 0, 0, 1, 0, 0, 0, 1, 0, 0],
        vec![0, 1, 1, 1, 0, 0, 0, 0, 0, 1],
        vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    ];

    let fingerprint_image2 = vec![
        vec![0, 0, 1, 0, 1, 0, 0, 0, 0, 1],
        vec![0, 0, 1, 1, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 1, 0, 1, 0, 0, 1, 0],
        vec![0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 1, 0, 0, 0, 1, 0],
        vec![0, 0, 0, 1, 0, 0, 1, 0, 0, 0],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 1, 0],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![0, 0, 0, 0, 1, 0, 0, 1, 0, 0],
        vec![0, 1, 0, 0, 0, 0, 0, 1, 0, 0],
        vec![0, 0, 0, 1, 0, 0, 0, 0, 0, 1],
        vec![0, 0, 0, 0, 1, 0, 1, 0, 0, 0],
        vec![0, 1, 1, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1, 0, 0, 0, 0],
    ];
    
    
     let fingerprint_image3 = vec![
        vec![0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
        vec![0, 1, 1, 0, 0, 1, 0, 0, 0, 0],
        vec![0, 1, 0, 0, 0, 1, 0, 0, 0, 0],
        vec![0, 0, 1, 0, 0, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 0],
        vec![0, 1, 1, 0, 1, 1, 0, 0, 0, 0],
        vec![1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
        vec![0, 0, 0, 0, 1, 1, 0, 1, 0, 0],
        vec![0, 0, 0, 1, 0, 0, 0, 1, 0, 0],
        vec![0, 1, 1, 1, 0, 0, 0, 0, 0, 1],
        vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    ];


    let matches = minutiae_matching(&fingerprint_image1, &fingerprint_image2);
    let matches2 = minutiae_matching(&fingerprint_image1, &fingerprint_image3);
    
    //println!("Matched minutiae: {:?}", matches);
    
    if matches.len() > 11 {
        println!("Fingerprints 1 and 2 match!");
    } else {
        println!("Fingerprints 1 and 2 do not match!");
    }
    
    //println!("Matched minutiae: {:?}", matches2);
    if matches2.len() > 11 {
        println!("Fingerprints 1 and 3 match!");
    } else {
        println!("Fingerprints 1 and 3 do not match!");
    }
}


async fn call_fingerprint_capture() -> io::Result<()> {
    let output = Command::new("powershell")
        .arg("-Command")
        .arg("Start-Process cmd -ArgumentList '/c .\\fingerprintCapture.exe' -Wait -Verb RunAs")
        .output()
        .await?;  // Await the command and handle the result

    if output.status.success() {
        println!("Process executed successfully");
    } else {
        eprintln!("Process failed with exit code: {:?}", output.status.code());
    }

    Ok(())
}

fn convert_image_to_binary(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut image = Vec::new();
    file.read_to_end(&mut image)?;
    Ok(image)
}

fn binary_to_image(image: &Vec<u8>, file_path: &str) -> io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(image)?;
    Ok(())
}   

#[tokio::main]
async fn main() {
    dotenv().ok();
    
    let rt = Runtime::new().unwrap();

    let current_dir = env::current_dir().expect("Failed to get current directory");
    let database_path = current_dir.join("users.db");
    let database_url = format!("sqlite://{}", database_path.to_str().unwrap());
    
    let pool = SqlitePool::connect(&database_url).await.unwrap();

    create_tables(&pool).await.expect("Failed to create tables");
    
    let size = (400.0, 400.0);
    let main_windows = WindowDesc::new(login_ui(pool.clone()))
    .title("Bioguard")
    .window_size(size);
    
    let initial_state = AppState::new();

    AppLauncher::with_window(main_windows)
    //.delegate(AppDelegate::new(pool))
    .launch(initial_state)
    .expect("Failed to launch application");
}


fn login_ui(pool: SqlitePool) -> impl druid::Widget<AppState> {
    let label = Label::new("Bioguard Login").padding(5.0);

    let username_input = TextBox::new().with_placeholder("Username").lens(AppState::username);

    let login_button = Button::new("Login").on_click(|_ctx, data: &mut AppState, _env| {
        call_fingerprint_capture();
        //_ctx.submit_command(LOGIN);
        //TODO CALL VERIFICATION FUNCTION
        //TODO LAUNCH CREDENTIALS UI
    });

    let register_button = Button::new("Register").on_click(move |_ctx, data: &mut AppState, _env| {
        _ctx.new_window(WindowDesc::new(register_ui(pool.clone())).title("Register"));
    });
    
    Flex::column()
    .with_child(label)
    .with_spacer(20.0)
    .with_child(username_input)
    .with_spacer(20.0)
    .with_child(login_button)
    .with_spacer(20.0)
    .with_child(register_button)
    
}


fn register_ui(pool: SqlitePool) -> impl druid::Widget<AppState> {
    let label = Label::new("Bioguard").padding(5.0);

    let username_input = TextBox::new().with_placeholder("Username").lens(AppState::username);
    
    let info = Label::new("To register your account with your fingerprint,\nPlease click on the Register button").padding(5.0);
    
    let register_button = Button::new("Register").on_click(move |_ctx, data: &mut AppState, _env| {
        
        let pool = pool.clone();
        let _username = data.username.clone();
        tokio::spawn(async move {
            println!("Username: {}", _username);
            if let Err(e) = call_fingerprint_capture().await {
                eprintln!("Failed to call fingerprint capture: {}", e);
                return;
            }
            
            let image_path = Path::new("data/fingerprint_Input.bmp");
            let binary_data = convert_image_to_binary(image_path.to_str().unwrap()).expect("Failed to convert image to binary");
            
            save_user(&pool, &_username, binary_data).await.expect("Failed to save user");
            println!("User saved successfully");
        });

    });
    
    Flex::column()
    .with_child(label)
    .with_spacer(20.0)
    .with_child(username_input)
    .with_spacer(20.0)
    .with_child(info)
    .with_spacer(20.0)
    .with_child(register_button)

    
}

fn credentials_ui(pool, SqlitePool, user_id: i64) -> impl druid::Widget<AppState> {
    let label = Label::new("Your Credentials").padding(5.0);

    let site_input = TextBox::new().with_placeholder("Site").lens(AppState::site);
    let site_username_input = TextBox::new().with_placeholder("Site Username").lens(AppState::site_username);
    let site_password_input = TextBox::new().with_placeholder("Site Password").lens(AppState::site_password);

    let save_button = Button::new("Add Credential").on_click(|_ctx, data: &mut AppState, _env| {
        println!("TODO save credentials");
        /*let rt = Runtime::new().unwrap();
        let pool = rt.block_on(establish_connection());
        //let user_id: i64 = get_user(&pool, &data.username).expect("Failed to find user").id;

        rt.block_on(save_credentials(&pool, 0/*user_id*/, &data.site, &data.site_username, &data.site_password))
        .expect("Failed to save credentials");*/
    });
    
    Flex::column()
    .with_child(label)
    .with_spacer(20.0)
    .with_child(site_input)
    .with_spacer(20.0)
    .with_child(site_username_input)
    .with_spacer(20.0)
    .with_child(site_password_input)
    .with_spacer(20.0)
    .with_child(save_button)
    .with_spacer(20.0)    
}

struct AppDelegate {
    pool: SqlitePool,
}

impl AppDelegate {
    fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
        }
    }
    
}

impl druid::AppDelegate<AppState> for AppDelegate {
    fn command(
        &mut self,
        _ctx: &mut druid::DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        data: &mut AppState,
        _env: &druid::Env,
    ) -> Handled {
        if cmd.is(LOGIN) {
            let rt = Runtime::new().unwrap();

            // Read the fingerprint image
            let mut file = File::open(&data.fingerprint_path).expect("Failed to open fingerprint image");
            let mut fingerprint = Vec::new();
            file.read_to_end(&mut fingerprint).expect("Failed to read fingerprint image");

            match rt.block_on(get_user(&self.pool, &data.username)) {
                Ok(user) => {
                    if 1 == 1 //TODO CALL VERIFICATION FUNCTION
                    {
                        _ctx.new_window(WindowDesc::new(credentials_ui()).title("Your Credentials"));
                    } else {
                        println!("Fingerprint does not match");
                    }
                }
                Err(_) => {
                    rt.block_on(save_user(&self.pool, &data.username, fingerprint)).expect("Failed to save user");
                    let user = rt.block_on(get_user(&self.pool, &data.username)).expect("Failed to find user");
                    _ctx.new_window(WindowDesc::new(credentials_ui()).title("Your Credentials"));
                }
            }
            return Handled::Yes;
        }
        Handled::No
    }
}

