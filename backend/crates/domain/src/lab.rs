//! Lab result rules (AI spec §5.3, §8.3): Claude reads the report, this code decides
//! Normal / Borderline / High / Low / Critical, so flags are always reproducible.

use serde::{Deserialize, Serialize};

/// A known test: normalised code, display name, the unit system ranges use, and the
/// names labs print for it (compared after lower-casing and dropping punctuation).
struct TestDef {
    code: &'static str,
    name: &'static str,
    unit: &'static str,
    aliases: &'static [&'static str],
}

const TESTS: &[TestDef] = &[
    TestDef {
        code: "HB",
        name: "Hemoglobin",
        unit: "g/dL",
        aliases: &[
            "hb",
            "hgb",
            "haemoglobin",
            "hemoglobin",
            "haemoglobinhb",
            "hemoglobinhb",
        ],
    },
    TestDef {
        code: "WBC",
        name: "WBC (total count)",
        unit: "/cumm",
        aliases: &[
            "wbc",
            "tc",
            "totalcount",
            "wbccount",
            "totalwbccount",
            "totalleucocytecount",
            "totalleukocytecount",
            "tlc",
            "totalcountwbc",
        ],
    },
    TestDef {
        code: "PLT",
        name: "Platelet count",
        unit: "/cumm",
        aliases: &[
            "platelet",
            "platelets",
            "plateletcount",
            "plt",
            "totalplateletcount",
        ],
    },
    TestDef {
        code: "ESR",
        name: "ESR",
        unit: "mm/1st hr",
        aliases: &["esr", "esrwestergren", "erythrocytesedimentationrate"],
    },
    TestDef {
        code: "NEUT",
        name: "Neutrophils",
        unit: "%",
        aliases: &[
            "neutrophil",
            "neutrophils",
            "polymorph",
            "polymorphs",
            "neutrophilspolymorphs",
        ],
    },
    TestDef {
        code: "LYMPH",
        name: "Lymphocytes",
        unit: "%",
        aliases: &["lymphocyte", "lymphocytes"],
    },
    TestDef {
        code: "MONO",
        name: "Monocytes",
        unit: "%",
        aliases: &["monocyte", "monocytes"],
    },
    TestDef {
        code: "EOS",
        name: "Eosinophils",
        unit: "%",
        aliases: &["eosinophil", "eosinophils"],
    },
    TestDef {
        code: "PCV",
        name: "PCV / Haematocrit",
        unit: "%",
        aliases: &["pcv", "hct", "haematocrit", "hematocrit"],
    },
    TestDef {
        code: "MCV",
        name: "MCV",
        unit: "fL",
        aliases: &["mcv"],
    },
    TestDef {
        code: "MCH",
        name: "MCH",
        unit: "pg",
        aliases: &["mch"],
    },
    TestDef {
        code: "MCHC",
        name: "MCHC",
        unit: "g/dL",
        aliases: &["mchc"],
    },
    TestDef {
        code: "RBS",
        name: "Random blood sugar",
        unit: "mmol/L",
        aliases: &[
            "rbs",
            "randombloodsugar",
            "randombloodglucose",
            "randomplasmaglucose",
            "bloodsugarrandom",
            "bloodglucoserandom",
        ],
    },
    TestDef {
        code: "FBS",
        name: "Fasting blood sugar",
        unit: "mmol/L",
        aliases: &[
            "fbs",
            "fastingbloodsugar",
            "fastingbloodglucose",
            "fastingplasmaglucose",
            "bloodsugarfasting",
            "bloodglucosefasting",
        ],
    },
    TestDef {
        code: "PPBS",
        name: "Blood sugar 2 hrs after breakfast",
        unit: "mmol/L",
        aliases: &[
            "2habf",
            "abf",
            "2hrsabf",
            "2hoursafterbreakfast",
            "bloodsugar2hrsafterbreakfast",
            "ppbs",
            "postprandialbloodsugar",
            "2hpp",
        ],
    },
    TestDef {
        code: "HBA1C",
        name: "HbA1c",
        unit: "%",
        aliases: &[
            "hba1c",
            "glycatedhaemoglobin",
            "glycatedhemoglobin",
            "glycosylatedhaemoglobin",
            "glycosylatedhemoglobin",
            "a1c",
        ],
    },
    TestDef {
        code: "CREAT",
        name: "Creatinine",
        unit: "mg/dL",
        aliases: &["creatinine", "screatinine", "serumcreatinine", "creat"],
    },
    TestDef {
        code: "UREA",
        name: "Blood urea",
        unit: "mg/dL",
        aliases: &["urea", "bloodurea", "serumurea", "surea"],
    },
    TestDef {
        code: "NA",
        name: "Sodium",
        unit: "mmol/L",
        aliases: &["sodium", "na", "ssodium", "serumsodium"],
    },
    TestDef {
        code: "K",
        name: "Potassium",
        unit: "mmol/L",
        aliases: &["potassium", "k", "spotassium", "serumpotassium"],
    },
    TestDef {
        code: "CL",
        name: "Chloride",
        unit: "mmol/L",
        aliases: &["chloride", "cl", "schloride", "serumchloride"],
    },
    TestDef {
        code: "CA",
        name: "Calcium",
        unit: "mg/dL",
        aliases: &["calcium", "scalcium", "serumcalcium", "totalcalcium"],
    },
    TestDef {
        code: "URIC",
        name: "Uric acid",
        unit: "mg/dL",
        aliases: &["uricacid", "suricacid", "serumuricacid"],
    },
    TestDef {
        code: "ALT",
        name: "ALT (SGPT)",
        unit: "U/L",
        aliases: &[
            "sgpt",
            "alt",
            "altsgpt",
            "sgptalt",
            "alaninetransaminase",
            "alanineaminotransferase",
        ],
    },
    TestDef {
        code: "AST",
        name: "AST (SGOT)",
        unit: "U/L",
        aliases: &[
            "sgot",
            "ast",
            "astsgot",
            "sgotast",
            "aspartatetransaminase",
            "aspartateaminotransferase",
        ],
    },
    TestDef {
        code: "ALP",
        name: "Alkaline phosphatase",
        unit: "U/L",
        aliases: &["alp", "alkalinephosphatase", "salkalinephosphatase"],
    },
    TestDef {
        code: "BILT",
        name: "Bilirubin (total)",
        unit: "mg/dL",
        aliases: &[
            "bilirubin",
            "totalbilirubin",
            "bilirubintotal",
            "sbilirubin",
            "serumbilirubin",
        ],
    },
    TestDef {
        code: "TSH",
        name: "TSH",
        unit: "mIU/L",
        aliases: &["tsh", "thyroidstimulatinghormone", "stsh"],
    },
    TestDef {
        code: "FT4",
        name: "Free T4",
        unit: "ng/dL",
        aliases: &["ft4", "freet4", "freethyroxine"],
    },
    TestDef {
        code: "CHOL",
        name: "Total cholesterol",
        unit: "mg/dL",
        aliases: &[
            "cholesterol",
            "totalcholesterol",
            "scholesterol",
            "cholesteroltotal",
        ],
    },
    TestDef {
        code: "TG",
        name: "Triglycerides",
        unit: "mg/dL",
        aliases: &["tg", "triglyceride", "triglycerides", "striglyceride"],
    },
    TestDef {
        code: "HDL",
        name: "HDL cholesterol",
        unit: "mg/dL",
        aliases: &["hdl", "hdlc", "hdlcholesterol"],
    },
    TestDef {
        code: "LDL",
        name: "LDL cholesterol",
        unit: "mg/dL",
        aliases: &["ldl", "ldlc", "ldlcholesterol"],
    },
    TestDef {
        code: "CRP",
        name: "CRP",
        unit: "mg/L",
        aliases: &["crp", "creactiveprotein"],
    },
    TestDef {
        code: "PUS",
        name: "Pus cells (urine)",
        unit: "/HPF",
        aliases: &["puscell", "puscells", "pus"],
    },
    TestDef {
        code: "RBC_URINE",
        name: "RBC (urine)",
        unit: "/HPF",
        aliases: &["rbcurine", "urinerbc"],
    },
    TestDef {
        code: "EPI",
        name: "Epithelial cells",
        unit: "/HPF",
        aliases: &["epithelialcell", "epithelialcells", "epicells"],
    },
    TestDef {
        code: "NITRITE",
        name: "Nitrite",
        unit: "",
        aliases: &["nitrite", "nitrites"],
    },
    TestDef {
        code: "ALB_URINE",
        name: "Albumin (urine)",
        unit: "",
        aliases: &["albuminurine", "urinealbumin", "protein", "urineprotein"],
    },
    TestDef {
        code: "SUGAR_URINE",
        name: "Sugar (urine)",
        unit: "",
        aliases: &["sugar", "urinesugar", "glucoseurine", "urineglucose"],
    },
    TestDef {
        code: "KETONE",
        name: "Ketone bodies",
        unit: "",
        aliases: &["ketone", "ketones", "ketonebodies", "acetone"],
    },
];

