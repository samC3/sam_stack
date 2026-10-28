use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List as ListWidget, ListItem, ListState, Paragraph, Wrap};

use crate::app::{App, List, Pane};
use crate::sessions::Row;
use crate::todo::{Item, Status};

pub const HELP: &str = " ←/→ pane  ↑/↓ move  tab today/backlog  enter open  r refresh  q quit";

fn state_color(state: &str) -> Color {
    match state {
        "working" => Color::Yellow,
        "idle" | "done" => Color::Green,
        "blocked" => Color::Red,
        _ => Color::DarkGray,
    }
}

fn label(row: &Row) -> String {
    match row {
        Row::Project(name) => name.clone(),
        Row::Slug(slug) => format!("  {slug}"),
        Row::Session { depth, name, .. } => format!("{}{name}", "  ".repeat(depth + 2)),
    }
}

fn row_line(row: &Row, width: usize) -> Line<'static> {
    let text = format!("{:width$}", label(row));
    match row {
        Row::Project(_) => Line::from(text.bold()),
        Row::Slug(_) => Line::from(text.cyan()),
        Row::Session { state, returned, .. } => Line::from(vec![
            Span::raw(text),
            Span::raw("  "),
            Span::styled(format!("{state:8}"), Style::new().fg(state_color(state))),
            Span::raw(format!(" return: {returned}")),
        ]),
    }
}

fn item_line(item: &Item) -> Line<'static> {
    let (glyph, style) = match item.status {
        Status::Todo => ("•", Style::new()),
        Status::Done => ("✓", Style::new().fg(Color::Green)),
        Status::Moved => (">", Style::new().fg(Color::DarkGray)),
        Status::Dropped => (
            "✗",
            Style::new().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT),
        ),
        Status::Note => ("–", Style::new().italic()),
    };
    let mut spans = vec![Span::styled(format!("{glyph} {}", item.text), style)];
    if let Some(project) = &item.project {
        spans.push(format!("  [{project}]").dark_gray());
    }
    if let Some(slug) = &item.slug {
        spans.push(format!(" → {slug}").cyan());
    }
    if item.details.is_some() {
        spans.push(" +".magenta());
    }
    if let Some(to) = &item.moved {
        spans.push(format!(" (to {to})").dark_gray());
    }
    Line::from(spans)
}

fn selection(focused: bool, at: usize) -> ListState {
    ListState::default().with_selected(focused.then_some(at))
}

fn list<'a>(items: Vec<ListItem<'a>>, block: Block<'a>) -> ListWidget<'a> {
    ListWidget::new(items)
        .block(block)
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
}

fn sessions(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::bordered().title(" Sessions ");
    if app.snap.rows.is_empty() {
        let hint = Paragraph::new("No sessions. The supervise skill starts them.").dark_gray();
        frame.render_widget(hint.block(block), area);
        return;
    }
    let width = app
        .snap
        .rows
        .iter()
        .map(|r| label(r).chars().count())
        .max()
        .unwrap_or(0);
    let items = app
        .snap
        .rows
        .iter()
        .map(|r| ListItem::new(row_line(r, width)))
        .collect();
    let mut state = selection(app.focus == Pane::Sessions, app.row);
    frame.render_stateful_widget(list(items, block), area, &mut state);
}

fn tab(text: String, active: bool) -> Span<'static> {
    if active {
        Span::styled(text, Style::new().bold().underlined())
    } else {
        Span::styled(text, Style::new().dark_gray())
    }
}

fn todo(frame: &mut Frame, app: &App, area: Rect) {
    let todo = &app.snap.todo;
    let day = match (&todo.date, todo.today) {
        (Some(date), true) => format!("Today {date}"),
        (Some(date), false) => format!("{date} (today not started)"),
        (None, _) => "Today".into(),
    };
    let title = Line::from(vec![
        Span::raw(" "),
        tab(day, app.list == List::Today),
        Span::raw("  "),
        tab(format!("Backlog ({})", todo.backlog.len()), app.list == List::Backlog),
        Span::raw(" "),
    ]);
    let mut area = area;
    if app.list == List::Today && !todo.now.is_empty() {
        let height = u16::try_from(todo.now.len() + 2).unwrap_or(u16::MAX);
        let [now, rest] = Layout::vertical([Constraint::Length(height), Constraint::Fill(1)]).areas(area);
        let lines: Vec<Line> = todo.now.iter().map(|l| Line::raw(l.clone())).collect();
        frame.render_widget(Paragraph::new(lines).block(Block::bordered().title(" Now ")), now);
        area = rest;
    }
    let block = Block::bordered().title(title);
    let items = app.items();
    if items.is_empty() {
        frame.render_widget(Paragraph::new("Nothing here.").dark_gray().block(block), area);
        return;
    }
    let lines = items.iter().map(|i| ListItem::new(item_line(i))).collect();
    let mut state = selection(app.focus == Pane::Todo, app.item);
    frame.render_stateful_widget(list(lines, block), area, &mut state);
}

pub fn draw(frame: &mut Frame, app: &App) {
    let [body, help] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());
    let direction = if body.width >= 100 {
        Direction::Horizontal
    } else {
        Direction::Vertical
    };
    let [left, right] = Layout::new(direction, [Constraint::Percentage(50); 2]).areas(body);
    sessions(frame, app, left);
    todo(frame, app, right);
    frame.render_widget(Paragraph::new(HELP).dark_gray(), help);
    if let Some(popup) = &app.popup {
        let area = body.inner(Margin::new(body.width / 10, body.height / 8));
        frame.render_widget(Clear, area);
        let block = Block::bordered()
            .title(format!(" {} ", popup.title))
            .title_bottom(" esc close ");
        frame.render_widget(
            Paragraph::new(popup.text.clone())
                .wrap(Wrap { trim: false })
                .block(block),
            area,
        );
    }
}

#[cfg(test)]
mod tests;
