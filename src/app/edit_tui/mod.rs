mod model;

use std::io::{self, IsTerminal};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::time::Duration;

use crossterm::cursor::Show;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::execute;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame, TerminalOptions, Viewport};
use unicode_width::UnicodeWidthStr;

use crate::app::shell::ShellType;
use crate::catalog::types::AliasCatalog;

pub use model::{EditorMode, EditorResult};
use model::{EditorState, Focus, Modal, TextAction};

const MIN_TERMINAL_ROWS: u16 = 10;
const WIDE_LAYOUT_WIDTH: u16 = 80;
const WIDE_LAYOUT_HEIGHT: u16 = 17;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MouseTarget {
    Focus(Focus),
    Alias(u64),
    ModalChoice(usize),
}

#[derive(Default)]
struct UiAreas(Vec<(Rect, MouseTarget)>);

impl UiAreas {
    fn add(&mut self, area: Rect, target: MouseTarget) {
        self.0.push((area, target));
    }
    fn at(&self, column: u16, row: u16) -> Option<MouseTarget> {
        self.0
            .iter()
            .rev()
            .find(|(area, _)| {
                column >= area.x
                    && column < area.x + area.width
                    && row >= area.y
                    && row < area.y + area.height
            })
            .map(|(_, target)| *target)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run(
    catalog: &AliasCatalog,
    name: Option<&str>,
    mode: EditorMode,
    shell: ShellType,
) -> Result<EditorResult, String> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("interactive editing requires a terminal on stdin and stdout".to_owned());
    }
    let (_, rows) = crossterm::terminal::size()
        .map_err(|error| format!("could not determine terminal size: {error}"))?;
    let height = inline_height(rows).ok_or_else(|| {
        format!(
            "terminal is too short for interactive editing (need at least {MIN_TERMINAL_ROWS} rows)"
        )
    })?;
    let mut state = match mode {
        EditorMode::Single => {
            EditorState::single(catalog, name.ok_or("an alias name is required")?, shell)?
        }
        EditorMode::All => EditorState::all(catalog, shell),
    };
    let options = TerminalOptions {
        viewport: Viewport::Inline(height),
    };
    let mut terminal = match ratatui::try_init_with_options(options) {
        Ok(terminal) => terminal,
        Err(error) => {
            ratatui::restore();
            return Err(format!("could not initialize interactive editor: {error}"));
        }
    };
    if let Err(error) = execute!(terminal.backend_mut(), EnableMouseCapture) {
        ratatui::restore();
        return Err(format!("could not enable mouse input: {error}"));
    }

    let result = catch_unwind(AssertUnwindSafe(|| event_loop(&mut terminal, &mut state)));
    let clear_result = terminal.clear();
    let mouse_result = execute!(terminal.backend_mut(), DisableMouseCapture, Show);
    let restore_result = ratatui::try_restore();
    if let Err(payload) = result {
        resume_unwind(payload);
    }
    clear_result.map_err(|error| format!("could not clear interactive editor: {error}"))?;
    mouse_result.map_err(|error| format!("could not restore mouse input: {error}"))?;
    restore_result.map_err(|error| format!("could not restore terminal: {error}"))?;
    result.unwrap()
}

