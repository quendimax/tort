use clap::Parser;
use colored::*;
use miette::{MietteDiagnostic, Result};
use rand::{self, seq::SliceRandom};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::cell::RefCell;
use std::time::Instant;

use crate::args::Args;
use crate::syntax::*;
use crate::lexis::Token;

pub struct QuizMachine {
    inner: RefCell<QuizMachineInner>
}

struct QuizMachineInner {
    quests: Vec<Line>,
    random: bool,
    readline: DefaultEditor,
    stats: AnswerStatistic,
    prev_was_comment: bool
}

#[derive(Clone)]
struct AnswerStatistic {
    right_answers: usize,
    wrong_answers: usize,
    to_run_tests: usize,
    all_tests: usize,
    done_tests: usize,
    start_time: Instant
}

impl AnswerStatistic {
    pub fn new(number_of_tests: usize, start_time: Instant) -> Self {
        Self {
            right_answers: 0,
            wrong_answers: 0,
            to_run_tests: number_of_tests,
            all_tests: 0,
            done_tests: 0,
            start_time
        }
    }

    pub fn print_headnote(&self) {
        println!("{}", str::repeat("=", 80).blue());
        println!("Starting of {} tests from {}", self.to_run_tests, self.all_tests);
        println!("{}\n", str::repeat("=", 80).blue());
    }

    pub fn print_footnote(&self) {
        println!("{}", str::repeat("=", 80).blue());
        println!("Done {} tests from {}", format!("{}", self.done_tests).bold(), format!("{}", self.all_tests).bold());
        println!("Elapsed time: {}\n", format!("{:?}", self.start_time.elapsed()).bold());
        let right_answers = format!("{}", self.right_answers).bold();
        let wrong_answers = format!("{}", self.wrong_answers).bold();
        let right_percent = format!("{:.1}", self.right_answers as f32 / self.done_tests as f32 * 100.).bold();
        let wrong_percent = format!("{:.1}", self.wrong_answers as f32 / self.done_tests as f32 * 100.).bold();
        println!("{} {} ({}%)", "Right answers:".green(), right_answers, right_percent);
        println!("{} {} ({}%)", "Wrong answers:".red(), wrong_answers, wrong_percent);
        println!("{}", str::repeat("=", 80).blue());
    }
}

impl QuizMachine {
    pub fn new(random: bool, number_of_tests: usize, start_time: Instant) -> Self {
        Self {
            inner: RefCell::new(QuizMachineInner {
                quests: Vec::new(),
                random,
                readline: DefaultEditor::new().unwrap(),
                stats: AnswerStatistic::new(number_of_tests, start_time),
                prev_was_comment: false
            })
        }
    }

    pub fn append(&self, lines: &mut Vec<Line>) {
        self.inner.borrow_mut().quests.append(lines);
    }
    
    fn pre_run(&self) -> Vec<Line> {
        let mut first_line = 0;
        let mut inner = self.inner.borrow_mut();
        let mut new_stats = inner.stats.clone();
        let mut new_random = inner.random;
        let mut is_first_pub_comment = true;
        let mut is_first_line_of_first_pub_comment = true;
        for line in &inner.quests {
            match line {
                Line::PubComment(token) => {
                    if token.span().start == 0 {
                        first_line += 1;
                        continue;
                    }
                    let prev_random = inner.random;
                    const ARGS_PATTERN: &'static str = "ARGS:";
                    if !prev_random && token.spelling().starts_with(ARGS_PATTERN) {
                        let arg_line = token.spelling()[ARGS_PATTERN.len()..].trim();
                        let args = Args::parse_from(arg_line.split_whitespace());
                        new_random = args.random;
                        first_line += 1;
                        continue;
                    }
                    if is_first_pub_comment {
                        if is_first_line_of_first_pub_comment {
                            println!("{}", str::repeat("=", 80).blue());
                            is_first_line_of_first_pub_comment = false;
                        }
                        println!("{}", token.spelling().blue());
                        first_line += 1;
                    }
                },
                Line::ComplexStmt { text: _, comment: _ } => {
                    new_stats.all_tests += 1;
                    is_first_pub_comment = false;
                }
                Line::PlainStmt { text: _, comment: _ } => {
                    new_stats.all_tests += 1;
                    is_first_pub_comment = false;
                },
                Line::TranslationStmt { original: _, translation: _, comment: _ } => {
                    new_stats.all_tests += 1;
                    is_first_pub_comment = false;
                },
                Line::Empty => {
                    is_first_pub_comment = false;
                }
            }
        }
        inner.stats = new_stats;
        let mut lines: Vec<Line> = inner.quests[first_line..].into();
        if new_random {
            let mut rng = rand::thread_rng();
            lines.shuffle(&mut rng);
            inner.random = new_random;
        }
        lines
    }

