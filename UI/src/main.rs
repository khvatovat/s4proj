#[macro_use]
extern crate sqlx;
extern crate dotenv;

mod extractor;
use crate::extractor::*;

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

use image::{load_from_memory, ImageError, DynamicImage, ImageFormat};

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
    println!("Matching minutiae1 {}, len[0] = {}", image1.len(), image1[0].len());
    let minutiae1 = detect_minutiae(image1);
    println!("Minutiae1 done");
    let minutiae2 = detect_minutiae(image2);
    println!("Minutiae2 done");

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

fn match_test(mat_try: Vec<Vec<u8>>, mat_db: Vec<Vec<u8>>) -> bool {
    let matches = minutiae_matching(&mat_try, &mat_db);
    
    //println!("Matched minutiae: {:?}", matches);
    
    if matches.len() > 11 {
        return true;
    } else {
        false
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

fn binary_to_image(image_data: &Vec<u8>, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::load_from_memory(image_data).unwrap();
    let mut output_file = File::create(output_path)?;
    img.write_to(&mut output_file, ImageFormat::Png)?;
    Ok(())
}


// Function to flatten Vec<Vec<u8>> into Vec<u8>
fn flatten(vec: Vec<Vec<u8>>) -> Vec<u8> {
    vec.into_iter().flatten().collect()
}

// Function to reconstruct Vec<Vec<u8>> from Vec<u8>
fn reconstruct(flat: Vec<u8>, rows: usize, cols: usize) -> Vec<Vec<u8>> {
    flat.chunks(cols).map(|chunk| chunk.to_vec()).collect()
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
        let logged = my_child_login(_username.clone(), Arc::clone(&pool_clone1));
        //TODO Handle login failure

        println!("Logged: {}", logged);

        if logged {
            data.view = ViewSelector::Credentials;
            my_child_update(&pool_clone1, &_username, None, None, None, data);
        }

        //TDOO notify not logged in

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

            data.site = "".to_string();
            data.site_username = "".to_string();
            data.site_password = "".to_string();
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

fn my_child_login(_username: String, pool: Arc<SqlitePool>) -> bool{
    let result = task::block_in_place (||  {

        let task_result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async{
            println!("Username: {}", _username);
            if let Err(e) = call_fingerprint_capture().await {
                eprintln!("Failed to call fingerprint capture: {}", e);
    
                return false;
            }

            let user = get_user(&pool, &_username).await.expect("Failed to find user");
            println!("User: {:?} found", user);

            if user.is_some() {

                let vec1 = user.unwrap().fingerprint_image;
                let vec2 = reconstruct(vec1, 80, 64);

                println!("Fingerprint image loaded");
                

                let image_path = String::from("data/fingerprint_Input.bmp");
                let hist_try = histogram_equalization(&image_path);
                let bin_try = binarization(hist_try.clone(), 128);
                let thin_try = thin(&bin_try);
                let minutia_try = mark_minutia(&thin_try);
                let res_image_try = remove_false_minutia(thin_try, minutia_try, 10.0, 0.5);
                //TODO  :
                ///     -LOAD FINGERPRINT IMAGE
                //      -PREPROCESS
                ///     -EXTRACT MINUTIAE
                ///     -MATCH MINUTIAE

                println!("About to do mathces");
                let matches = minutiae_matching(&res_image_try, &vec2);
                println!("Matches: {:?}", matches);
                if matches.len() > 11 {
                    println!("Fingerprints match! ta mere");
                    return true;
                } else {
                    println!("Fingerprints do not match!");
                    return false;
                }
                return matches.len() > 11;
            } else {
                println!("User not found");
            }
            false
        });
        task_result
    });
    result
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
            
            let image_path = String::from("data/fingerprint_Input.bmp");
            let hist_try = histogram_equalization(&image_path);
            let bin_try = binarization(hist_try.clone(), 128);
            let thin_try = thin(&bin_try);
            let minutia_try = mark_minutia(&thin_try);
            let res_image_try = remove_false_minutia(thin_try, minutia_try, 10.0, 0.5);
            
            save_user(&pool, &_username, flatten(res_image_try)).await.expect("Failed to save user");
            println!("User saved successfully");
    
        });
    });
    
    
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



