use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

use linetotal::{compute_line, parse_line, LineItem, LineResult};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut json_output = false;
    let mut file_path: Option<&str> = None;
    for arg in &args {
        if arg == "--json" {
            json_output = true;
        } else if file_path.is_none() {
            file_path = Some(arg);
        } else {
            eprintln!("error: unexpected argument: {}", arg);
            process::exit(1);
        }
    }

    let input = if let Some(path) = file_path {
        match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: could not read {}: {}", path, e);
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

    let mut lines: Vec<(LineItem, LineResult)> = Vec::new();
    let mut grand_total = 0.0_f64;
    let mut had_error = false;

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
                lines.push((item, result));
            }
            Err(msg) => {
                eprintln!("error: line {}: {}", line_no, msg);
                had_error = true;
            }
        }
    }

    if json_output {
        print_json(&lines, grand_total);
    } else {
        print_table(&lines, grand_total);
    }

    if had_error {
        process::exit(1);
    }
}

fn print_table(lines: &[(LineItem, LineResult)], grand_total: f64) {
    println!(
        "{:<24} {:>10} {:>10} {:>10} {:>10}",
        "description", "extended", "discount", "tax", "total"
    );

    for (item, result) in lines {
        println!(
            "{:<24} {:>10.2} {:>10.2} {:>10.2} {:>10.2}",
            truncate(&item.description, 24),
            result.extended,
            result.discount_amount,
            result.tax_amount,
            result.total
        );
    }

    println!("{:-<58}", "");
    println!("{:<46} {:>10.2}", "grand total", grand_total);
}

// Hand-rolled instead of pulling in serde: the shape is fixed and small
// enough that a dependency isn't worth it.
fn print_json(lines: &[(LineItem, LineResult)], grand_total: f64) {
    let mut out = String::from("{\"lines\":[");
    for (i, (item, result)) in lines.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"description\":{},\"extended\":{:.2},\"discount\":{:.2},\"tax\":{:.2},\"total\":{:.2}}}",
            json_string(&item.description),
            result.extended,
            result.discount_amount,
            result.tax_amount,
            result.total
        ));
    }
    out.push_str(&format!("],\"grand_total\":{:.2}}}", grand_total));
    println!("{}", out);
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        s.chars().take(max).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_string_escapes_special_characters() {
        assert_eq!(json_string("Widget"), "\"Widget\"");
        assert_eq!(json_string(r#"Widget "XL""#), r#""Widget \"XL\"""#);
        assert_eq!(json_string("a\\b"), r#""a\\b""#);
        assert_eq!(json_string("a\tb\nc"), r#""a\tb\nc""#);
    }
}