    pub fn run(&self) -> Result<()> {
        let lines = self.pre_run();
        let mut inner = self.inner.borrow_mut();
        
        inner.stats.print_headnote();
        for line in lines {
            match line {
                Line::Empty => continue,
                Line::PubComment(token) => {
                    if !inner.random {
                        let spelling = token.spelling();
                        inner.print_comment(spelling);
                    }
                    continue;
                },
                Line::PlainStmt { text, comment } => {
                    let original = aid::spell_text(&text);
                    let comment = comment.as_ref().map(|c| c.spelling());
                    if inner.ask("Repeat", "Type", &original, &original, comment)? {
                        break;
                    }
                },
                Line::ComplexStmt { text, comment } => {
                    let question = text.spell_question().yellow();
                    let right_answer = text.spell_answer();
                    let comment = comment.as_ref().map(|c| c.spelling());
                    if inner.ask("Fill gaps", "Your answer", &question, &right_answer, comment)? {
                        break;
                    }
                },
                Line::TranslationStmt { original, translation, comment } => {
                    let original = aid::spell_text(&original);
                    let translation = aid::spell_text(&translation);
                    let comment = comment.as_ref().map(|c| c.spelling());
                    if inner.ask("Translate", "Your answer", &original, &translation, comment)? {
                        break;
                    }
                }
            }
            if inner.stats.to_run_tests == inner.stats.done_tests {
                break;
            } else {
            }
        }
        inner.stats.print_footnote();
        Ok(())
    }
}

impl QuizMachineInner {
    fn print_comment(&mut self, comment: &str) {
        println!(" {}", comment.blue());
        self.prev_was_comment = true;
    }
    
    fn ask(&mut self, quest_prompt: &str, answer_prompt: &str, question: &str, right_answer: &str, comment: Option<&str>)
        -> Result<bool>
    {
        if self.prev_was_comment {
            println!("{}\n", str::repeat("_", 80).blue());
        }
        let prompt_width = std::cmp::max(quest_prompt.len(), answer_prompt.len());
        let prompt_width = std::cmp::max(prompt_width, " ---> ".len()) + 1;
        let quest_prompt = format!("{quest_prompt}:").bold();
        let answer_prompt = format!("{answer_prompt}:").bold();

        print!("{quest_prompt:>prompt_width$}  {question}");
        if let Some(comment) = comment {
            print!("   {}", format!("({comment})").blue());
        }
        println!();

        let answer_prompt = format!("{answer_prompt:>prompt_width$}  ");
        let Some(answer) = self.readline(&answer_prompt)? else { return Ok(true) };
        if answer != right_answer {
            println!("{:>prompt_width$}  {}", "---> ".bold(), "Wrong".red().bold());
            let diff = prettydiff::diff_chars(&answer, right_answer);
            println!("{:>prompt_width$}  {}", "Right:".bold(), diff);
            self.stats.wrong_answers += 1;
        } else {
            println!("{:>prompt_width$}  {}", "---> ".bold(), "Right".green().bold());
            self.stats.right_answers += 1;
        }
        self.stats.done_tests += 1;
        self.prev_was_comment = false;
        println!("{}\n", str::repeat("_", 80).blue());
        Ok(false)
    }
    
    fn readline(&mut self, prompt: &str) -> miette::Result<Option<String>> {
        match self.readline.readline(prompt) {
            Ok(line) => Ok(Some(line)),
            Err(ReadlineError::Interrupted) => Ok(None),
            Err(err) => Err(MietteDiagnostic::new(format!("input error occured: {}", err))
                .with_severity(miette::Severity::Error).into())
        }
    }
}

trait Quiz {
    fn spell_question(&self) -> String;
    fn spell_answer(&self) -> String;
}

impl Quiz for Vec<Lexeme> {
    fn spell_question(&self) -> String {
        let mut spelling = String::new();
        for lexeme in self {
            spelling += &lexeme.spell_question();
        }
        spelling.into()
    }

    fn spell_answer(&self) -> String {
        let mut spelling = String::new();
        for lexeme in self {
            spelling += &lexeme.spell_answer();
        }
        spelling.into()
    }
}

impl Quiz for Lexeme {
    fn spell_question(&self) -> String {
        match self {
            Lexeme::Normal(token) => token.spelling().to_owned(),
            Lexeme::Orthogram(orthogram) => orthogram.spell_question()
        }
    }

    fn spell_answer(&self) -> String {
        match self {
            Lexeme::Normal(token) => token.spelling().to_owned(),
            Lexeme::Orthogram(orthogram) => orthogram.spell_answer()
        }
    }
}

impl Quiz for Orthogram {
    fn spell_question(&self) -> String {
        let mut rnd = rand::thread_rng();
        match self {
            Orthogram::Gap { answer: _, comment } => {
                if let Some(comment) = comment {
                    let comment = format!("({})", aid::spell_text(comment)).blue();
                    format!("{}{}", "_".bold().yellow(), comment)
                } else {
                    format!("{}", "_".bold().yellow())
                }
            },
            Orthogram::Choice { right_answer, wrong_answers } => {
                let mut answers = Vec::new();
                answers.push(aid::spell_text(right_answer));
                wrong_answers.iter().for_each(|item| answers.push(aid::spell_text(item)));
                answers.shuffle(&mut rnd);
                format!("{}", answers.join("/").underline().bold().yellow())
            }
        }
    }

    fn spell_answer(&self) -> String {
        match self {
            Orthogram::Gap { answer, comment: _ } => {
                aid::spell_text(answer)
            },
            Orthogram::Choice { right_answer, wrong_answers: _ } => {
                aid::spell_text(right_answer)
            }
        }
    }
}

pub(super) mod aid {
    use super::*;

    pub fn spell_text(text: &Vec<Token>) -> String {
        let mut spelling = String::new();
        for token in text {
            spelling += token.spelling();
        }
        spelling
    }
}
