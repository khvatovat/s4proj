#[macro_use]
extern crate sqlx;
extern crate dotenv;

use std::io::Write;
use database::*;
use tokio::main;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;
use tokio::task;
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

use std::sync::Arc;

use druid::widget::{Button, Flex, Label, Padding, TextBox, Scroll, List, SizedBox};
use druid::{AppDelegate as OtherAppDelegate, AppLauncher, Data, Handled, Lens, Selector, WidgetExt, WindowDesc, Widget, Command as DruidCommand, Target, Color};

#[derive(Debug, Clone, Data, Lens)]
struct AppState {
    view: ViewSelector,
    name: String,
    username: String,
    fingerprint_path: String,
    site: String,
    site_username: String,
    site_password: String,
    credentials: Arc<Vec<Credential>>,
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
            credentials: Arc::new(Vec::new()),
            view: ViewSelector::Login,
        }
    }
}

#[derive(Debug, Clone, Data, PartialEq)]
enum ViewSelector {
    Login,
    Register,
    Credentials,
}

const CELL_WIDTH: f64 = 150.0;
const CELL_HEIGHT: f64 = 50.0;
const HEADER_COLOR: Color = Color::rgb8(0x2e, 0x2e, 0x2e);
const TEXT_COLOR: Color = Color::WHITE;
const CELL_BG_COLOR: Color = Color::rgb8(0x3c, 0x3c, 0x3c);

const LOGIN: Selector = Selector::new("login");
const SHOW_REGISTER: Selector = Selector::new("show-register");
const SHOW_LOGIN: Selector = Selector::new("show-login");
const SHOW_CREDENTIALS: Selector = Selector::new("show-credentials");
const UPDATE_CREDENTIALS: Selector<Arc<Vec<Credential>>> = Selector::new("update-credentials");


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
    
    let size = (800.0, 400.0);
    let main_windows = WindowDesc::new(build_ui(pool.clone().into()))
    .title("Bioguard")
    .window_size(size);
    
    let initial_state = AppState::new();

    AppLauncher::with_window(main_windows)
    //.delegate(AppDelegate::new(pool))
    .launch(initial_state)
    .expect("Failed to launch application");
}

