use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

use linetotal::{compute_line, parse_line};

fn main() {
    let args: Vec<String> = env::args().collect();

    let input = if args.len() > 1 {
        match fs::read_to_string(&args[1]) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: could not read {}: {}", args[1], e);
                process::exit(1);
            }
        }
    } else {
        let mut buf = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut buf) {
            eprintln!("error: could not read stdin: {}", e);
            process::exit(1);
        }
        buf
    };

    let mut grand_total = 0.0_f64;
    let mut had_error = false;

    println!(
        "{:<24} {:>10} {:>10} {:>10} {:>10}",
        "description", "extended", "discount", "tax", "total"
    );

    for (i, raw) in input.lines().enumerate() {
        let line_no = i + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }

        match parse_line(trimmed) {
            Ok(item) => {
                let result = compute_line(&item);
                grand_total += result.total;
                println!(
                    "{:<24} {:>10.2} {:>10.2} {:>10.2} {:>10.2}",
                    truncate(&item.description, 24),
                    result.extended,
                    result.discount_amount,
                    result.tax_amount,
                    result.total
                );
            }
            Err(msg) => {
                eprintln!("error: line {}: {}", line_no, msg);
                had_error = true;
            }
        }
    }

    println!("{:-<58}", "");
    println!("{:<46} {:>10.2}", "grand total", grand_total);

    if had_error {
        process::exit(1);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect()
    }
}
