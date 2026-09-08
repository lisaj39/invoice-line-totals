//! Core math for turning a raw invoice line (quantity, unit price, discount,
//! tax) into rounded dollar amounts. Kept separate from main.rs so the
//! rounding rules can be table-tested without touching stdin/stdout.

pub struct LineItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub discount_percent: f64,
    pub tax_rate: f64,
}

pub struct LineResult {
    pub extended: f64,
    pub discount_amount: f64,
    pub tax_amount: f64,
    pub total: f64,
}

/// Round a dollar amount to the nearest cent, half away from zero. Every
/// intermediate money value (extended price, discount, tax) is rounded
/// through this before the next step uses it, so the pieces displayed on
/// an invoice always add up to the total shown - nothing gets re-derived
/// from unrounded numbers later.
pub fn round_to_cents(dollars: f64) -> f64 {
    (dollars * 100.0).round() / 100.0
}

pub fn compute_line(item: &LineItem) -> LineResult {
    let extended = round_to_cents(item.quantity * item.unit_price);
    let discount_amount = round_to_cents(extended * item.discount_percent / 100.0);
    let after_discount = extended - discount_amount;
    let tax_amount = round_to_cents(after_discount * item.tax_rate / 100.0);
    let total = after_discount + tax_amount;

    LineResult {
        extended,
        discount_amount,
        tax_amount,
        total,
    }
}

