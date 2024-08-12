use std::rc::Rc;

mod models;

mod db;
use anyhow::Result;
use db::*;

mod ui;

mod io_utils;
use io_utils::*;

mod navigator;
use navigator::*;

fn main() -> Result<()> {
    // TODO: create database and navigator
    let db = Rc::new(JiraDatabase::new("./db.json")?);
    let mut nav = Navigator::new(db);

    loop {
        // clearscreen::clear().unwrap();

        // 1. get current page from navigator. If there is no current page exit the loop.
        let page = match nav.get_current_page() {
            Some(p) => p,
            None => {
                break Ok(());
            }
        };
        // 2. render page
        if let Err(e) = page.draw_page() {
            eprintln!("failed to render page: {e}");
            wait_for_key_press();
            break Err(e);
        }
        // 3. get user input
        let input = io_utils::get_user_input();
        // 4. pass input to page's input handler
        let action = match page.handle_input(&input.trim()) {
            Err(e) => {
                eprintln!("failed to handle input '{input}': {e}");
                wait_for_key_press();
                break Err(e);
            }
            Ok(a) => a,
        };
        // 5. if the page's input handler returns an action let the navigator process the action
        if let Some(a) = action {
            let action = a.clone();
            if let Err(e) = nav.handle_action(a) {
                eprintln!("failed to handle action '{action:?}': {e}");
                wait_for_key_press();
                break Err(e);
            }
        }
    }
}