fn build_ui(pool: Arc<SqlitePool>) -> impl Widget<AppState> {


    // LOGIN VIEW
    let label_log = Label::new("Bioguard Login").padding(5.0);

    let username_input = TextBox::new().with_placeholder("Username").lens(AppState::username);

    let pool_clone1 = Arc::clone(&pool);

    let login_button = Button::new("Login").on_click(move |_ctx, data: &mut AppState, _env| {
        let _username = data.username.clone();
        my_child_login(_username.clone(), Arc::clone(&pool_clone1));
        //TODO Handle login failure
        data.view = ViewSelector::Credentials;

        my_child_update(&pool_clone1, &_username, None, None, None, data);
        //_ctx.submit_command(DruidCommand::new(SHOW_CREDENTIALS, (), Target::Global));
        //_ctx.submit_command(LOGIN);
        //TODO CALL VERIFICATION FUNCTION
        //TODO LAUNCH CREDENTIALS UI
    });

    let pool_clone2 = Arc::clone(&pool);
    let register_button_log = Button::new("Register").on_click({
        move |_ctx, data: &mut AppState, _env| {
            data.view = ViewSelector::Register;
            //_ctx.submit_command(DruidCommand::new(SHOW_REGISTER, (), Target::Global));
            }
    });
    
    let login_view = Flex::column()
    .with_child(label_log)
    .with_spacer(20.0)
    .with_child(username_input)
    .with_spacer(20.0)
    .with_child(login_button)
    .with_spacer(20.0)
    .with_child(register_button_log);



    // REGISTER VIEW
    let label_reg = Label::new("Bioguard Register").padding(5.0);
    
    let info = Label::new("To register your account with your fingerprint,\nPlease click on the Register button").padding(5.0);

    let username_input = TextBox::new().with_placeholder("Username").lens(AppState::username);
    
    let pool_clone3 = Arc::clone(&pool);
    let register_button_reg = Button::new("Register").on_click(
        move |_ctx, data: &mut AppState, _env| {
            //_ctx.submit_command(DruidCommand::new(SHOW_REGISTER, (), Target::Global));
        
            //let pool_clone3 = Arc::clone(&pool);
            let _username = data.username.clone();
            let mut ret = 0;
            my_child_register(_username.clone(), Arc::clone(&pool_clone3));

            /*if ret == 1 {
                println!("Launch new windows");
            }*/
            println!("Registering user");

            data.view = ViewSelector::Credentials;

            my_child_update(&pool_clone3, &_username, None, None, None, data);
            }
        );
    
    let register_view = Flex::column()
    .with_child(label_reg)
    .with_spacer(20.0)
    .with_child(username_input)
    .with_spacer(20.0)
    .with_child(info)
    .with_spacer(20.0)
    .with_child(register_button_reg);


    // CREDENTIALS VIEW
    let label_cr = Label::new("Your Credentials").padding(5.0);

    let site_input = TextBox::new().with_placeholder("Site").lens(AppState::site);
    let site_username_input = TextBox::new().with_placeholder("Site Username").lens(AppState::site_username);
    let site_password_input = TextBox::new().with_placeholder("Site Password").lens(AppState::site_password);

    let pool_save  = Arc::clone(&pool);

    let save_button = Button::new("Add Credential").on_click({
        let pool_clone = Arc::clone(&pool);
        move |_ctx, data: &mut AppState, _env| {

            let user = data.username.clone();
            let binding = pool.clone();
            let site = data.site.clone();
            let site_username = data.site_username.clone();
            let site_password = data.site_password.clone();

            //let tx = _ctx.get_external_handle();
            
            my_child_update(&binding, &user, Some(&site), Some(&site_username), Some(&site_password), data);
        }
    });

    let delete_credentials_button = Button::new("Delete Credential").on_click({
        let pool_clone = Arc::clone(&pool_save);
        move |_ctx, data: &mut AppState, _env| {
            let user = data.username.clone();
            let site = data.site.clone();
            let binding = Arc::clone(&pool_clone);

            my_child_delete(&binding, &user, Some(&site), data);

            my_child_update(&binding, &user, None, None, None, data);
        }
    });    

   // Table headers
   let headers = Flex::row()
   .with_child(
       SizedBox::new(
           Label::new("Site")
               .with_text_color(TEXT_COLOR)
               .center()
               .background(HEADER_COLOR)
       )
       .width(CELL_WIDTH)
       .padding(5.0)
       .border(Color::BLACK, 1.0),
   )
   .with_child(
       SizedBox::new(
           Label::new("Username")
               .with_text_color(TEXT_COLOR)
               .center()
               .background(HEADER_COLOR)
       )
       .width(CELL_WIDTH)
       .padding(5.0)
       .border(Color::BLACK, 1.0),
   )
   .with_child(
       SizedBox::new(
           Label::new("Password")
               .with_text_color(TEXT_COLOR)
               .center()
               .background(HEADER_COLOR)
       )
       .width(CELL_WIDTH)
       .padding(5.0)
       .border(Color::BLACK, 1.0),
   );

    // List of credentials
    let credentials_list = List::new(|| {
    Flex::row()
        .with_child(
            SizedBox::new(
                Label::new(|cred: &Credential, _env: &_| format!("{}", cred.site))
                    .with_text_color(TEXT_COLOR)
                    .center()
                    .background(CELL_BG_COLOR)
            )
            .width(CELL_WIDTH)
            .padding(5.0)
            .border(Color::BLACK, 1.0),
        )
        .with_child(
            SizedBox::new(
                Label::new(|cred: &Credential, _env: &_| format!("{}", cred.site_username))
                    .with_text_color(TEXT_COLOR)
                    .center()
                    .background(CELL_BG_COLOR)
            )
            .width(CELL_WIDTH)
            .padding(5.0)
            .border(Color::BLACK, 1.0),
        )
        .with_child(
            SizedBox::new(
                Label::new(|cred: &Credential, _env: &_| format!("{}", cred.site_password))
                    .with_text_color(TEXT_COLOR)
                    .center()
                    .background(CELL_BG_COLOR)
            )
            .width(CELL_WIDTH)
            .padding(5.0)
            .border(Color::BLACK, 1.0),
        )
    })
    .lens(AppState::credentials);

    // Table view
    let table = Flex::column()
    .with_child(headers)
    .with_child(credentials_list);

        
    let credentials_view = 
    Flex::column()
    .with_child(label_cr)
    .with_spacer(40.0)
    .with_child(
        Flex::row()
            .with_child(
                Flex::column()
                .with_child(site_input)
                .with_spacer(20.0)
                .with_child(site_username_input)
                .with_spacer(20.0)
                .with_child(site_password_input)
                .with_spacer(20.0)
                .with_child(save_button)
                .with_spacer(20.0)
                .with_child(delete_credentials_button)
            )
            .with_spacer(40.0)
            .with_child(table)
    );


    // MAIN VIEW
    let main_view = Flex::column().with_child(druid::widget::Either::new(
        |data: &AppState, _env| data.view == ViewSelector::Login,
        login_view,
        druid::widget::Either::new(
            |data: &AppState, _env| data.view == ViewSelector::Register,
            register_view,
            credentials_view,
        ),
    ));

    main_view
}


