# linetotal

A command-line tool that takes raw invoice line items (quantity, unit
price, discount, tax rate) and computes the rounded dollar amounts:
extended price, discount amount, tax amount, and line total, plus a
grand total across all lines.

The reason this exists as its own tool instead of a spreadsheet formula:
rounding order matters. If you compute a discount on the unrounded
extended price, then tax on the unrounded discounted price, then round
only the final total, you get numbers that don't match what a human
would compute line by line with a calculator - and they don't match
what most accounting software does either. This tool rounds each money
value (extended, discount, tax) to the cent as soon as it's produced,
the same way a paper invoice would.

## Usage

Input is a CSV file with one line item per row, no header, five columns:

```
description,quantity,unit_price,discount_percent,tax_rate
```

Example `items.csv`:

```
Widget A,3,19.99,0,8.25
Consulting hour,0.5,150.00,10,0
Restocking credit,1,-15.00,0,0
Bulk order,3,33.33,0,8.25
```

Run it:

```
cargo run -- items.csv
```

Output:

```
description              extended   discount        tax      total
Widget A                    59.97       0.00       4.95      64.92
Consulting hour             75.00       7.50       0.00      67.50
Restocking credit          -15.00       0.00       0.00     -15.00
Bulk order                  99.99       0.00       8.25     108.24
----------------------------------------------------------------
grand total                                                 225.66
```

You can also pipe input in over stdin:

```
cat items.csv | cargo run
```

Negative quantities (returned goods) and negative unit prices (credit
lines) are both valid and net out of the grand total as expected.

## Rules

- `discount_percent` must be between 0 and 100.
- `tax_rate` must be zero or positive (no upper bound - tax rates vary
  too much by jurisdiction to hardcode a ceiling).
- `quantity` and `unit_price` can be negative, to represent returns and
  credit lines.
- Rounding is half-away-from-zero to the nearest cent, applied to the
  extended price, the discount amount, and the tax amount separately
  and in that order, so the printed columns always sum to the printed
  total.

## Known limitations

- CSV parsing is a plain comma split - a description containing a
  comma will break the row. No quoting support yet.
- No header row is expected or skipped; the first line is always
  treated as data.

## Building

Standard library only, no external crates:

```
cargo build --release
cargo test
```

## License

MIT, see LICENSE.