fn key(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Normalised code and display name for a printed test name (FR-11). Microscopy
/// "RBC" counted per HPF is the urine test, not the blood count.
pub fn normalise(test: &str, unit: Option<&str>) -> (String, String) {
    let k = key(test);
    let per_hpf = unit.is_some_and(|u| key(u).contains("hpf"));
    if per_hpf && k.starts_with("rbc") {
        return ("RBC_URINE".into(), "RBC (urine)".into());
    }
    if let Some(t) = TESTS.iter().find(|t| t.aliases.contains(&k.as_str())) {
        return (t.code.into(), t.name.into());
    }
    let fallback: String = k.to_uppercase().chars().take(50).collect();
    let code = if fallback.is_empty() {
        "UNKNOWN".into()
    } else {
        fallback
    };
    (code, test.trim().chars().take(100).collect())
}

/// The unit a code's system ranges are written in.
pub fn system_unit(code: &str) -> Option<&'static str> {
    TESTS.iter().find(|t| t.code == code).map(|t| t.unit)
}

/// A printed result: one number, a span ("20-25"), or words ("Positive").
#[derive(Clone, Debug, PartialEq)]
pub enum Reading {
    Num(f64),
    Span(f64, f64),
    Text(String),
}

fn number(s: &str) -> Option<f64> {
    // Indian / Bangladeshi grouping ("2,40,000") and plain thousands both just drop commas.
    let t: String = s
        .trim()
        .chars()
        .filter(|c| *c != ',' && *c != ' ')
        .collect();
    let t = t.trim_start_matches(['<', '>', '≤', '≥', '=']);
    t.parse::<f64>().ok().filter(|x| x.is_finite())
}