fn inline_height(rows: u16) -> Option<u16> {
    if rows < MIN_TERMINAL_ROWS {
        return None;
    }
    if rows < 15 {
        Some(rows - 1)
    } else {
        Some(((rows * 3) / 5).clamp(10, 22).min(rows - 3))
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn event_loop(
    terminal: &mut DefaultTerminal,
    state: &mut EditorState,
) -> Result<EditorResult, String> {
    let mut areas = UiAreas::default();
    loop {
        terminal
            .draw(|frame| {
                areas = UiAreas::default();
                render(frame, state, &mut areas);
            })
            .map_err(|error| format!("could not draw interactive editor: {error}"))?;
        if !event::poll(Duration::from_millis(250))
            .map_err(|error| format!("could not poll terminal input: {error}"))?
        {
            continue;
        }
        let result = match event::read()
            .map_err(|error| format!("could not read terminal input: {error}"))?
        {
            Event::Key(key) if key.kind == event::KeyEventKind::Press => handle_key(state, key),
            Event::Mouse(mouse) => handle_mouse(state, mouse, &areas),
            Event::Resize(_, _) => EditorResult::Continue,
            _ => EditorResult::Continue,
        };
        if result != EditorResult::Continue {
            return Ok(result);
        }
    }
}

fn handle_key(state: &mut EditorState, key: KeyEvent) -> EditorResult {
    if state.modal.is_some() {
        return match key.code {
            KeyCode::Esc => {
                state.dismiss_modal();
                EditorResult::Continue
            }
            KeyCode::Left | KeyCode::Up | KeyCode::BackTab => {
                state.move_modal_choice(-1);
                EditorResult::Continue
            }
            KeyCode::Right | KeyCode::Down | KeyCode::Tab => {
                state.move_modal_choice(1);
                EditorResult::Continue
            }
            KeyCode::Enter => state.activate_modal(),
            _ => EditorResult::Continue,
        };
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
        return state.request_save();
    }
    match key.code {
        KeyCode::Tab => state.cycle_focus(false),
        KeyCode::BackTab => state.cycle_focus(true),
        KeyCode::Up
            if state.mode == EditorMode::All
                && matches!(state.focus, Focus::Search | Focus::List) =>
        {
            state.move_selection(-1)
        }
        KeyCode::Down
            if state.mode == EditorMode::All
                && matches!(state.focus, Focus::Search | Focus::List) =>
        {
            state.move_selection(1)
        }
        KeyCode::Up if state.mode == EditorMode::All && is_form_focus(state.focus) => {
            move_form_focus(state, -1)
        }
        KeyCode::Down if state.mode == EditorMode::All && is_form_focus(state.focus) => {
            move_form_focus(state, 1)
        }
        KeyCode::Right
            if state.mode == EditorMode::All
                && matches!(state.focus, Focus::Search | Focus::List) =>
        {
            state.compact_form = true;
            state.focus = Focus::Name;
        }
        KeyCode::Left
            if state.mode == EditorMode::All
                && matches!(state.focus, Focus::Enabled | Focus::Global) =>
        {
            focus_catalog_list(state)
        }
        KeyCode::Left
            if state.mode == EditorMode::All
                && state.focus == Focus::Name
                && state.selected().is_some_and(|draft| draft.name.cursor == 0) =>
        {
            focus_catalog_list(state)
        }
        KeyCode::Left if state.focus.is_text() => state.edit_focused(TextAction::Left),
        KeyCode::Right if state.focus.is_text() => state.edit_focused(TextAction::Right),
        KeyCode::Home if state.focus.is_text() => state.edit_focused(TextAction::Home),
        KeyCode::End if state.focus.is_text() => state.edit_focused(TextAction::End),
        KeyCode::Backspace if state.focus.is_text() => state.edit_focused(TextAction::Backspace),
        KeyCode::Delete if state.focus.is_text() => state.edit_focused(TextAction::Delete),
        KeyCode::Char(character)
            if state.focus.is_text() && !key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            state.edit_focused(TextAction::Insert(character))
        }
        KeyCode::Char(' ') if matches!(state.focus, Focus::Enabled | Focus::Global) => {
            state.toggle_focused()
        }
        KeyCode::Char('?') => state.modal = Some(Modal::Help),
        KeyCode::Char('q') => return state.request_quit(),
        KeyCode::Esc if state.mode == EditorMode::All && state.compact_form => {
            state.compact_form = false;
            state.focus = Focus::List;
        }
        KeyCode::Esc => return state.request_quit(),
        KeyCode::Enter => return activate_focus(state),
        _ => {}
    }
    EditorResult::Continue
}

fn is_form_focus(focus: Focus) -> bool {
    matches!(
        focus,
        Focus::Name
            | Focus::Command
            | Focus::Description
            | Focus::Tags
            | Focus::Enabled
            | Focus::Global
    )
}

fn move_form_focus(state: &mut EditorState, delta: isize) {
    let mut fields = vec![
        Focus::Name,
        Focus::Command,
        Focus::Description,
        Focus::Tags,
        Focus::Enabled,
    ];
    if state.shell == ShellType::Zsh {
        fields.push(Focus::Global);
    }
    let current = fields
        .iter()
        .position(|focus| *focus == state.focus)
        .unwrap_or(0) as isize;
    let next = (current + delta).clamp(0, fields.len() as isize - 1) as usize;
    state.focus = fields[next];
}

fn focus_catalog_list(state: &mut EditorState) {
    state.compact_form = false;
    state.focus = Focus::List;
}

fn activate_focus(state: &mut EditorState) -> EditorResult {
    match state.focus {
        Focus::Enabled | Focus::Global => state.toggle_focused(),
        Focus::Add => state.add_alias(),
        Focus::Delete => state.request_delete(),
        Focus::Save => return state.request_save(),
        Focus::Cancel => return state.request_quit(),
        Focus::List | Focus::Search if state.mode == EditorMode::All => {
            state.compact_form = true;
            state.focus = Focus::Name;
        }
        _ => {}
    }
    EditorResult::Continue
}

fn handle_mouse(state: &mut EditorState, mouse: MouseEvent, areas: &UiAreas) -> EditorResult {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            state.move_selection(-1);
            EditorResult::Continue
        }
        MouseEventKind::ScrollDown => {
            state.move_selection(1);
            EditorResult::Continue
        }
        MouseEventKind::Down(MouseButton::Left) => {
            match areas.at(mouse.column, mouse.row) {
                Some(MouseTarget::Alias(id)) => {
                    state.select(id);
                    if state.compact_layout {
                        state.compact_form = true;
                        state.focus = Focus::Name;
                    } else {
                        state.focus = Focus::List;
                    }
                }
                Some(MouseTarget::Focus(focus)) => {
                    state.focus = focus;
                    if focus.is_text() {
                        return EditorResult::Continue;
                    }
                    return activate_focus(state);
                }
                Some(MouseTarget::ModalChoice(choice)) => {
                    state.set_modal_choice(choice);
                    return state.activate_modal();
                }
                None => {}
            }
            EditorResult::Continue
        }
        _ => EditorResult::Continue,
    }
}

