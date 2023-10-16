use ratatui::{
    prelude::*,
    style::Color,
    style::Style,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    state::LetterState,
    word::Word,
    wordle::{CheckResult, Wordle, MAX_RETRY_TIMES},
};

use super::widgets::{Keyboard, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiState {
    #[default]
    Init,
    Main(MainState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MainState {
    #[default]
    Main,
    Difficult,
    GameOver,
}

pub fn ui<B: Backend>(wordle: &Wordle, frame: &mut Frame<'_, B>, keyboards: &[Vec<Keyboard>]) {
    let area = frame.size();
    match wordle.ui_state {
        UiState::Init => {
            let layout = Layout::new()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(area);
            frame.render_widget(
                Paragraph::new("请输入指定猜测的词!")
                    .style(Style::default().fg(Color::Green))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::LightCyan)),
                    ),
                layout[0],
            );
            frame.render_widget(
                Paragraph::new(wordle.final_word.to_string()).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::LightCyan)),
                ),
                layout[1],
            );

            let ws = wordle.final_word.to_string();
            let ws = format!(
                "输入的猜测词无效, 请按 <Backspace> 键重新输入猜测词! {}",
                ws
            );
            let status = if !wordle.final_word.is_full() {
                "猜测词未设置或者正在输入中!".yellow()
            } else if wordle.is_final_word_valid() {
                "输入的猜测词有效, 请按 <Enter> 键进入游戏!".green()
            } else {
                ws.red()
            };
            let lines = vec!["状态: ".green(), status];
            frame.render_widget(
                Paragraph::new(Line::from(lines)).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::LightCyan)),
                ),
                layout[2],
            );
        }
        UiState::Main(main_state) => {
            let block = Block::new()
                .title("猜词游戏")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightCyan))
                .title_style(Style::default().fg(Color::LightGreen));
            let inner = block.inner(frame.size());
            frame.render_widget(block, frame.size());
            let layout = Layout::default()
                .direction(ratatui::prelude::Direction::Vertical)
                .constraints(vec![
                    Constraint::Min(15),
                    Constraint::Min(9),
                    Constraint::Length(3),
                ])
                .split(inner);
            let gussing_area_block = Block::new()
                .title("猜测词输入区")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::LightCyan))
                .title_style(Style::default().fg(Color::Red));
            let gussing_inner = gussing_area_block.inner(layout[0]);
            frame.render_widget(gussing_area_block, layout[0]);
            let mut row_constraint = (0..6)
                .into_iter()
                .flat_map(|_| [Constraint::Length(1), Constraint::Length(1)])
                .collect::<Vec<_>>();
            row_constraint.push(Constraint::Min(0));
            let table_row_layout = Layout::default()
                .direction(ratatui::prelude::Direction::Vertical)
                .constraints(row_constraint)
                .split(gussing_inner);

            let default_word = Word::whitespace_word_for_render();
            for idx in wordle.history_words.len()..MAX_RETRY_TIMES as usize {
                render_buttons(frame, table_row_layout[idx * 2], &default_word);
            }
            // 历史记录渲染
            for (idx, word) in wordle.history_words.iter().enumerate() {
                render_buttons(frame, table_row_layout[idx * 2], word);
            }

            // 当前行渲染
            render_buttons(
                frame,
                table_row_layout[(wordle.states.current_try_times as usize) * 2],
                &wordle.states.current_word,
            );

            // status render
            let s = format!(
                "-> 状态: [{:?}] 困难模式: [{}], 随机模式: [{}] | ",
                main_state,
                if wordle.opt.difficult { "是" } else { "否" },
                if wordle.opt.random { "是" } else { "否" }
            );
            let status_bar = s.light_green();
            let mut status = vec![status_bar];
            let current_times = wordle.states.current_try_times;
            let current_word = wordle.states.current_word.to_string();
            let mut s = String::new();
            if let Some(result) = wordle.states.current_checked_result {
                match result {
                    CheckResult::InValid => {
                        s = format!("输入的猜测词 [{}] 不在词库中!", current_word);
                    }
                    CheckResult::Success => {
                        status.push(
                            "您猜对了, 请按 <Enter> 键继续, 按 <Esc> 退出游戏!".light_green(),
                        );
                    }
                    CheckResult::Wrong => {
                        s = format!("第 {} 次猜测错误, 请继续!", current_times);
                    }
                    CheckResult::Difficult => {}
                }
            } else {
                s = "等待用户输入猜测词!".to_string();
            }
            status.push(s.light_yellow());
            let footer = Line::from(status);
            frame.render_widget(Paragraph::new(footer), layout[2]);

            // render keyboards
            render_keyboards(wordle, frame, keyboards, layout[1]);
            if let MainState::GameOver = main_state {
                let popup_block = Block::new()
                    .title("提示")
                    .title_style(Style::default().fg(Color::Yellow))
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Black))
                    .style(Style::default().bg(Color::DarkGray).fg(Color::White));
                let paragraph = Paragraph::new("按 <Enter> 键开始新游戏 / 按 <Esc> 键退出游戏!")
                    .alignment(Alignment::Center)
                    .block(popup_block);
                let area = centered_rect(40, 10, frame.size());
                frame.render_widget(Clear, area);
                frame.render_widget(paragraph, area);
            }

            if let MainState::Difficult = main_state {
                // 困难模式下, 需要popup 并按回车键确认退出!
                let block = Block::new()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .borders(Borders::ALL)
                    .title("警告")
                    .title_style(Style::default().fg(Color::Red))
                    .title_alignment(Alignment::Left);
                let area = centered_rect(60, 20, frame.size());
                let inner = block.inner(area);
                // frame.render_widget(Clear, area);
                frame.render_widget(block, area);
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(3)])
                    .split(inner);

                let greens = wordle.get_diffcult_errors_in_green();
                let mut text = String::new();
                if greens.is_empty() {
                    text.push_str("必须包含以下: ");
                    wordle
                        .get_diffcult_errors_in_yellow()
                        .iter()
                        .for_each(|(_, letter)| {
                            text.push(letter.0);
                            text.push(',');
                        });
                    text.push_str("字符");
                } else {
                    text.push_str("以下位置必须一一对应: ");
                    greens.iter().for_each(|(index, letter)| {
                        let s = format!("{} -> {}, ", index + 1, letter.0);
                        text.push_str(s.as_str());
                    });
                }
                frame.render_widget(
                    Paragraph::new(text).style(Style::default().fg(Color::Red)),
                    layout[0],
                );
                frame.render_widget(
                    Paragraph::new("按 <Enter> 键继续")
                        .alignment(Alignment::Center)
                        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
                        .block(
                            Block::new()
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(Color::White)),
                        ),
                    layout[1],
                );
            }
        }
    }
}