fn split_span(s: &str) -> Option<(&str, &str)> {
    for sep in [" - ", "–", "—", "-", " to "] {
        if let Some((a, b)) = s.split_once(sep)
            && !a.trim().is_empty()
        {
            return Some((a, b));
        }
    }
    None
}

pub fn read_value(value: &str) -> Reading {
    let v = value.trim();
    if let Some(x) = number(v) {
        return Reading::Num(x);
    }
    if let Some((a, b)) = split_span(v)
        && let (Some(lo), Some(hi)) = (number(a), number(b))
    {
        return Reading::Span(lo.min(hi), lo.max(hi));
    }
    Reading::Text(v.to_string())
}

impl Reading {
    /// The number stored for charts: the value itself, or the upper end of a span.
    pub fn numeric(&self) -> Option<f64> {
        match self {
            Self::Num(x) => Some(*x),
            Self::Span(_, hi) => Some(*hi),
            Self::Text(_) => None,
        }
    }
}

/// Bounds parsed from a printed reference range.
#[derive(Clone, Debug, PartialEq)]
pub enum Bounds {
    Numeric {
        low: Option<f64>,
        high: Option<f64>,
    },
    /// "Negative", "Nil", "Absent": any positive result is abnormal.
    Negative,
}

pub fn read_range(range: &str) -> Option<Bounds> {
    let r = range.trim();
    let k = key(r);
    if ["negative", "nil", "absent", "notfound", "notseen", "neg"].contains(&k.as_str()) {
        return Some(Bounds::Negative);
    }
    let lower = r.to_lowercase();
    if let Some(rest) = lower
        .strip_prefix("up to")
        .or_else(|| lower.strip_prefix("upto"))
        .or_else(|| lower.strip_prefix('<'))
        .or_else(|| lower.strip_prefix('≤'))
    {
        return number(rest).map(|h| Bounds::Numeric {
            low: None,
            high: Some(h),
        });
    }
    if let Some(rest) = lower.strip_prefix('>').or_else(|| lower.strip_prefix('≥')) {
        return number(rest).map(|l| Bounds::Numeric {
            low: Some(l),
            high: None,
        });
    }
    let (a, b) = split_span(r)?;
    match (number(a), number(b)) {
        (Some(lo), Some(hi)) if lo <= hi => Some(Bounds::Numeric {
            low: Some(lo),
            high: Some(hi),
        }),
        _ => None,
    }
}

