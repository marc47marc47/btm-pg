use std::error::Error;
use std::env;
use tokio_postgres::{NoTls, Row};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Table, Row as TableRow},
    Terminal,
};
use crossterm::{event, terminal::{enable_raw_mode, disable_raw_mode}};
use std::io::stdout;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Read DATABASE_URL from environment variables
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment variables");

    // 2. Connect to PostgreSQL
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // 3. Initialize terminal UI
    let mut stdout = stdout();
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 4. Main event loop
    let mut interval = time::interval(Duration::from_secs(2));
    loop {
        let rows = fetch_pg_stat_all_tables(&client).await?;
        terminal.draw(|f| {
            let size = f.area();

            // Layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(90),
                    ]
                    .as_ref(),
                )
                .split(size);

            // Title
            let title = Paragraph::new("PostgreSQL Table Statistics Monitor").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Dashboard")
                    .title_style(Style::default().fg(Color::Yellow)),
            );
            f.render_widget(title, chunks[0]);

            // Data Table with Headers
            let header = TableRow::new(vec![
                "Schema", "Table", "Index Fetch", "Tuples Inserted",
                "Tuples Updated", "Tuples Deleted", "Hot Updates", "Live Tuples", "Dead Tuples",
            ])
            .style(Style::default().fg(Color::Green).add_modifier(ratatui::style::Modifier::BOLD));

            let table = Table::new(rows, [
                    Constraint::Percentage(15),
                    Constraint::Percentage(15),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                ])
                .header(header)
                .block(Block::default().borders(Borders::ALL).title("pg_stat_all_tables"));

            f.render_widget(table, chunks[1]);
        })?;

        // Refresh interval
        interval.tick().await;

        // Exit condition
        if event::poll(Duration::from_millis(200))? {
            if let event::Event::Key(key) = event::read()? {
                if key.code == event::KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    Ok(())
}

// Fetch pg_stat_all_tables data
async fn fetch_pg_stat_all_tables(client: &tokio_postgres::Client) -> Result<Vec<TableRow>, Box<dyn Error>> {
    let rows = client.query(
        "SELECT schemaname, relname, idx_tup_fetch, n_tup_ins, n_tup_upd, n_tup_del, n_tup_hot_upd, n_live_tup, n_dead_tup FROM pg_stat_all_tables ORDER BY n_tup_ins desc LIMIT 15",
        &[],
    ).await?;

    Ok(rows
        .iter()
        .map(|row| {
            TableRow::new(vec![
                row.get::<_, &str>("schemaname").to_string(),
                row.get::<_, &str>("relname").to_string(),
                row.get::<_, Option<i64>>("idx_tup_fetch").unwrap_or(0).to_string(),
                row.get::<_, Option<i64>>("n_tup_ins").unwrap_or(0).to_string(),
                row.get::<_, Option<i64>>("n_tup_upd").unwrap_or(0).to_string(),
                row.get::<_, Option<i64>>("n_tup_del").unwrap_or(0).to_string(),
                row.get::<_, Option<i64>>("n_tup_hot_upd").unwrap_or(0).to_string(),
                row.get::<_, Option<i64>>("n_live_tup").unwrap_or(0).to_string(),
                row.get::<_, Option<i64>>("n_dead_tup").unwrap_or(0).to_string(),
            ])
        })
        .collect())
}

