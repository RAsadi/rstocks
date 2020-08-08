use std::collections::hash_map::HashMap;
use std::env::args;
use std::io;
use std::io::Read;
use std::{thread, time};

use termion::{async_stdin, event::Key, input::TermRead, raw::IntoRawMode};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols::{Marker, DOT},
    text::{Span, Spans},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Row, Table, Tabs},
    Terminal,
};

use util::{LoopingIndex, SortedBTreeMap};

mod api;
mod util;

fn fetch_quotes(
    historical_quotes: &mut SortedBTreeMap,
    quotes: &mut HashMap<String, api::Quote>,
    tickers: &Vec<String>,
) {
    for ticker in tickers {
        let res = api::fetch_quote(&ticker);
        match res {
            Ok(r) => {
                if quotes.contains_key(ticker) {
                    historical_quotes.insert(ticker.to_string(), quotes[ticker].to_chartable());
                }
                quotes.insert(ticker.to_string(), r);
            }
            Err(e) => println!("could retrieve quote with ticker {}, error: {}", ticker, e),
        }
    }
}

fn main() -> Result<(), io::Error> {
    // Argument parsing
    let mut args: Vec<String> = args().collect();
    args.remove(0); // Remove file name

    // Setting up terminal
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut asi = async_stdin();
    let normal_style = Style::default().fg(Color::White);
    let pos_style = Style::default().fg(Color::Green);
    let neg_style = Style::default().fg(Color::Red);

    let mut quotes: HashMap<String, api::Quote> = HashMap::new();
    let mut historical_quotes: SortedBTreeMap = SortedBTreeMap::new(32);
    let sleep_time = time::Duration::from_secs(1);
    let mut tab_index = LoopingIndex::new(0);
    let headers = api::Quote::get_table_headers();
    loop {
        // Fetching quotes
        fetch_quotes(&mut historical_quotes, &mut quotes, &args);

        tab_index.max_size = quotes.len() + 1;
        // Drawing UI
        terminal.draw(|f| {
            // Define sizes
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            /*****************
             * Tabs
             *****************/
            let mut titles = vec!["Summary"];
            for (k, _) in historical_quotes.get_btree_map().iter() {
                titles.push(k);
            }
            let tab_titles = titles.iter().cloned().map(Spans::from).collect();
            let tabs = Tabs::new(tab_titles)
                .block(Block::default().title("Tabs").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .select(tab_index.index)
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(DOT);
            f.render_widget(tabs, chunks[0]);

            /*****************
             * Summary Table
             *****************/
            if tab_index.index == 0 {
                // Set colors and define rows
                let mut quote_rows = Vec::new();
                for (_, quote) in quotes.iter() {
                    let row = match quote.get_state() {
                        api::QuoteState::POSITIVE => Row::StyledData(quote.as_row().into_iter(), pos_style),
                        api::QuoteState::NEGATIVE => Row::StyledData(quote.as_row().into_iter(), neg_style),
                        api::QuoteState::NEUTRAL => Row::StyledData(quote.as_row().into_iter(), normal_style),
                    };
                    quote_rows.push(row);
                }
                // Draw table
                let table = Table::new(headers.iter(), quote_rows.into_iter())
                    .block(Block::default().title("Stocks").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .widths(&[
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                    ]);
                f.render_widget(table, chunks[1]);
            } else if tab_index.index < tab_index.max_size {
                let ticker = titles[tab_index.index];
                let historical_data = &historical_quotes.get_btree_map()[ticker];

                // Get some y-bounds for the graph
                let min_value = historical_quotes.get_min(ticker.to_string());
                let max_value = historical_quotes.get_max(ticker.to_string());
                let diff = (max_value - min_value) / 8.0;
                let min_value = (min_value - diff).floor();
                let max_value = (max_value + diff).ceil();
                // TODO: Generate x and y-axis labels

                let datasets = vec![Dataset::default()
                    .name("price")
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Cyan))
                    .data(historical_data)];

                let min_time = historical_quotes.min_time(ticker.to_string());
                let max_time = historical_quotes.max_time(ticker.to_string());
                let chart = Chart::new(datasets)
                    .block(
                        Block::default()
                            .title(Span::styled(
                                ticker,
                                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                            ))
                            .borders(Borders::ALL),
                    )
                    .x_axis(
                        Axis::default()
                            .title("X Axis")
                            .style(Style::default().fg(Color::Gray))
                            .labels(vec![
                                Span::styled(
                                    format!("{}", min_time.format("%H:%M:%S")),
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(
                                    format!("{}", max_time.format("%H:%M:%S")),
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                            ])
                            .bounds([min_time.timestamp() as f64, max_time.timestamp() as f64]),
                    )
                    .y_axis(
                        Axis::default()
                            .title("Y Axis")
                            .style(Style::default().fg(Color::Gray))
                            .labels(vec![
                                Span::styled(format!("{}", min_value), Style::default().add_modifier(Modifier::BOLD)),
                                Span::styled(format!("{}", max_value), Style::default().add_modifier(Modifier::BOLD)),
                            ])
                            .bounds([min_value, max_value]),
                    );
                f.render_widget(chart, chunks[1]);
            } else {
                f.render_widget(Block::default().title("Unknown").borders(Borders::ALL), chunks[1]);
            }
        })?;

        // Event handling
        for k in asi.by_ref().keys() {
            match k.unwrap() {
                Key::Char('q') => {
                    // Clear the terminal before exit so as not to leave
                    // a mess.
                    terminal.clear()?;
                    return Ok(());
                }
                Key::Right => tab_index.next(),
                Key::Left => tab_index.previous(),
                // Otherwise, throw them away.
                _ => (),
            }
        }

        // Wait for 5 seconds
        thread::sleep(sleep_time);
    }
}