/// A system reference range (from `reference_ranges`), in the test's system unit.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SystemRange {
    pub low: Option<f64>,
    pub high: Option<f64>,
    pub critical_low: Option<f64>,
    pub critical_high: Option<f64>,
}

/// Converts a value printed in `unit` to the test's system unit (FR-12). `None`
/// when the unit is not one we know how to convert.
pub fn to_system_unit(code: &str, value: f64, unit: Option<&str>) -> Option<f64> {
    let target = system_unit(code)?;
    let u = unit.map(key).unwrap_or_default();
    if u.is_empty() || u == key(target) {
        return Some(value);
    }
    let factor = match code {
        "RBS" | "FBS" | "PPBS" if u == "mgdl" => 1.0 / 18.0,
        "CREAT" if u == "umoll" || u == "µmoll" || u == "μmoll" => 1.0 / 88.4,
        "HB" | "MCHC" if u == "gl" => 0.1,
        "WBC" | "PLT"
            if u.contains("109")
                || u.contains("103")
                || u.starts_with("k")
                || u.contains("thousand") =>
        {
            1000.0
        }
        "WBC" | "PLT" if u == "ul" || u == "µl" || u == "μl" || u == "mm3" || u == "cmm" => 1.0,
        "UREA" if u == "mmoll" => 6.006,
        "CHOL" | "HDL" | "LDL" if u == "mmoll" => 38.67,
        "TG" if u == "mmoll" => 88.57,
        "CA" if u == "mmoll" => 4.008,
        _ => return None,
    };
    Some(value * factor)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Normal,
    Borderline,
    High,
    Low,
    Critical,
    /// A qualitative result that should be negative but is not.
    Abnormal,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Borderline => "borderline",
            Self::High => "high",
            Self::Low => "low",
            Self::Critical => "critical",
            Self::Abnormal => "abnormal",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [
            Self::Normal,
            Self::Borderline,
            Self::High,
            Self::Low,
            Self::Critical,
            Self::Abnormal,
        ]
        .into_iter()
        .find(|x| x.as_str() == s)
    }

    /// Short marker used in the ℞ short form (FR-35).
    pub fn marker(self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::Borderline => "●",
            Self::High => "▲",
            Self::Low => "▼",
            Self::Critical => "⚠",
            Self::Abnormal => "+",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RangeSource {
    Lab,
    System,
}

impl RangeSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lab => "lab",
            Self::System => "system",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assessment {
    pub status: Option<Status>,
    pub source: Option<RangeSource>,
}

/// Out of range by no more than this share of the limit counts as borderline.
const BORDERLINE: f64 = 0.05;

fn positive(text: &str) -> Option<bool> {
    let k = key(text);
    if k.is_empty() {
        return None;
    }
    if [
        "negative",
        "nil",
        "absent",
        "notfound",
        "notseen",
        "neg",
        "normal",
        "nonreactive",
    ]
    .contains(&k.as_str())
    {
        return Some(false);
    }
    if text.contains('+')
        || ["positive", "present", "reactive", "seen", "found", "pos"].contains(&k.as_str())
    {
        return Some(true);
    }
    None
}

fn grade(lo: f64, hi: f64, low: Option<f64>, high: Option<f64>) -> Status {
    if let Some(h) = high
        && hi > h
    {
        return if hi <= h + h.abs() * BORDERLINE {
            Status::Borderline
        } else {
            Status::High
        };
    }
    if let Some(l) = low
        && lo < l
    {
        return if lo >= l - l.abs() * BORDERLINE {
            Status::Borderline
        } else {
            Status::Low
        };
    }
    Status::Normal
}

/// Grades one result (FR-15–FR-17): the lab's printed range first, else the system
/// range for the patient's age and sex; system critical limits always apply.
pub fn assess(
    code: &str,
    value: &str,
    unit: Option<&str>,
    lab_range: Option<&str>,
    lab_flag: Option<&str>,
    system: Option<&SystemRange>,
) -> Assessment {
    let reading = read_value(value);
    let lab = lab_range.and_then(read_range);

    if let Reading::Text(text) = &reading {
        let qualitative = matches!(lab, Some(Bounds::Negative))
            || matches!(code, "NITRITE" | "ALB_URINE" | "SUGAR_URINE" | "KETONE");
        if !qualitative {
            return Assessment {
                status: None,
                source: None,
            };
        }
        let source = Some(if lab.is_some() {
            RangeSource::Lab
        } else {
            RangeSource::System
        });
        let status = if key(text) == "trace" {
            Some(Status::Borderline)
        } else {
            positive(text).map(|p| if p { Status::Abnormal } else { Status::Normal })
        };
        return Assessment { status, source };
    }

    let (lo, hi) = match reading {
        Reading::Num(x) => (x, x),
        Reading::Span(a, b) => (a, b),
        Reading::Text(_) => unreachable!("handled above"),
    };
    let sys_lo = to_system_unit(code, lo, unit);
    let sys_hi = to_system_unit(code, hi, unit);

    let mut status = None;
    let mut source = None;
    if let Some(Bounds::Numeric { low, high }) = lab {
        status = Some(grade(lo, hi, low, high));
        source = Some(RangeSource::Lab);
    } else if let (Some(s), Some(a), Some(b)) = (system, sys_lo, sys_hi)
        && (s.low.is_some() || s.high.is_some())
    {
        status = Some(grade(a, b, s.low, s.high));
        source = Some(RangeSource::System);
    } else if let Some(flag) = lab_flag.map(|f| f.trim().to_uppercase()) {
        status = match flag.as_str() {
            "H" | "HIGH" => Some(Status::High),
            "L" | "LOW" => Some(Status::Low),
            _ => None,
        };
        source = status.map(|_| RangeSource::Lab);
    }

    if let (Some(s), Some(a), Some(b)) = (system, sys_lo, sys_hi) {
        let crit = s.critical_high.is_some_and(|c| b > c) || s.critical_low.is_some_and(|c| a < c);
        if crit {
            status = Some(Status::Critical);
            source = source.or(Some(RangeSource::System));
        }
    }
    Assessment { status, source }
}

/// One line for the Investigations section (FR-35), e.g.
/// "CBC: Hb 10.2 g/dL ▼, WBC 13,800 /cumm ▲". Only results that are not normal are
/// listed; `None` if there is nothing to say.
pub fn short_line(
    report_type: &str,
    results: &[(String, String, Option<String>, Option<Status>)],
) -> Option<String> {
    let parts: Vec<String> = results
        .iter()
        .filter(|(_, _, _, s)| s.is_some_and(|s| s != Status::Normal))
        .map(|(name, value, unit, s)| {
            let unit = unit.as_deref().filter(|u| !u.is_empty());
            let marker = s.map(Status::marker).unwrap_or_default();
            [
                Some(name.as_str()),
                Some(value.as_str()),
                unit,
                Some(marker),
            ]
            .into_iter()
            .flatten()
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
        })
        .collect();
    let label = if report_type.trim().is_empty() {
        "Report"
    } else {
        report_type.trim()
    };
    if parts.is_empty() {
        return None;
    }
    Some(format!("{label}: {}", parts.join(", ")))
}

const TITLES: &[&str] = &[
    "mr", "mrs", "ms", "miss", "md", "mst", "mohammad", "mohammed", "muhammad", "baby", "master",
    "dr", "begum", "khatun",
];

/// Whether the name printed on a report could be this patient's (§14 "name
/// mismatch" warning). Lenient: one shared name part is enough.
pub fn name_matches(on_report: &str, names: &[&str]) -> bool {
    let parts = |s: &str| -> Vec<String> {
        s.split(|c: char| !c.is_alphanumeric())
            .map(|w| w.to_lowercase())
            .filter(|w| w.chars().count() >= 3 && !TITLES.contains(&w.as_str()))
            .collect()
    };
    let printed = parts(on_report);
    if printed.is_empty() {
        return true;
    }
    names
        .iter()
        .flat_map(|n| parts(n))
        .any(|w| printed.contains(&w))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hb_f() -> SystemRange {
        SystemRange {
            low: Some(12.0),
            high: Some(15.5),
            critical_low: Some(7.0),
            critical_high: Some(20.0),
        }
    }

    #[test]
    fn normalises_names() {
        assert_eq!(normalise("Haemoglobin (Hb)", Some("g/dL")).0, "HB");
        assert_eq!(normalise("S. Creatinine", None).0, "CREAT");
        assert_eq!(normalise("Glycated Haemoglobin", None).0, "HBA1C");
        assert_eq!(normalise("Pus Cells", Some("/HPF")).0, "PUS");
        assert_eq!(normalise("RBC", Some("/HPF")).0, "RBC_URINE");
        assert_eq!(normalise("Total Count (WBC)", None).0, "WBC");
        assert_eq!(normalise("Vitamin D", None).0, "VITAMIND");
    }

    #[test]
    fn reads_values_and_ranges() {
        assert_eq!(read_value("2,40,000"), Reading::Num(240000.0));
        assert_eq!(read_value("20-25"), Reading::Span(20.0, 25.0));
        assert_eq!(read_value("Positive"), Reading::Text("Positive".into()));
        assert_eq!(
            read_range("4,000 - 11,000"),
            Some(Bounds::Numeric {
                low: Some(4000.0),
                high: Some(11000.0)
            })
        );
        assert_eq!(
            read_range("0–5"),
            Some(Bounds::Numeric {
                low: Some(0.0),
                high: Some(5.0)
            })
        );
        assert_eq!(
            read_range("< 200"),
            Some(Bounds::Numeric {
                low: None,
                high: Some(200.0)
            })
        );
        assert_eq!(
            read_range("Up to 20"),
            Some(Bounds::Numeric {
                low: None,
                high: Some(20.0)
            })
        );
        assert_eq!(read_range("Nil"), Some(Bounds::Negative));
        assert_eq!(read_range("see note"), None);
    }

    #[test]
    fn grades_against_lab_range_first() {
        let a = assess(
            "WBC",
            "13,800",
            Some("/cumm"),
            Some("4,000-11,000"),
            None,
            None,
        );
        assert_eq!(a.status, Some(Status::High));
        assert_eq!(a.source, Some(RangeSource::Lab));
        let a = assess(
            "HB",
            "11.8",
            Some("g/dL"),
            Some("12-15.5"),
            None,
            Some(&hb_f()),
        );
        assert_eq!(a.status, Some(Status::Borderline));
        let a = assess("HB", "10.2", Some("g/dL"), None, None, Some(&hb_f()));
        assert_eq!(a.status, Some(Status::Low));
        assert_eq!(a.source, Some(RangeSource::System));
        let a = assess("PUS", "20-25", Some("/HPF"), Some("0-5"), None, None);
        assert_eq!(a.status, Some(Status::High));
    }

    #[test]
    fn critical_limits_always_apply() {
        // The lab range says only "low", but Hb 6.1 is below the critical limit.
        let a = assess(
            "HB",
            "6.1",
            Some("g/dL"),
            Some("12-15.5"),
            None,
            Some(&hb_f()),
        );
        assert_eq!(a.status, Some(Status::Critical));
        let k = SystemRange {
            low: Some(3.5),
            high: Some(5.1),
            critical_low: Some(2.5),
            critical_high: Some(6.0),
        };
        assert_eq!(
            assess("K", "6.4", Some("mmol/L"), None, None, Some(&k)).status,
            Some(Status::Critical)
        );
        // Units are converted before comparing with system limits: 61 g/L = 6.1 g/dL.
        assert_eq!(
            assess("HB", "61", Some("g/L"), None, None, Some(&hb_f())).status,
            Some(Status::Critical)
        );
    }

    #[test]
    fn converts_units() {
        assert_eq!(to_system_unit("RBS", 180.0, Some("mg/dL")), Some(10.0));
        assert_eq!(
            to_system_unit("PLT", 250.0, Some("x10^9/L")),
            Some(250000.0)
        );
        assert_eq!(to_system_unit("HB", 12.0, Some("g/dL")), Some(12.0));
        assert_eq!(to_system_unit("HB", 12.0, Some("furlongs")), None);
    }

    #[test]
    fn qualitative_results() {
        assert_eq!(
            assess("NITRITE", "Positive", None, Some("Negative"), None, None).status,
            Some(Status::Abnormal)
        );
        assert_eq!(
            assess("ALB_URINE", "Trace", None, None, None, None).status,
            Some(Status::Borderline)
        );
        assert_eq!(
            assess("SUGAR_URINE", "Nil", None, None, None, None).status,
            Some(Status::Normal)
        );
        assert_eq!(
            assess("COLOUR", "Straw", None, None, None, None).status,
            None
        );
    }

    #[test]
    fn falls_back_to_the_lab_flag() {
        let a = assess("VITAMIND", "12", Some("ng/mL"), None, Some("L"), None);
        assert_eq!(a.status, Some(Status::Low));
    }

    #[test]
    fn short_form_line() {
        let rows = vec![
            (
                "Hb".to_string(),
                "10.2".to_string(),
                Some("g/dL".to_string()),
                Some(Status::Low),
            ),
            (
                "WBC".to_string(),
                "13,800".to_string(),
                Some("/cumm".to_string()),
                Some(Status::High),
            ),
            (
                "Platelet".to_string(),
                "2,40,000".to_string(),
                None,
                Some(Status::Normal),
            ),
        ];
        assert_eq!(
            short_line("CBC", &rows).as_deref(),
            Some("CBC: Hb 10.2 g/dL ▼, WBC 13,800 /cumm ▲")
        );
        assert_eq!(short_line("CBC", &rows[2..]), None);
    }

    #[test]
    fn name_check_is_lenient() {
        assert!(name_matches("Mst. Maisha Akter", &["Maisha Rahman"]));
        assert!(!name_matches("Md. Karim Hossain", &["Maisha Rahman"]));
        assert!(name_matches("", &["Maisha Rahman"]));
    }
}