/// Parse one CSV row: description,quantity,unit_price,discount_percent,tax_rate
///
/// No header row, no quoted fields yet - a description containing a comma
/// will misparse. Fine for a first pass; real CSV quoting is on the list.
pub fn parse_line(raw: &str) -> Result<LineItem, String> {
    let fields: Vec<&str> = raw.split(',').map(|f| f.trim()).collect();
    if fields.len() != 5 {
        return Err(format!("expected 5 fields, got {}", fields.len()));
    }

    let description = fields[0].to_string();
    if description.is_empty() {
        return Err("description is empty".to_string());
    }

    let quantity: f64 = fields[1]
        .parse()
        .map_err(|_| format!("invalid quantity: {}", fields[1]))?;
    let unit_price: f64 = fields[2]
        .parse()
        .map_err(|_| format!("invalid unit price: {}", fields[2]))?;
    let discount_percent: f64 = fields[3]
        .parse()
        .map_err(|_| format!("invalid discount percent: {}", fields[3]))?;
    let tax_rate: f64 = fields[4]
        .parse()
        .map_err(|_| format!("invalid tax rate: {}", fields[4]))?;

    if discount_percent < 0.0 || discount_percent > 100.0 {
        return Err(format!(
            "discount percent out of range 0-100: {}",
            discount_percent
        ));
    }
    if tax_rate < 0.0 {
        return Err(format!("tax rate cannot be negative: {}", tax_rate));
    }

    Ok(LineItem {
        description,
        quantity,
        unit_price,
        discount_percent,
        tax_rate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Case {
        name: &'static str,
        quantity: f64,
        unit_price: f64,
        discount_percent: f64,
        tax_rate: f64,
        want_extended: f64,
        want_discount: f64,
        want_tax: f64,
        want_total: f64,
    }

    #[test]
    fn compute_line_cases() {
        let cases = [
            Case {
                name: "plain line, no discount or tax",
                quantity: 1.0,
                unit_price: 10.0,
                discount_percent: 0.0,
                tax_rate: 0.0,
                want_extended: 10.0,
                want_discount: 0.0,
                want_tax: 0.0,
                want_total: 10.0,
            },
            Case {
                name: "quantity multiplies unit price",
                quantity: 3.0,
                unit_price: 19.99,
                discount_percent: 0.0,
                tax_rate: 0.0,
                want_extended: 59.97,
                want_discount: 0.0,
                want_tax: 0.0,
                want_total: 59.97,
            },
            Case {
                name: "fractional quantity (partial unit, e.g. hours or weight)",
                quantity: 0.5,
                unit_price: 100.0,
                discount_percent: 10.0,
                tax_rate: 0.0,
                want_extended: 50.0,
                want_discount: 5.0,
                want_tax: 0.0,
                want_total: 45.0,
            },
            Case {
                name: "discount amount rounds to nearest cent",
                quantity: 1.0,
                unit_price: 10.01,
                discount_percent: 15.0,
                tax_rate: 0.0,
                want_extended: 10.01,
                want_discount: 1.50,
                want_tax: 0.0,
                want_total: 8.51,
            },
            Case {
                name: "fractional tax rate (real-world sales tax)",
                quantity: 3.0,
                unit_price: 33.33,
                discount_percent: 0.0,
                tax_rate: 8.25,
                want_extended: 99.99,
                want_discount: 0.0,
                want_tax: 8.25,
                want_total: 108.24,
            },
            Case {
                name: "returned goods, negative quantity",
                quantity: -2.0,
                unit_price: 25.0,
                discount_percent: 0.0,
                tax_rate: 0.0,
                want_extended: -50.0,
                want_discount: 0.0,
                want_tax: 0.0,
                want_total: -50.0,
            },
            Case {
                name: "credit line, negative unit price",
                quantity: 1.0,
                unit_price: -15.0,
                discount_percent: 0.0,
                tax_rate: 0.0,
                want_extended: -15.0,
                want_discount: 0.0,
                want_tax: 0.0,
                want_total: -15.0,
            },
            Case {
                name: "zero quantity produces a zero total, not an error",
                quantity: 0.0,
                unit_price: 40.0,
                discount_percent: 5.0,
                tax_rate: 8.0,
                want_extended: 0.0,
                want_discount: 0.0,
                want_tax: 0.0,
                want_total: 0.0,
            },
            Case {
                name: "full discount wipes out the tax base too",
                quantity: 1.0,
                unit_price: 20.0,
                discount_percent: 100.0,
                tax_rate: 20.0,
                want_extended: 20.0,
                want_discount: 20.0,
                want_tax: 0.0,
                want_total: 0.0,
            },
        ];

        for c in cases {
            let item = LineItem {
                description: "x".into(),
                quantity: c.quantity,
                unit_price: c.unit_price,
                discount_percent: c.discount_percent,
                tax_rate: c.tax_rate,
            };
            let got = compute_line(&item);
            assert!(
                (got.extended - c.want_extended).abs() < 0.001,
                "{}: extended got {} want {}",
                c.name,
                got.extended,
                c.want_extended
            );
            assert!(
                (got.discount_amount - c.want_discount).abs() < 0.001,
                "{}: discount got {} want {}",
                c.name,
                got.discount_amount,
                c.want_discount
            );
            assert!(
                (got.tax_amount - c.want_tax).abs() < 0.001,
                "{}: tax got {} want {}",
                c.name,
                got.tax_amount,
                c.want_tax
            );
            assert!(
                (got.total - c.want_total).abs() < 0.001,
                "{}: total got {} want {}",
                c.name,
                got.total,
                c.want_total
            );
        }
    }

    #[test]
    fn parse_line_accepts_valid_rows() {
        let cases = [
            ("Widget, 2, 9.99, 0, 5", "whitespace around fields is trimmed"),
            ("Widget,2,9.99,0,5", "no whitespace"),
            ("Widget,2,9.99,0,0", "zero tax is allowed"),
            ("Widget,2,9.99,100,0", "100 percent discount is allowed"),
        ];
        for (input, name) in cases {
            assert!(parse_line(input).is_ok(), "{}: expected ok, input {:?}", name, input);
        }
    }

    #[test]
    fn parse_line_rejects_bad_rows() {
        let cases = [
            ("Widget,2,9.99,0", "too few fields"),
            ("Widget,2,9.99,0,5,extra", "too many fields"),
            (",2,9.99,0,5", "empty description"),
            ("Widget,abc,9.99,0,5", "non-numeric quantity"),
            ("Widget,2,abc,0,5", "non-numeric unit price"),
            ("Widget,2,9.99,150,5", "discount over 100 percent"),
            ("Widget,2,9.99,-5,5", "negative discount"),
            ("Widget,2,9.99,0,-5", "negative tax rate"),
        ];
        for (input, name) in cases {
            assert!(parse_line(input).is_err(), "{}: expected error, input {:?}", name, input);
        }
    }
}
