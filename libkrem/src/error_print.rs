use colored::*;
use std::collections::VecDeque;

use crate::error::{self, Info};

fn get_count_of_digits(number: i32) -> i32 {
    let mut number_of_digits = 0;
    let mut tmp_num = number;

    while tmp_num > 0 {
        number_of_digits += 1;
        tmp_num /= 10;
    }

    number_of_digits
}

fn generate_spaces(amount: i32) -> String {
    let mut spaces = String::new();

    for _ in 0..amount {
        spaces.push(' ');
    }

    spaces
}

pub fn print_errors<T>(info: &str, path: &str, content: &str, errors: VecDeque<error::Error<T>>) {
    let mut highest_digit_count = 0;

    for error in &errors {
        let count = get_count_of_digits(error.position.line);
        if count > highest_digit_count {
            highest_digit_count = count;
        }
    }

    for error in errors {
        let error_message = error.get_message();
        let suggestion_message = error.get_suggestion();
        let spaces = generate_spaces(highest_digit_count);

        println!(
            "{}{}{}",
            info.red().bold(),
            ": ".bold(),
            error_message.bold()
        );
        println!(
            "{}{} {}:{}:{}",
            spaces,
            "-->".blue().bold(),
            path,
            error.position.line,
            error.position.column
        );

        println!("{} {}", spaces, "|".blue().bold());
        print!(
            "{}{} {}",
            error.position.line.to_string().blue().bold(),
            generate_spaces(highest_digit_count - get_count_of_digits(error.position.line)),
            "|".blue().bold()
        );

        println!(
            "    {}",
            content
                .lines()
                .nth((error.position.line - 1) as usize)
                .unwrap()
                .trim()
        );

        println!(
            "{} {}    {}{} {}",
            spaces,
            "|".blue().bold(),
            generate_spaces(error.position.column as i32),
            "^^".red().bold(),
            error_message.red().bold()
        );

        println!(
            "{} {} {}: {}\n",
            spaces,
            "=".blue().bold(),
            "suggestion".bold(),
            suggestion_message
        );
    }
}