#[derive(Debug, Clone, PartialEq)]
pub enum MinutiaType {
    RidgeEnding,
    Bifurcation,
}

#[derive(Debug, Clone)]
pub struct Minutia {
    x: usize,
    y: usize,
    minutia_type: MinutiaType,
}

///PREPROCESSING BLOCK
///
// Histogram equalization
fn histogram_equalization(image_path: &str) -> Vec<Vec<u8>> {
    let img = image::open(&Path::new(image_path)).unwrap().to_luma8();
    let mut histogram = [0u32; 256];
    let mut cdf = [0u32; 256];
    let mut res = vec![vec![0u8; img.width() as usize]; img.height() as usize];

    for pixel in img.pixels() {
        histogram[pixel[0] as usize] += 1;
    }

    // Calculate the cumulative distribution function (CDF)
    cdf[0] = histogram[0];
    for i in 1..256 {
        cdf[i] = cdf[i - 1] + histogram[i];
    }

    let n = img.width() * img.height();
    let scale = 255.0 / (n as f32);
    for (y, row) in res.iter_mut().enumerate() {
        for (x, pixel) in row.iter_mut().enumerate() {
            let orig = img.get_pixel(x as u32, y as u32)[0];
            *pixel = (cdf[orig as usize] as f32 * scale).round() as u8;
        }
    }
    res
}

// To Black&Whrite
fn binarization(input: Vec<Vec<u8>>, threshold: u8) -> Vec<Vec<u8>> {
    let mut res = vec![vec![0u8; input[0].len()]; input.len()];
    for (y, row) in input.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            res[y][x] = if pixel > threshold { 1 } else { 0 };
        }
    }
    res
}
///
///END OF PREPROCESSING BLOCK

///EXTRACTION BLOCK
///
//Implements thhe Zhang-Suen thinning algorithm (https://dl.acm.org/doi/epdf/10.1145/357994.358023)
//and applies the 3 morphological operations after the thinngin
fn thin(image: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut image = image.clone();
    let mut changed = true;
    
    while changed {
        changed = false;
        let mut to_del = vec![vec![false; image[0].len()]; image.len()];
        
        // Step 1
        for i in 1..image.len() - 1 {
            for j in 1..image[0].len() - 1 {
                if image[i][j] == 1 && step1(&image, i, j) {
                    to_del[i][j] = true;
                    changed = true;
                }
            }
        }
        //Remove pixels marked in Step 1 
        for i in 0..image.len() {
            for j in 0..image[0].len() {
                if to_del[i][j] {
                    image[i][j] = 0;
                }
            }
        }
        
        // Step 2
        for i in 1..image.len() - 1 {
            for j in 1..image[0].len() - 1 {
                if image[i][j] == 1 && step2(&image, i, j) {
                    to_del[i][j] = true;
                    changed = true;
                }
            }
        }
        //Remove pixels marked in Strp 2
        for i in 0..image.len() {
            for j in 0..image[0].len() {
                if to_del[i][j] {
                    image[i][j] = 0;
                }
            }
        }
    }
   
    remove_h_breaks(&mut image);
    remove_isolated_points(&mut image);
    remove_spikes(&mut image);

    image
}