fn render(frame: &mut Frame<'_>, state: &mut EditorState, areas: &mut UiAreas) {
    let area = frame.area();
    let block = Block::default()
        .title(" aliasmgr interactive edit ")
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let rows = Layout::vertical([Constraint::Min(4), Constraint::Length(1)]).split(inner);
    let compact = area.width < WIDE_LAYOUT_WIDTH || area.height < WIDE_LAYOUT_HEIGHT;
    state.compact_layout = compact;
    if state.mode == EditorMode::All && !(compact && state.compact_form) {
        if compact {
            render_catalog_list(frame, state, rows[0], areas);
        } else {
            let columns =
                Layout::horizontal([Constraint::Percentage(34), Constraint::Percentage(66)])
                    .split(rows[0]);
            render_catalog_list(frame, state, columns[0], areas);
            render_form(frame, state, columns[1], areas);
        }
    } else {
        render_form(frame, state, rows[0], areas);
    }
    render_footer(frame, state, rows[1]);
    if let Some(modal) = &state.modal {
        render_modal(frame, modal, area, areas);
    }
}

fn render_catalog_list(
    frame: &mut Frame<'_>,
    state: &EditorState,
    area: Rect,
    areas: &mut UiAreas,
) {
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);
    render_input(
        frame,
        "Search all fields",
        &state.search,
        state.focus == Focus::Search,
        rows[0],
        Focus::Search,
        areas,
    );
    let ids = state.filtered_ids();
    let list_inner = Block::default().borders(Borders::ALL).inner(rows[1]);
    let visible_count = list_inner.height as usize;
    let selected_position = state
        .selected_id
        .and_then(|selected| ids.iter().position(|id| *id == selected))
        .unwrap_or(0);
    let start = selected_position.saturating_sub(visible_count.saturating_sub(1));
    let visible_ids = ids
        .iter()
        .skip(start)
        .take(visible_count)
        .copied()
        .collect::<Vec<_>>();
    let items = visible_ids
        .iter()
        .filter_map(|id| state.drafts.iter().find(|draft| draft.id == *id))
        .map(|draft| {
            let marker = if Some(draft.id) == state.selected_id {
                "› "
            } else {
                "  "
            };
            let item = ListItem::new(format!(
                "{marker}{}",
                if draft.name.value.is_empty() {
                    "<new alias>"
                } else {
                    &draft.name.value
                }
            ));
            if Some(draft.id) == state.selected_id {
                item.style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                item
            }
        })
        .collect::<Vec<_>>();
    let list = List::new(items)
        .block(Block::default().title("Aliases").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(list, rows[1]);
    for (offset, id) in visible_ids.into_iter().enumerate() {
        areas.add(
            Rect {
                x: list_inner.x,
                y: list_inner.y + offset as u16,
                width: list_inner.width,
                height: 1,
            },
            MouseTarget::Alias(id),
        );
    }
    let actions = Rect {
        x: rows[2].x,
        y: rows[2].y,
        width: rows[2].width,
        height: 1,
    };
    frame.render_widget(
        Line::from(vec![
            button(" Add ", state.focus == Focus::Add),
            Span::raw("  "),
            button(" Delete ", state.focus == Focus::Delete),
            Span::raw("  "),
            save_button(
                state.focus == Focus::Save,
                state.validation_errors().is_empty(),
            ),
            Span::raw("  "),
            button(" Cancel ", state.focus == Focus::Cancel),
        ]),
        actions,
    );
    let add_width = 5;
    areas.add(
        Rect {
            width: add_width,
            ..actions
        },
        MouseTarget::Focus(Focus::Add),
    );
    areas.add(
        Rect {
            x: actions.x + add_width + 2,
            width: 8,
            ..actions
        },
        MouseTarget::Focus(Focus::Delete),
    );
    areas.add(
        Rect {
            x: actions.x + 17,
            width: 6,
            ..actions
        },
        MouseTarget::Focus(Focus::Save),
    );
    areas.add(
        Rect {
            x: actions.x + 25,
            width: 8,
            ..actions
        },
        MouseTarget::Focus(Focus::Cancel),
    );
}