fn render_keyboards<B: Backend>(
    wordle: &Wordle,
    frame: &mut Frame<'_, B>,
    keyboards: &[Vec<Keyboard>],
    area: Rect,
) {
    let block = Block::new()
        .title("键盘区")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::LightCyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(inner);
    for i in 0..3 {
        let keys = &keyboards[i];
        let mut constraints: Vec<Constraint> = keys
            .iter()
            .flat_map(|key| [Constraint::Length(key.size), Constraint::Length(1)])
            .collect();
        constraints.push(Constraint::Min(0));
        let key_layout = Layout::new()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(layout[i * 2]);
        for (idx, key) in keys.iter().enumerate() {
            let state = key.letter().map(|key| {
                wordle
                    .cached_letter_states
                    .iter()
                    .filter(|letter| letter.0.eq_ignore_ascii_case(&key))
                    .into_iter()
                    .min_by(|s0, s1| s0.1.cmp(&s1.1))
            });

            let graph = Paragraph::new(key.text.as_str())
                .style(if let Some(Some(state)) = state {
                    get_style(&state.1)
                } else {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                })
                .alignment(Alignment::Center);
            frame.render_widget(graph, key_layout[idx * 2]);
        }
    }
}

fn render_buttons<B: Backend>(frame: &mut Frame<'_, B>, area: Rect, word: &Word) {
    let layout = Layout::default()
        .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
        .direction(Direction::Vertical)
        .split(area);
    let mut col_constraits = (0..5)
        .into_iter()
        .flat_map(|_| [Constraint::Length(3), Constraint::Length(1)])
        .collect::<Vec<_>>();
    col_constraits.push(Constraint::Min(0));
    let table_col_layout = Layout::default()
        .direction(ratatui::prelude::Direction::Horizontal)
        .constraints(col_constraits)
        .split(layout[0]);
    for (index, letter) in word.get_letters().iter().enumerate() {
        frame.render_widget(
            Paragraph::new(letter.0.to_string())
                .style(get_style(&letter.1))
                .alignment(Alignment::Center),
            table_col_layout[index * 2],
        );
    }
}

pub fn get_style(letter_state: &LetterState) -> Style {
    Style::default()
        .bg(match letter_state {
            LetterState::G => Color::Green,
            LetterState::Y => Color::Yellow,
            LetterState::R => Color::Red,
            LetterState::X => Color::DarkGray,
        })
        .fg(match letter_state {
            LetterState::G => Color::White,
            LetterState::Y => Color::White,
            LetterState::R => Color::White,
            LetterState::X => Color::White,
        })
        .bold()
}

pub fn get_theme(letter_state: &LetterState) -> Theme {
    match letter_state {
        LetterState::G => Theme::new(Color::White, Color::Green),
        LetterState::Y => Theme::new(Color::White, Color::Yellow),
        LetterState::R => Theme::new(Color::White, Color::Red),
        LetterState::X => Theme::new(Color::White, Color::Black),
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
