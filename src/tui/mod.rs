use std::{io::{stdout, Result}, path::PathBuf, fmt::Display};

use crossterm::{ExecutableCommand, terminal::{EnterAlternateScreen, enable_raw_mode, LeaveAlternateScreen, disable_raw_mode}, event::{self, KeyEventKind, KeyCode, KeyModifiers, KeyEvent, Event}};
use ratatui::{Terminal, prelude::{CrosstermBackend, Stylize, Layout, Direction, Constraint}, widgets::{Paragraph, List, ListDirection, ListState}, style::{Modifier, Style, Color}, Frame,};

use crate::model::base::Model;

use crate::{model::GLOB_CORPUS, indexer::IndexerTask};

#[derive(PartialEq, Eq)]
pub enum UserMode {
    Normal,
    Query,
    ResultBrowsing
}

impl Display for UserMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserMode::Normal => write!(f, "Normal"),
            UserMode::Query => write!(f, "Query input"),
            UserMode::ResultBrowsing => write!(f, "Result browsing"),
        }
    }
}

struct App {
    search_results: Option<Vec<PathBuf>>,
    query_input: String,
    query_send: String,
    user_mode: UserMode,
    result_list_state: ListState,

}

impl App {
    pub fn new() -> Self {
        App { search_results: None, query_input: "".to_string(), user_mode: UserMode::Normal, result_list_state: ListState::default(), query_send: "".into() }
    }

    pub fn search(&mut self){
        self.query_send = self.query_input.clone();
        let results: Vec<PathBuf> = if self.query_input.len() > 2 && self.query_input.starts_with('"') && self.query_input.ends_with('"') {
            //execute phrase query
            let actual_query = self.query_input.replace('"', "").chars().collect::<Vec<char>>();
            GLOB_CORPUS.get().unwrap().read().unwrap().search_phrase(&actual_query).iter().take(5).map(|m| m.0.clone()).collect()
        }else{
            let actual_query = self.query_input.replace('"', "").chars().collect::<Vec<char>>();
            GLOB_CORPUS.get().unwrap().read().unwrap().search_simple(&actual_query).iter().take(5).map(|m| m.0.clone()).collect()
        };

        if results.is_empty(){
            self.search_results = None;
        }else{
            self.result_list_state.select(Some(0));
            self.user_mode = UserMode::ResultBrowsing;

            self.search_results = Some(results);

        }

    }

}

pub fn tui(indexer: &mut IndexerTask) -> Result<()> {
    init_panic_handler();
    
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    //crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    terminal.clear()?;
    let mut app = App::new();
    let poller = indexer.create_watcher().unwrap();

    loop {
        terminal.draw(|frame| ui(frame, &mut app))?;
        poller.poll().unwrap();
         
        if event::poll(std::time::Duration::from_millis(16))? && handle_event(&mut app, event::read()?){
            break;

        }

    };

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn handle_event(app: &mut App, event: Event) -> bool {
    if let event::Event::Key(key) = event{
        return handle_key_event(app, key);
    }

    false
}

fn handle_key_event(app: &mut App, key: KeyEvent) -> bool {

    if key.kind == KeyEventKind::Press {
        if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c'){
            app.user_mode = UserMode::Normal

        }else if app.user_mode == UserMode::Normal {
            return handle_normal_mode_inputs(app, key);
        }else if app.user_mode == UserMode::Query {
            handle_query_mode_inputs(app, key);
        }else if app.user_mode == UserMode::ResultBrowsing {
            handle_result_mode_inputs(app, key);
        }

    }

    false

}

fn ui(frame: &mut Frame, app: &mut App){

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(96),
            Constraint::Percentage(2),
            Constraint::Percentage(2)

        ])
        .split(frame.size());

    frame.render_widget(Paragraph::new(format!("Query: {}", app.query_input)).white().on_black(), layout[1]);
    frame.render_widget(Paragraph::new(format!("Mode: {}", app.user_mode)).white().on_black(), layout[2]);

    if let Some(results) = &app.search_results {
        let list = List::new(results.clone().iter().map(|r| r.display().to_string().replace('\n', "")).collect::<Vec<String>>())
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        frame.render_stateful_widget(list, layout[0], &mut app.result_list_state);
    }else if app.query_send.is_empty(){
        frame.render_widget(Paragraph::new("").white().on_black(), layout[0]);
    }else{
        frame.render_widget(Paragraph::new(format!("No results found for the query \"{}\"", app.query_input)).white().on_black(), layout[0]);
    }
}


pub fn init_panic_handler(){
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info|{
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        original_hook(panic_info);

    }))

}

fn handle_normal_mode_inputs(app: &mut App, key: KeyEvent) -> bool {
    if key.code == KeyCode::Char('q'){
        return true;
    }
    if key.code == KeyCode::Char('j'){
        app.user_mode = UserMode::Query;
        return false;
    }
    if key.code == KeyCode::Char('k'){
        app.user_mode = UserMode::ResultBrowsing;
    }
    false

}

fn handle_query_mode_inputs(app: &mut App, key: KeyEvent){
    if key.code == KeyCode::Enter && !app.query_input.is_empty() {
            app.search();
            return;
    }
    if let KeyCode::Char(ch) = key.code {
        app.query_input.push(ch);
        return;
    }

    if key.code == KeyCode::Backspace && !app.query_input.is_empty() {
        app.query_input.pop().unwrap();
    }

}

fn handle_result_mode_inputs(app: &mut App, key: KeyEvent){
    if app.search_results.is_none() {
        return;
    }

    let results = app.search_results.as_ref().unwrap();

    let n = match app.result_list_state.selected() {
        Some(idx) => {
            idx
        },
        None => {
            if results.is_empty(){
                return;
            }

            app.result_list_state.select(Some(0));
            return;

        },
    };

    if key.code == KeyCode::Char('j') {
        let idx = if n + 1 >= results.len() {
            0
        }else{
            n + 1
        };

        app.result_list_state.select(Some(idx));

        return;
    }

    if key.code == KeyCode::Char('k') {
        let idx = if n == 0 {
           results.len() - 1 
        }else{
            n - 1
        };

        app.result_list_state.select(Some(idx));

        return;
    }

    if key.code != KeyCode::Enter {
        return;
    }


    open::that(results.get(n).unwrap()).unwrap();
}