fn render_form(frame: &mut Frame<'_>, state: &EditorState, area: Rect, areas: &mut UiAreas) {
    let Some(draft) = state.selected() else {
        let rows = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(area);
        frame.render_widget(
            Paragraph::new("No alias selected. Use Add to create one."),
            rows[0],
        );
        frame.render_widget(
            Line::from(vec![
                save_button(
                    state.focus == Focus::Save,
                    state.validation_errors().is_empty(),
                ),
                Span::raw("  "),
                button(" Cancel ", state.focus == Focus::Cancel),
            ]),
            rows[1],
        );
        areas.add(
            Rect {
                width: 6,
                ..rows[1]
            },
            MouseTarget::Focus(Focus::Save),
        );
        areas.add(
            Rect {
                x: rows[1].x + 8,
                width: 8,
                ..rows[1]
            },
            MouseTarget::Focus(Focus::Cancel),
        );
        return;
    };
    if area.height < WIDE_LAYOUT_HEIGHT {
        render_compact_form(frame, state, draft, area, areas);
        return;
    }
    let global_rows = u16::from(state.shell == ShellType::Zsh);
    let rows = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(1 + global_rows),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);
    render_input(
        frame,
        "Name",
        &draft.name,
        state.focus == Focus::Name,
        rows[0],
        Focus::Name,
        areas,
    );
    render_input(
        frame,
        "Command",
        &draft.command,
        state.focus == Focus::Command,
        rows[1],
        Focus::Command,
        areas,
    );
    render_input(
        frame,
        "Description (optional)",
        &draft.description,
        state.focus == Focus::Description,
        rows[2],
        Focus::Description,
        areas,
    );
    render_input(
        frame,
        "Tags (comma-separated)",
        &draft.tags,
        state.focus == Focus::Tags,
        rows[3],
        Focus::Tags,
        areas,
    );
    let toggle_line = Line::from(vec![
        button(
            if draft.enabled {
                " [x] Enabled "
            } else {
                " [ ] Enabled "
            },
            state.focus == Focus::Enabled,
        ),
        Span::raw("  "),
        if state.shell == ShellType::Zsh {
            button(
                if draft.global {
                    " [x] Global "
                } else {
                    " [ ] Global "
                },
                state.focus == Focus::Global,
            )
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(toggle_line, rows[4]);
    areas.add(
        Rect {
            width: 13.min(rows[4].width),
            ..rows[4]
        },
        MouseTarget::Focus(Focus::Enabled),
    );
    if state.shell == ShellType::Zsh {
        areas.add(
            Rect {
                x: rows[4].x + 15,
                width: 12.min(rows[4].width.saturating_sub(15)),
                ..rows[4]
            },
            MouseTarget::Focus(Focus::Global),
        );
    }
    let actions = rows[6];
    frame.render_widget(
        Line::from(vec![
            save_button(
                state.focus == Focus::Save,
                state.validation_errors().is_empty(),
            ),
            Span::raw("  "),
            button(" Cancel ", state.focus == Focus::Cancel),
        ]),
        actions,
    );
    areas.add(
        Rect {
            width: 6,
            ..actions
        },
        MouseTarget::Focus(Focus::Save),
    );
    areas.add(
        Rect {
            x: actions.x + 8,
            width: 8,
            ..actions
        },
        MouseTarget::Focus(Focus::Cancel),
    );
}

fn render_compact_form(
    frame: &mut Frame<'_>,
    state: &EditorState,
    draft: &model::AliasDraft,
    area: Rect,
    areas: &mut UiAreas,
) {
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);
    render_compact_input(
        frame,
        "Name",
        &draft.name,
        state.focus == Focus::Name,
        rows[0],
        Focus::Name,
        areas,
    );
    render_compact_input(
        frame,
        "Command",
        &draft.command,
        state.focus == Focus::Command,
        rows[1],
        Focus::Command,
        areas,
    );
    render_compact_input(
        frame,
        "Description",
        &draft.description,
        state.focus == Focus::Description,
        rows[2],
        Focus::Description,
        areas,
    );
    render_compact_input(
        frame,
        "Tags",
        &draft.tags,
        state.focus == Focus::Tags,
        rows[3],
        Focus::Tags,
        areas,
    );
    frame.render_widget(
        Line::from(vec![
            button(
                if draft.enabled {
                    " [x] Enabled "
                } else {
                    " [ ] Enabled "
                },
                state.focus == Focus::Enabled,
            ),
            Span::raw("  "),
            if state.shell == ShellType::Zsh {
                button(
                    if draft.global {
                        " [x] Global "
                    } else {
                        " [ ] Global "
                    },
                    state.focus == Focus::Global,
                )
            } else {
                Span::raw("")
            },
        ]),
        rows[4],
    );
    areas.add(
        Rect {
            width: 13.min(rows[4].width),
            ..rows[4]
        },
        MouseTarget::Focus(Focus::Enabled),
    );
    if state.shell == ShellType::Zsh {
        areas.add(
            Rect {
                x: rows[4].x + 15,
                width: 12.min(rows[4].width.saturating_sub(15)),
                ..rows[4]
            },
            MouseTarget::Focus(Focus::Global),
        );
    }
    frame.render_widget(
        Line::from(vec![
            save_button(
                state.focus == Focus::Save,
                state.validation_errors().is_empty(),
            ),
            Span::raw("  "),
            button(" Cancel ", state.focus == Focus::Cancel),
        ]),
        rows[5],
    );
    areas.add(
        Rect {
            width: 6,
            ..rows[5]
        },
        MouseTarget::Focus(Focus::Save),
    );
    areas.add(
        Rect {
            x: rows[5].x + 8,
            width: 8,
            ..rows[5]
        },
        MouseTarget::Focus(Focus::Cancel),
    );
}

fn render_compact_input(
    frame: &mut Frame<'_>,
    label: &str,
    field: &model::InputField,
    focused: bool,
    area: Rect,
    focus: Focus,
    areas: &mut UiAreas,
) {
    let marker = if focused { ">" } else { " " };
    let prefix = format!("{marker} {label}: ");
    let style = if focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let available = area.width.saturating_sub(prefix.width() as u16);
    let (visible, cursor) = input_view(field, available);
    frame.render_widget(
        Line::from(vec![
            Span::styled(prefix.clone(), style),
            Span::raw(visible),
        ]),
        area,
    );
    areas.add(area, MouseTarget::Focus(focus));
    if focused && available > 0 {
        frame.set_cursor_position((area.x + prefix.width() as u16 + cursor, area.y));
    }
}

fn render_input(
    frame: &mut Frame<'_>,
    title: &str,
    field: &model::InputField,
    focused: bool,
    area: Rect,
    focus: Focus,
    areas: &mut UiAreas,
) {
    let border = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border);
    let inner = block.inner(area);
    let (visible, cursor) = input_view(field, inner.width);
    frame.render_widget(Paragraph::new(visible).block(block), area);
    areas.add(area, MouseTarget::Focus(focus));
    if focused && inner.width > 0 {
        frame.set_cursor_position((inner.x + cursor, inner.y));
    }
}