struct Delegate;

impl druid::AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut druid::DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        data: &mut AppState,
        _env: &druid::Env,
    ) -> druid::Handled {
        if cmd.is(SHOW_REGISTER) {
            data.view = ViewSelector::Register;
            return druid::Handled::Yes;
        } else if cmd.is(SHOW_LOGIN) {
            data.view = ViewSelector::Login;
            return druid::Handled::Yes;
        } else if cmd.is(SHOW_CREDENTIALS) {
            data.view = ViewSelector::Credentials;
            return druid::Handled::Yes;
        } else if let Some(credentials) = cmd.get(UPDATE_CREDENTIALS) {
            println!("Updating credentials");
            // **Modified**: Update credentials in AppState
            data.credentials = credentials.clone();
            return druid::Handled::Yes;
        }
        druid::Handled::No
    }
}
fn my_child_delete(binding: &Arc<SqlitePool>, user: &str, site: Option<&str>, data: &mut AppState) {
    let result = task::block_in_place (||  {

        let task_result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async{    
            delete_credentials(&binding, &user, &(site.unwrap())).await.expect("Failed to delete credential");
        });
    });
}



fn my_child_update(binding: &Arc<SqlitePool>, user: &str, site: Option<&str>, site_username: Option<&str>, site_password: Option<&str>, data: &mut AppState) {
    let result = task::block_in_place (||  {

        let task_result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async{
            
            if site.unwrap_or("") == "" || site_username.unwrap_or("") == "" || site_password.unwrap_or("") == "" {
                println!("Just updated, Please fill in all fields");
                match get_credentials(&binding, &user).await {
                    Ok(creds) => {
                        data.credentials = Arc::new(creds);
                    }
                    Err(e) => {
                        eprintln!("Failed to get credentials: {}", e);
                    }
                };
                return;
            }
            match save_credentials(&binding, &user, &(site.unwrap()), &(site_username.unwrap()), &(site_password.unwrap())).await {
                Ok(_) => {
                    println!("Credential saved successfully");
                    match get_credentials(&binding, &user).await {
                        Ok(creds) => {
                            data.credentials = Arc::new(creds);
                        }
                        Err(e) => {
                            eprintln!("Failed to get credentials: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to save credential: {}", e);
                }
            }
    
        });
    });
}

fn my_child_login(_username: String, pool: Arc<SqlitePool>) {
    let result = task::block_in_place (||  {

        let task_result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async{
            println!("Username: {}", _username);
            if let Err(e) = call_fingerprint_capture().await {
                eprintln!("Failed to call fingerprint capture: {}", e);
    
                return;
            }

            let user = get_user(&pool, &_username).await.expect("Failed to find user");

            if user.is_some() {

                let image_login = binary_to_image(&user.unwrap().fingerprint_image, "data/fingerprint_Login.bmp").expect("Failed to convert image to binary");

                //TODO  :
                ///     -LOAD FINGERPRINT IMAGE
                //      -PREPROCESS
                ///     -EXTRACT MINUTIAE
                ///     -MATCH MINUTIAE

                //let matches = minutiae_matching(&image_login, &attempt_image);
                if true {//matches.len() > 11 {
                    println!("Fingerprints match!");
                } else {
                    println!("Fingerprints do not match!");
                }
            } else {
                println!("User not found");
            }
            println!("User successfully logged in");
    
        });
    });
}

