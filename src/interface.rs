use std::fmt::Display;
use std::io::Write;

use colored::*;
use console::{Key, Term};
use hottext::HotText;

use crate::colors;

pub trait Comma
where
    Self: Display,
{
    /// Formats number with commas separating every three decimal places
    /// # Examples
    /// ```
    /// let s = 6000.commas();
    /// assert_eq(s, "6,000");
    /// ```
    fn commas(&self) -> String {
        let raw_display = format!("{}", self);
        raw_display
            .chars()
            .rev()
            .fold((String::new(), 0), |(mut output, mut length), char| {
                length += 1;
                if length % 3 == 1 && length >= 3 {
                    output.push(',');
                }
                output.push(char);

                (output, length)
            })
            .0
            .chars()
            .rev()
            .collect()
    }
}

impl Comma for u8 {}
impl Comma for u16 {}
impl Comma for u32 {}
impl Comma for u64 {}
impl Comma for i8 {}
impl Comma for i16 {}
impl Comma for i32 {}
impl Comma for i64 {}

pub struct Context {
    pub hottext: HotText<rand::rngs::ThreadRng>,
    pub term: Term,
    pub rng: rand::rngs::ThreadRng,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            hottext: HotText::default(),
            term: Term::stdout(),
            rng: rand::thread_rng(),
        }
    }
}

/// Asks the user to type one of N choices.
pub fn read_choice<'a>(ctx: &mut Context, prompt: &str, choices: &[&'a str]) -> usize {
    ctx.term
        .write_line(&format!("{} ({})", prompt, choices.join(", ")))
        .unwrap();
    loop {
        ctx.term.write_all(b"? ").unwrap();
        if let Ok(mut input) = ctx.term.read_line() {
            input = input.trim().to_lowercase();
            if let Some(index) = choices.iter().position(|&option| option == input) {
                break index;
            }
        }
        println!("That's not a valid choice.");
    }
}

/// Prompts the user with a menu to select one of N choices.
pub fn get_choice<'a>(ctx: &mut Context, prompt: &str, choices: &[&'a str]) -> usize {
    ctx.term.write_line(prompt).unwrap();

    let mut selection = 0;
    let last_index = choices.len() - 1;
    ctx.term.hide_cursor().unwrap();
    loop {
        for (index, option) in choices.iter().enumerate() {
            let prefix = if index == selection { '>' } else { ' ' };
            ctx.term
                .write_line(
                    &format!("{} {}", prefix, option)
                        .color(colors::INPUT)
                        .to_string(),
                )
                .unwrap();
        }

        match ctx.term.read_key().unwrap() {
            Key::Enter | Key::Char('e') => {
                ctx.term.show_cursor().unwrap();
                break selection;
            }
            Key::ArrowUp | Key::Char('w') => selection = selection.saturating_sub(1),
            Key::ArrowDown | Key::Char('s') => selection = (selection + 1).min(last_index),
            Key::Home => selection = 0,
            Key::End => selection = last_index,
            _ => {}
        }
        ctx.term.clear_last_lines(choices.len()).unwrap();
    }
}

/// Prompts the user to press any key to continue
pub fn wait_any_key(ctx: &mut Context) {
    ctx.term.hide_cursor().unwrap();
    ctx.term
        .write_line(
            "Press any key to continue..."
                .color(colors::INPUT)
                .to_string()
                .as_ref(),
        )
        .unwrap();
    ctx.term.read_key().unwrap();
    ctx.term.show_cursor().unwrap();
}

/// Inserts a blank line into stdout
pub fn spacer(ctx: &mut Context) {
    ctx.term.write_line("").unwrap();
}