//Applies the first step conditions of the algothithm
fn step1(image: &Vec<Vec<u8>>, x: usize, y: usize) -> bool {
    let neighbors = get_neighbors(image, x, y);
    let transition_count = count_transitions(&neighbors);
    let neighbor_count = neighbors.iter().sum::<u8>();
    
    neighbor_count >= 2 && neighbor_count <= 6 &&
    transition_count == 1 &&
    neighbors[0] * neighbors[2] * neighbors[4] == 0 &&
    neighbors[2] * neighbors[4] * neighbors[6] == 0
}
//Applies the second step conditions og the algorithm
fn step2(image: &Vec<Vec<u8>>, x: usize, y: usize) -> bool {
    let neighbors = get_neighbors(image, x, y);
    let transition_count = count_transitions(&neighbors);
    let neighbor_count = neighbors.iter().sum::<u8>();
    
    neighbor_count >= 2 && neighbor_count <= 6 &&
    transition_count == 1 &&
    neighbors[0] * neighbors[2] * neighbors[6] == 0 &&
    neighbors[0] * neighbors[4] * neighbors[6] == 0
}

//Returns a vector with the 8 neighbours in the specified order
fn get_neighbors(image: &Vec<Vec<u8>>, x: usize, y: usize) -> Vec<u8> {
    vec![
        image[x - 1][y],     // N
        image[x - 1][y + 1], // NE
        image[x][y + 1],     // E
        image[x + 1][y + 1], // SE
        image[x + 1][y],     // S
        image[x + 1][y - 1], // SW
        image[x][y - 1],     // W
        image[x - 1][y - 1], // NW
    ]
}

//Counts the number of transitions from 0 to 1 (0 is directly followed by 1)
fn count_transitions(neighbors: &Vec<u8>) -> usize {
    let mut count = 0;
    for i in 0..neighbors.len() {
        if neighbors[i] == 0 && neighbors[(i + 1) % neighbors.len()] == 1 {
            count += 1;
        }
    }
    count
}

//Desides whether the pixel is part of an H break by comparing in to two known neighbour H-patterns
fn is_h_break(image: &Vec<Vec<u8>>, x: usize, y: usize) -> bool {
    let neighbors = get_neighbors(image, x, y);

    let reference = vec![
        vec![1, 0, 1, 0, 1, 1, 1, 0], // H pattern
        vec![1, 1, 1, 0, 1, 0, 1, 0], // Rotated H pattern
    ];

    reference.iter().any(|r| {
        neighbors.iter().zip(r.iter()).all(|(x, y)| x == y)
    })
}
//Morphological operation to remove the H breaks
fn remove_h_breaks(image: &mut Vec<Vec<u8>>) {
    let mut changed = true;

    while changed {
        changed = false;
        let mut to_del = vec![vec![false; image[0].len()]; image.len()];

        for i in 1..image.len() - 1 {
            for j in 1..image[0].len() - 1 {
                if is_h_break(&image, i, j) {
                    to_del[i][j] = true;
                    changed = true;
                }
            }
        }

        for i in 0..image.len() {
            for j in 0..image[0].len() {
                if to_del[i][j] {
                    image[i][j] = 0;
                }
            }
        }
    }
}

//Removes the isolated pixels
fn remove_isolated_points(image: &mut Vec<Vec<u8>>) {
    let mut to_del = vec![vec![false; image[0].len()]; image.len()];

    for i in 1..image.len() - 1 {
        for j in 1..image[0].len() - 1 {
            if image[i][j] == 1 && is_isolated(&image, i, j) {
                to_del[i][j] = true;
            }
        }
    }

    for i in 0..image.len() {
        for j in 0..image[0].len() {
            if to_del[i][j] {
                image[i][j] = 0;
            }
        }
    }
}
//Isolated == all the neighbouring pixels are 0
fn is_isolated(image: &Vec<Vec<u8>>, x: usize, y: usize) -> bool {
    let neighbors = get_neighbors(image, x, y);
    neighbors.iter().sum::<u8>() == 0
}