fn login_ui(pool: Arc<SqlitePool>) -> impl druid::Widget<AppState> {
    let label = Label::new("Bioguard Login").padding(5.0);

    let username_input = TextBox::new().with_placeholder("Username").lens(AppState::username);

    let pool_clone = Arc::clone(&pool);
    let pool_clone2 = Arc::clone(&pool);

    let login_button = Button::new("Login").on_click(move |_ctx, data: &mut AppState, _env| {
        let _username = data.username.clone();
        my_child_login(_username.clone(), Arc::clone(&pool_clone));
        //TODO Handle login failure
        _ctx.new_window(WindowDesc::new(credentials_ui(Arc::clone(&pool_clone2), &_username.clone())));
        //_ctx.submit_command(LOGIN);
        //TODO CALL VERIFICATION FUNCTION
        //TODO LAUNCH CREDENTIALS UI
    });

    let pool_clone = Arc::clone(&pool);
    let register_button = Button::new("Register").on_click(move |_ctx, data: &mut AppState, _env| {
        _ctx.new_window(WindowDesc::new(register_ui(Arc::clone(&pool_clone))));
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



fn my_child_register(_username: String, pool: Arc<SqlitePool>) {
    let result = task::block_in_place (||  {

        let task_result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async{
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
    
    
}

fn register_ui(pool: Arc<SqlitePool>) -> impl druid::Widget<AppState> {

    let label = Label::new("Bioguard").padding(5.0);

    let username_input = TextBox::new().with_placeholder("Username").lens(AppState::username);
    
    let info = Label::new("To register your account with your fingerprint,\nPlease click on the Register button").padding(5.0);
    
    let register_button = Button::new("Register").on_click(move |_ctx, data: &mut AppState, _env| {
        
        let pool = Arc::clone(&pool);
        let _username = data.username.clone();
        let mut ret = 0;
        my_child_register(_username.clone(), Arc::clone(&pool));

        /*if ret == 1 {
            println!("Launch new windows");
        }*/
        println!("Registering user");

        _ctx.new_window(WindowDesc::new(credentials_ui(Arc::clone(&pool), &_username)));

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

fn credentials_ui(pool: Arc<SqlitePool>, _username: &str) -> impl druid::Widget<AppState> {
    let label = Label::new("Your Credentials").padding(5.0);

    let site_input = TextBox::new().with_placeholder("Site").lens(AppState::site);
    let site_username_input = TextBox::new().with_placeholder("Site Username").lens(AppState::site_username);
    let site_password_input = TextBox::new().with_placeholder("Site Password").lens(AppState::site_password);

    let user = _username.to_string();
    let binding = pool.clone();
    let cred = get_credentials(&binding, &user);


    let mut list = List::new(|| {
        Label::new(|cred: &Credential, _env: &_| format!("Site: {}, Username: {}, Password: {}", cred.site, cred.site_username, cred.site_password))
    }).lens(AppState::credentials);

    let user = _username.to_string();
    let save_button = Button::new("Add Credential").on_click(move |_ctx, data: &mut AppState, _env| {

        let user = user.clone();
        let binding = pool.clone();
        let site = data.site.clone();
        let site_username = data.site_username.clone();
        let site_password = data.site_password.clone();
        let mut data_clone = data.clone();

        tokio::spawn(async move {
            match save_credentials(&binding, &user, &site, &site_username, &site_password).await {
                Ok(_) => {
                    println!("Credential saved successfully");
                    match get_credentials(&binding, &user).await {
                        Ok(creds) => {
                            data_clone.credentials = creds.into();
                        }
                        Err(e) => {
                            eprintln!("Failed to get credentials: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to save credential: {}", e);
                }
            }
        });

        //TODO CALL SAVE CREDENTIALS FUNCTION
        println!("credential saved successfully");
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
    .with_child(list)
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
/*
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
}*/

