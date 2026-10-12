//! Medicine catalogue import (roadmap 2.2): a clinic uploads its brand list as CSV.

use serde::Serialize;

/// One catalogue row from the CSV.
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogueRow {
    pub brand: String,
    pub generic: String,
    pub strength: Option<String>,
    pub form: Option<String>,
    pub company: Option<String>,
    pub price: Option<f64>,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
pub struct ImportProblem {
    /// 1-based line number in the file.
    pub line: usize,
    pub message: String,
}

pub const MAX_ROWS: usize = 20_000;
const COLUMNS: [&str; 6] = ["brand", "generic", "strength", "form", "company", "price"];

/// Splits CSV text into records: commas, double-quoted fields with "" escapes, CRLF
/// or LF line ends. Returns each record with its starting line number.
fn records(text: &str) -> Vec<(usize, Vec<String>)> {
    let mut out = Vec::new();
    let mut field = String::new();
    let mut record: Vec<String> = Vec::new();
    let mut quoted = false;
    let mut line = 1;
    let mut start = 1;
    let mut chars = text.trim_start_matches('\u{feff}').chars().peekable();
    while let Some(c) = chars.next() {
        match (c, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                chars.next();
                field.push('"');
            }
            ('"', true) => quoted = false,
            ('"', false) if field.is_empty() => quoted = true,
            (',', false) => record.push(std::mem::take(&mut field)),
            ('\r', false) => {}
            ('\n', false) => {
                record.push(std::mem::take(&mut field));
                if record.iter().any(|f| !f.trim().is_empty()) {
                    out.push((start, std::mem::take(&mut record)));
                } else {
                    record.clear();
                }
                line += 1;
                start = line;
            }
            ('\n', true) => {
                line += 1;
                field.push('\n');
            }
            _ => field.push(c),
        }
    }
    record.push(field);
    if record.iter().any(|f| !f.trim().is_empty()) {
        out.push((start, record));
    }
    out
}

/// Parses a catalogue CSV. The first line is a header naming the columns (any order;
/// `brand` and `generic` required; extra columns ignored). Bad rows are reported and
/// skipped, the rest are returned.
pub fn parse(text: &str) -> Result<(Vec<CatalogueRow>, Vec<ImportProblem>), String> {
    let mut recs = records(text).into_iter();
    let (_, header) = recs.next().ok_or("the file is empty")?;
    let header: Vec<String> = header.iter().map(|h| h.trim().to_lowercase()).collect();
    let col = |name: &str| header.iter().position(|h| h == name);
    let (Some(brand_i), Some(generic_i)) = (col("brand"), col("generic")) else {
        return Err(format!(
            "the first line must be a header with at least brand and generic (columns: {})",
            COLUMNS.join(", ")
        ));
    };
    let [strength_i, form_i, company_i, price_i] =
        ["strength", "form", "company", "price"].map(col);

    let mut rows = Vec::new();
    let mut problems = Vec::new();
    for (line, rec) in recs {
        if rows.len() >= MAX_ROWS {
            problems.push(ImportProblem {
                line,
                message: format!("only the first {MAX_ROWS} rows are imported"),
            });
            break;
        }
        let get = |i: Option<usize>, max: usize| -> Option<String> {
            i.and_then(|i| rec.get(i))
                .map(|v| v.trim().chars().take(max).collect::<String>())
                .filter(|v| !v.is_empty())
        };
        let (Some(brand), Some(generic)) = (get(Some(brand_i), 150), get(Some(generic_i), 255))
        else {
            problems.push(ImportProblem {
                line,
                message: "brand and generic are required".into(),
            });
            continue;
        };
        let price = match get(price_i, 20) {
            None => None,
            Some(p) => match p.replace(',', "").parse::<f64>() {
                Ok(x) if x.is_finite() && (0.0..1e7).contains(&x) => {
                    Some((x * 100.0).round() / 100.0)
                }
                _ => {
                    problems.push(ImportProblem {
                        line,
                        message: format!("price \"{p}\" is not a number"),
                    });
                    continue;
                }
            },
        };
        rows.push(CatalogueRow {
            brand,
            generic,
            strength: get(strength_i, 100),
            form: get(form_i, 50),
            company: get(company_i, 150),
            price,
        });
    }
    Ok((rows, problems))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_quoted_fields_and_reports_bad_rows() {
        let csv = "\u{feff}Brand,Generic,Strength,Form,Company,Price\r\n\
                   Napa,Paracetamol,500 mg,Tablet,\"Beximco Pharmaceuticals, Ltd.\",1.20\r\n\
                   ,Omeprazole,20 mg,Capsule,,\r\n\
                   \"Ace \"\"Plus\"\"\",Paracetamol + Caffeine,,,, \r\n\
                   X,Y,,,,abc\r\n\r\n";
        let (rows, problems) = parse(csv).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].company.as_deref(),
            Some("Beximco Pharmaceuticals, Ltd.")
        );
        assert_eq!(rows[0].price, Some(1.2));
        assert_eq!(rows[1].brand, "Ace \"Plus\"");
        assert_eq!(rows[1].strength, None);
        assert_eq!(
            problems.iter().map(|p| p.line).collect::<Vec<_>>(),
            vec![3, 5]
        );
    }

    #[test]
    fn needs_a_header() {
        assert!(parse("Napa,Paracetamol\n").is_err());
        let (rows, _) = parse("generic,brand\nParacetamol,Napa\n").unwrap();
        assert_eq!(rows[0].brand, "Napa");
    }
}
