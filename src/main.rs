mod schema;
mod loads;
mod selects;
mod utils;
mod config;
mod sql;
mod label;
mod menu;

use loads::{ load_brands, load_models, load_references, load_years };
use label::{ Label };
use menu::{ MainMenu, MaintMenu, LoadMenu };
use utils::{ clear_screen, press_key_continue };
use config::{ setup_db, update_status, select_status };
use rusqlite::{ Connection, Result };
use owo_colors::OwoColorize;
use inquire::Select;
use inquire::ui::{ RenderConfig, Styled, Color };

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open("fipe_rs.db")?;

    loop {
        clear_screen();

        let config = select_status(&conn)?;
        let db_status = match config.db_status.as_str() {
            "empty" => "Empty".italic().to_string(),
            "stable" => "Stable".bright_green().to_string(),
            _ => "Outdated".bright_red().blink_fast().to_string(),
        };
        let last_update = config.last_update.unwrap_or_else(|| "Never".to_string());

        (Label::Header { db_status: &db_status, last_update: &last_update }).log();

        let options = vec![MainMenu::Loads, MainMenu::Maintenance, MainMenu::Exit];
        let render_config = RenderConfig::default()
            .with_prompt_prefix(Styled::new(""))
            .with_highlighted_option_prefix(Styled::new("> ").with_fg(Color::LightGreen));
        let main_ans = Select::new("Main Menu", options).with_render_config(render_config).prompt();

        match main_ans {
            Ok(MainMenu::Loads) => run_loads(&conn).await?,
            Ok(MainMenu::Maintenance) => run_maint(&conn).await?,
            Ok(MainMenu::Exit) | Err(_) => {
                break;
            }
        }
    }

    Ok(())
}

async fn run_loads(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let options = vec![
            LoadMenu::LoadRefs,
            LoadMenu::LoadBrands,
            LoadMenu::LoadModels,
            LoadMenu::LoadYears,
            LoadMenu::Back
        ];
        let render_config = RenderConfig::default()
            .with_prompt_prefix(Styled::new(""))
            .with_highlighted_option_prefix(Styled::new("> ").with_fg(Color::LightGreen));
        let ans = Select::new("Loads", options).with_render_config(render_config).prompt()?;
        match ans {
            LoadMenu::LoadRefs => load_references(conn).await?,
            LoadMenu::LoadBrands => load_brands(conn).await?,
            LoadMenu::LoadModels => load_models(conn).await?,
            LoadMenu::LoadYears => load_years(conn).await?,
            LoadMenu::Back => {
                break;
            }
        }
        press_key_continue();
    }
    Ok(())
}

async fn run_maint(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let options = vec![MaintMenu::RecreateDatabase, MaintMenu::CheckUpdates, MaintMenu::Back];
        let render_config = RenderConfig::default()
            .with_prompt_prefix(Styled::new(""))
            .with_highlighted_option_prefix(Styled::new("> ").with_fg(Color::LightGreen));
        let ans = Select::new("Maintenance", options).with_render_config(render_config).prompt()?;

        match ans {
            MaintMenu::RecreateDatabase => {
                setup_db(conn, true)?;
                update_status(conn, "stable")?;
            }
            MaintMenu::CheckUpdates => println!("(Em implementação...)"),
            MaintMenu::Back => {
                break;
            }
        }
        press_key_continue();
    }
    Ok(())
}