fn button(label: &'static str, focused: bool) -> Span<'static> {
    let style = if focused {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan)
    };
    Span::styled(label, style)
}

fn save_button(focused: bool, valid: bool) -> Span<'static> {
    if valid {
        button(" Save ", focused)
    } else {
        Span::styled(" Save ", Style::default().fg(Color::DarkGray))
    }
}

fn input_view(field: &model::InputField, width: u16) -> (String, u16) {
    if width == 0 {
        return (String::new(), 0);
    }
    let characters = field.value.chars().collect::<Vec<_>>();
    let mut start = 0;
    while start < field.cursor {
        let prefix = characters[start..field.cursor].iter().collect::<String>();
        if UnicodeWidthStr::width(prefix.as_str()) < width as usize {
            break;
        }
        start += 1;
    }
    let visible = characters[start..].iter().collect::<String>();
    let before_cursor = characters[start..field.cursor].iter().collect::<String>();
    let cursor = UnicodeWidthStr::width(before_cursor.as_str()) as u16;
    (visible, cursor.min(width - 1))
}

fn render_footer(frame: &mut Frame<'_>, state: &EditorState, area: Rect) {
    let errors = state.validation_errors();
    let message = errors
        .first()
        .map(String::as_str)
        .or(state.status.as_deref())
        .unwrap_or("Tab move  Space toggle  Ctrl-S save  ? help  q quit");
    let style = if state.status.is_some() || !errors.is_empty() {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    frame.render_widget(
        Paragraph::new(message)
            .style(style)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_modal(frame: &mut Frame<'_>, modal: &Modal, area: Rect, areas: &mut UiAreas) {
    let width = area.width.saturating_sub(2).clamp(1, 64);
    let height = match modal {
        Modal::Help => 12,
        _ => 7,
    }
    .min(area.height.saturating_sub(2));
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(width)) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    };
    frame.render_widget(Clear, popup);
    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    match modal {
        Modal::Help => frame.render_widget(Paragraph::new("Tab / Shift-Tab  Move focus\nArrows             Navigate\nSpace              Toggle\nEnter              Activate\nCtrl-S             Save\nEsc                 Back or cancel\nq                   Quit\nMouse               Select and activate\n\nPress Esc to close help."), inner),
        Modal::Replace { name, choice, .. } => render_choices(frame, inner, &format!("Replace existing alias '{name}'?"), &["Replace", "Keep editing"], *choice, areas),
        Modal::Delete { name, choice, .. } => render_choices(frame, inner, &format!("Delete alias '{name}' from this draft?"), &["Delete", "Cancel"], *choice, areas),
        Modal::DirtyExit { choice } => render_choices(frame, inner, "You have unsaved changes.", &["Save", "Discard", "Cancel"], *choice, areas),
    }
}

fn render_choices(
    frame: &mut Frame<'_>,
    area: Rect,
    prompt: &str,
    labels: &[&'static str],
    choice: usize,
    areas: &mut UiAreas,
) {
    let rows = Layout::vertical([Constraint::Min(2), Constraint::Length(1)]).split(area);
    frame.render_widget(Paragraph::new(prompt).wrap(Wrap { trim: true }), rows[0]);
    let mut spans = Vec::new();
    let mut x = rows[1].x;
    for (index, label) in labels.iter().enumerate() {
        let text = format!(" {label} ");
        let width = text.width() as u16;
        spans.push(Span::styled(
            text,
            if index == choice {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Yellow)
            },
        ));
        spans.push(Span::raw("  "));
        areas.add(
            Rect {
                x,
                y: rows[1].y,
                width,
                height: 1,
            },
            MouseTarget::ModalChoice(index),
        );
        x += width + 2;
    }
    frame.render_widget(Line::from(spans), rows[1]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::types::Alias;
    use crossterm::event::KeyEventKind;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        catalog
            .aliases
            .insert("ll".into(), Alias::new("ls -la".into(), true, false));
        catalog
    }
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new_with_kind(code, KeyModifiers::NONE, KeyEventKind::Press)
    }
    #[test]
    fn automatic_height_preserves_terminal_history_space() {
        assert_eq!(inline_height(9), None);
        assert_eq!(inline_height(10), Some(9));
        assert_eq!(inline_height(40), Some(22));
        assert!(inline_height(24).unwrap() < 24);
    }
    #[test]
    fn long_inputs_scroll_to_keep_the_cursor_visible() {
        let field = model::InputField::new("0123456789".into());
        let (visible, cursor) = input_view(&field, 5);
        assert!(visible.starts_with("6789"));
        assert_eq!(cursor, 4);
    }
    #[test]
    fn keyboard_edits_toggles_and_opens_dirty_exit() {
        let mut state = EditorState::single(&catalog(), "ll", ShellType::Zsh).unwrap();
        state.focus = Focus::Command;
        handle_key(&mut state, key(KeyCode::Char('!')));
        state.focus = Focus::Enabled;
        handle_key(&mut state, key(KeyCode::Char(' ')));
        assert_eq!(state.selected().unwrap().command.value, "ls -la!");
        assert!(!state.selected().unwrap().enabled);
        handle_key(&mut state, key(KeyCode::Char('q')));
        assert!(matches!(state.modal, Some(Modal::DirtyExit { .. })));
    }
    #[test]
    fn all_mode_arrow_keys_navigate_list_form_and_text() {
        let mut catalog = catalog();
        catalog
            .aliases
            .insert("test".into(), Alias::new("true".into(), true, false));
        let mut state = EditorState::all(&catalog, ShellType::Zsh);
        state.focus = Focus::List;
        let first = state.selected_id;

        handle_key(&mut state, key(KeyCode::Down));
        assert_ne!(state.selected_id, first);
        handle_key(&mut state, key(KeyCode::Right));
        assert_eq!(state.focus, Focus::Name);
        assert!(state.compact_form);

        handle_key(&mut state, key(KeyCode::Down));
        assert_eq!(state.focus, Focus::Command);
        handle_key(&mut state, key(KeyCode::Home));
        handle_key(&mut state, key(KeyCode::Right));
        assert_eq!(state.selected().unwrap().command.cursor, 1);
        handle_key(&mut state, key(KeyCode::Up));
        assert_eq!(state.focus, Focus::Name);

        handle_key(&mut state, key(KeyCode::Home));
        handle_key(&mut state, key(KeyCode::Left));
        assert_eq!(state.focus, Focus::List);
        assert!(!state.compact_form);
    }
    #[test]
    fn all_mode_left_from_toggle_returns_to_catalog_list() {
        let mut state = EditorState::all(&catalog(), ShellType::Bash);
        state.compact_form = true;
        state.focus = Focus::Enabled;
        handle_key(&mut state, key(KeyCode::Left));
        assert_eq!(state.focus, Focus::List);
        assert!(!state.compact_form);
    }
    #[test]
    fn mouse_targets_select_and_activate_controls() {
        let mut state = EditorState::single(&catalog(), "ll", ShellType::Zsh).unwrap();
        let mut areas = UiAreas::default();
        areas.add(Rect::new(2, 2, 5, 1), MouseTarget::Focus(Focus::Enabled));
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 3,
            row: 2,
            modifiers: KeyModifiers::NONE,
        };
        handle_mouse(&mut state, mouse, &areas);
        assert!(!state.selected().unwrap().enabled);
    }
    #[test]
    fn wide_and_compact_layouts_render_without_full_screen() {
        for (width, height) in [(100, 22), (50, 10)] {
            let backend = TestBackend::new(width, height);
            let mut terminal = Terminal::new(backend).unwrap();
            let mut state = EditorState::all(&catalog(), ShellType::Bash);
            let mut areas = UiAreas::default();
            terminal
                .draw(|frame| render(frame, &mut state, &mut areas))
                .unwrap();
            assert!(!areas.0.is_empty());
            state.compact_form = true;
            terminal
                .draw(|frame| render(frame, &mut state, &mut areas))
                .unwrap();
        }
    }
    #[test]
    fn zsh_controls_and_every_modal_render_with_mouse_targets() {
        let backend = TestBackend::new(100, 22);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = EditorState::single(&catalog(), "ll", ShellType::Zsh).unwrap();
        let mut areas = UiAreas::default();
        for modal in [
            Modal::Help,
            Modal::Replace {
                source_id: 0,
                target_id: None,
                name: "other".into(),
                choice: 1,
            },
            Modal::Delete {
                id: 0,
                name: "ll".into(),
                choice: 1,
            },
            Modal::DirtyExit { choice: 2 },
        ] {
            state.modal = Some(modal);
            terminal
                .draw(|frame| render(frame, &mut state, &mut areas))
                .unwrap();
        }
        assert!(
            areas
                .0
                .iter()
                .any(|(_, target)| *target == MouseTarget::Focus(Focus::Global))
        );
        assert!(
            areas
                .0
                .iter()
                .any(|(_, target)| matches!(target, MouseTarget::ModalChoice(_)))
        );
    }
}