//Removes spikes
fn remove_spikes(image: &mut Vec<Vec<u8>>) {
    let mut to_del = vec![vec![false; image[0].len()]; image.len()];

    for i in 1..image.len() - 1 {
        for j in 1..image[0].len() - 1 {
            if image[i][j] == 1 && is_spike(&image, i, j) {
                to_del[i][j] = true;
            }
        }
    }

    for i in 0..image.len() {
        for j in 0..image[0].len() {
            if to_del[i][j] {
                image[i][j] = 0;
            }
        }
    }
}

//Pixel is a spike == it has exactly one ridge pixel in its vacinity
fn is_spike(image: &Vec<Vec<u8>>, x: usize, y: usize) -> bool {
    let neighbors = get_neighbors(image, x, y);
    neighbors.iter().sum::<u8>() == 1
}
///
///END OF EXTRACTION BLOCK


///POSTPROCESSING BLOCK
///
//Marks each valuable pixel (not a regular ridge) as either an ending, or a bifurcation point.
//Stores this information in a structure and returns a vector of valuale minutia
fn mark_minutia(image: &Vec<Vec<u8>>) -> Vec<Minutia> {
    let mut res = Vec::new();

    for i in 1..image.len() - 1 {
        for j in 1..image[0].len() - 1 {
            if image[i][j] == 1 {
                let neighbors = get_neighbors(image, i, j);
                let neighbor_count = neighbors.iter().sum::<u8>();

                if neighbor_count == 1 {
                    res.push(Minutia {
                        x: i,
                        y: j,
                        minutia_type: MinutiaType::RidgeEnding,
                    });
                } else if neighbor_count == 3 {
                    res.push(Minutia {
                        x: i,
                        y: j,
                        minutia_type: MinutiaType::Bifurcation,
                    });
                }
            }
        }
    }
    res
}

fn euclidean_distance(a: &Minutia, b: &Minutia) -> f64 {
    (((a.x as f64 - b.x as f64).powi(2) + (a.y as f64 - b.y as f64).powi(2)) as f64).sqrt()
}
fn calculate_angle(a: &Minutia, b: &Minutia) -> f64 {
    let delta_x = b.x as f64 - a.x as f64;
    let delta_y = b.y as f64 - a.y as f64;
    delta_y.atan2(delta_x).abs()
}
//Removes the false minutia using Fuzzy rules
fn remove_false_minutia(mut image: Vec<Vec<u8>>, minutia: Vec<Minutia>, distance_threshold: f64, angle_threshold: f64) -> Vec<Vec<u8>> {
    let mut to_del = Vec::new();
    for i in 0..minutia.len() {
        for j in (i + 1)..minutia.len() {
            let mut is_false = false;

            let distance = euclidean_distance(&minutia[i], &minutia[j]);

            // Rule 1: Termination and Bifurcation on the same ridge
            if distance < distance_threshold
                && minutia[i].minutia_type != minutia[j].minutia_type
            {
                is_false = true;
            }

            // Rule 2: Distance between two bifurcations on the same ridge
            if !is_false
                && distance < distance_threshold
                && minutia[i].minutia_type == MinutiaType::Bifurcation
                && minutia[j].minutia_type == MinutiaType::Bifurcation
            {
                is_false = true;
            }

            // Rule 3: Distance between two terminations on the same ridge
            if !is_false
                && distance < distance_threshold
                && minutia[i].minutia_type == MinutiaType::RidgeEnding
                && minutia[j].minutia_type == MinutiaType::RidgeEnding
            {
                is_false = true;
            }

            // Rule 4: Angle
            if !is_false
                && distance < distance_threshold
                && minutia[i].minutia_type == MinutiaType::RidgeEnding
                && minutia[j].minutia_type == MinutiaType::RidgeEnding
            {
                let angle_variation = calculate_angle(&minutia[i], &minutia[j]);

                if angle_variation < angle_threshold {
                    is_false = true;
                }
            }

            if is_false {
                to_del.push(minutia[i].clone());
                to_del.push(minutia[j].clone());
            }
        }
    }

    for m in &to_del {
        image[m.x][m.y] = 0;
    }

    image
}