//! Investigation reports and AI output types (AI spec §5, §8.7–8.8).

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use vd_common::AppError;

use crate::lab::{RangeSource, Status};
use crate::text::{check_len, opt};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    Uploaded,
    Extracting,
    Extracted,
    Verified,
    Failed,
}

impl ReportStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Uploaded => "uploaded",
            Self::Extracting => "extracting",
            Self::Extracted => "extracted",
            Self::Verified => "verified",
            Self::Failed => "failed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [
            Self::Uploaded,
            Self::Extracting,
            Self::Extracted,
            Self::Verified,
            Self::Failed,
        ]
        .into_iter()
        .find(|x| x.as_str() == s)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub id: u64,
    pub patient_id: u64,
    pub visit_id: Option<u64>,
    pub report_type: Option<String>,
    pub lab_name: Option<String>,
    pub report_date: Option<NaiveDate>,
    pub mime_type: String,
    pub original_name: Option<String>,
    pub status: ReportStatus,
    pub impression: Option<String>,
    pub name_mismatch: bool,
    /// Parts of the report Claude could not read.
    pub unreadable_parts: Vec<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub results: Vec<ReportResult>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReportResult {
    pub id: u64,
    pub test_code: String,
    pub test_name: String,
    pub value_text: String,
    pub value_num: Option<f64>,
    pub unit: Option<String>,
    pub ref_range_text: Option<String>,
    pub lab_flag: Option<String>,
    pub range_source: Option<RangeSource>,
    pub status: Option<Status>,
    pub confidence: Option<Confidence>,
    pub edited_by_doctor: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "high" => Some(Self::High),
            "medium" => Some(Self::Medium),
            "low" => Some(Self::Low),
            _ => None,
        }
    }
}

/// What Claude's extraction tool returns (AI spec §8.7). Lenient: anything missing
/// takes a default, and the backend checks it before storing.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Extraction {
    pub report_type: Option<String>,
    pub lab_name: Option<String>,
    pub report_date: Option<String>,
    pub patient_name_on_report: Option<String>,
    pub results: Vec<ExtractedResult>,
    pub impression: Option<String>,
    pub unreadable_parts: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ExtractedResult {
    pub test: String,
    pub value: String,
    pub unit: Option<String>,
    pub range: Option<String>,
    pub lab_flag: Option<String>,
    pub confidence: Option<String>,
}

/// One result row as entered or corrected by the doctor (FR-14).
#[derive(Clone, Debug, Deserialize)]
pub struct ResultInput {
    pub test_name: String,
    pub value_text: String,
    pub unit: Option<String>,
    pub ref_range_text: Option<String>,
    pub lab_flag: Option<String>,
    pub confidence: Option<Confidence>,
    #[serde(default)]
    pub edited: bool,
}

/// A checked result row ready to grade and store.
#[derive(Clone, Debug)]
pub struct CleanResult {
    pub test_name: String,
    pub value_text: String,
    pub unit: Option<String>,
    pub ref_range_text: Option<String>,
    pub lab_flag: Option<String>,
    pub confidence: Option<Confidence>,
    pub edited: bool,
}

pub const MAX_RESULTS: usize = 200;

impl ResultInput {
    pub fn clean(self) -> Result<Option<CleanResult>, AppError> {
        let test_name = self.test_name.trim().to_string();
        let value_text = self.value_text.trim().to_string();
        if test_name.is_empty() && value_text.is_empty() {
            return Ok(None);
        }
        if test_name.is_empty() || value_text.is_empty() {
            return Err(AppError::Validation(
                "each result needs a test name and a value".into(),
            ));
        }
        check_len(&test_name, "test name", 100)?;
        check_len(&value_text, "value", 50)?;
        Ok(Some(CleanResult {
            test_name,
            value_text,
            unit: opt(self.unit, "unit", 30)?,
            ref_range_text: opt(self.ref_range_text, "reference range", 50)?,
            lab_flag: opt(self.lab_flag, "lab flag", 10)?,
            confidence: self.confidence,
            edited: self.edited,
        }))
    }
}

impl ExtractedResult {
    /// Claude's row as a result input, trimmed to the column sizes.
    pub fn into_clean(self) -> Option<CleanResult> {
        let cut = |s: String, n: usize| -> String { s.trim().chars().take(n).collect() };
        let some = |s: Option<String>, n: usize| s.map(|x| cut(x, n)).filter(|x| !x.is_empty());
        let test_name = cut(self.test, 100);
        let value_text = cut(self.value, 50);
        if test_name.is_empty() || value_text.is_empty() {
            return None;
        }
        Some(CleanResult {
            test_name,
            value_text,
            unit: some(self.unit, 30),
            ref_range_text: some(self.range, 50),
            lab_flag: some(self.lab_flag, 10),
            confidence: self.confidence.as_deref().and_then(Confidence::parse),
            edited: false,
        })
    }
}

/// Report details the doctor can correct alongside the values.
#[derive(Clone, Debug, Deserialize)]
pub struct ReportPatch {
    pub report_type: Option<String>,
    pub lab_name: Option<String>,
    pub report_date: Option<NaiveDate>,
    pub impression: Option<String>,
    pub results: Vec<ResultInput>,
}

// ---- Clinical analysis (§8.8) ----

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ClinicalAnalysis {
    pub summary: String,
    pub report_summaries: Vec<ReportSummary>,
    pub findings: Vec<Finding>,
    pub possible_issues: Vec<Named>,
    pub suggested_investigations: Vec<Named>,
    pub advice_en: Vec<String>,
    pub advice_bn: Vec<String>,
    pub next_plan: Option<String>,
    pub red_flags: Vec<String>,
    pub conflicts: Vec<String>,
    pub trend_comments: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportSummary {
    pub report: String,
    pub summary: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Finding {
    pub text: String,
    pub source: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Named {
    pub name: String,
    pub reason: String,
    pub likelihood: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section {
    Diagnosis,
    Investigation,
    Advice,
    NextPlan,
    RedFlag,
    Safety,
}

impl Section {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Diagnosis => "diagnosis",
            Self::Investigation => "investigation",
            Self::Advice => "advice",
            Self::NextPlan => "next_plan",
            Self::RedFlag => "red_flag",
            Self::Safety => "safety",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [
            Self::Diagnosis,
            Self::Investigation,
            Self::Advice,
            Self::NextPlan,
            Self::RedFlag,
            Self::Safety,
        ]
        .into_iter()
        .find(|x| x.as_str() == s)
    }
}

/// A suggestion before it is stored.
#[derive(Clone, Debug, PartialEq)]
pub struct NewSuggestion {
    pub section: Section,
    pub text_en: String,
    pub text_bn: Option<String>,
    pub likelihood: Option<String>,
    pub source_refs: Vec<String>,
}

impl ClinicalAnalysis {
    /// The tick-box suggestions the doctor reviews (§6.2), one per item.
    pub fn suggestions(&self) -> Vec<NewSuggestion> {
        let mut out = Vec::new();
        let mut push = |section,
                        text: &str,
                        bn: Option<&String>,
                        likelihood: Option<&String>,
                        refs: Vec<String>| {
            let text = text.trim();
            if !text.is_empty() {
                out.push(NewSuggestion {
                    section,
                    text_en: text.to_string(),
                    text_bn: bn.map(|b| b.trim().to_string()).filter(|b| !b.is_empty()),
                    likelihood: likelihood.cloned(),
                    source_refs: refs,
                });
            }
        };
        for i in &self.possible_issues {
            push(
                Section::Diagnosis,
                &i.name,
                None,
                i.likelihood.as_ref(),
                vec![i.reason.clone()],
            );
        }
        for i in &self.suggested_investigations {
            push(
                Section::Investigation,
                &i.name,
                None,
                None,
                vec![i.reason.clone()],
            );
        }
        for (k, en) in self.advice_en.iter().enumerate() {
            push(Section::Advice, en, self.advice_bn.get(k), None, vec![]);
        }
        // Bangla advice with no English partner still counts.
        for bn in self.advice_bn.iter().skip(self.advice_en.len()) {
            push(Section::Advice, bn, None, None, vec![]);
        }
        if let Some(p) = &self.next_plan {
            push(Section::NextPlan, p, None, None, vec![]);
        }
        for r in &self.red_flags {
            push(Section::RedFlag, r, None, None, vec![]);
        }
        out
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Suggestion {
    pub id: u64,
    pub section: Section,
    pub text_en: String,
    pub text_bn: Option<String>,
    pub likelihood: Option<String>,
    pub source_refs: Vec<String>,
    pub action: String,
    pub edited_text: Option<String>,
    pub reason: Option<String>,
}

/// A saved analysis with its suggestions, as shown in the AI Findings panel.
#[derive(Clone, Debug, Serialize)]
pub struct AnalysisView {
    pub id: u64,
    pub content: serde_json::Value,
    pub claude_model: String,
    pub prompt_version: String,
    pub created_at: DateTime<Utc>,
    pub suggestions: Vec<Suggestion>,
}

// ---- Safety check (§5.5) ----

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SafetyWarning {
    pub medicines: Vec<String>,
    pub kind: String,
    pub severity: String,
    pub message: String,
    pub source: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct SafetyAnswer {
    pub warnings: Vec<SafetyWarning>,
}

// ---- Patient summary (PM spec §7.1) ----

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PatientSummary {
    pub one_line: String,
    pub active_problems: Vec<String>,
    pub recurring_issues: Vec<String>,
    pub lab_trends: Vec<String>,
    pub medicines_history: Vec<String>,
    pub alerts: Vec<String>,
    pub points_to_check: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SavedSummary {
    pub id: u64,
    pub content: PatientSummary,
    pub claude_model: String,
    pub prompt_version: String,
    pub created_at: DateTime<Utc>,
}

// ---- Smart search (PM spec §7.2) ----

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_min: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_max: Option<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnosis_terms: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub complaint_terms: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub medicine_terms: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub chronic_terms: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visited_from: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visited_to: Option<NaiveDate>,
}

impl SearchFilters {
    /// Drops anything unusable from Claude's answer: unknown sex, blank or long
    /// terms, impossible ages, and caps the number of terms.
    pub fn sanitise(mut self) -> Self {
        let clean = |v: Vec<String>| -> Vec<String> {
            let mut out: Vec<String> = Vec::new();
            for t in v {
                let t = t.trim().to_lowercase();
                if t.chars().count() >= 2 && t.chars().count() <= 60 && !out.contains(&t) {
                    out.push(t);
                }
            }
            out.truncate(8);
            out
        };
        self.sex = self.sex.filter(|s| s == "M" || s == "F");
        self.age_min = self.age_min.filter(|a| (0..=130).contains(a));
        self.age_max = self.age_max.filter(|a| (0..=130).contains(a));
        self.diagnosis_terms = clean(self.diagnosis_terms);
        self.complaint_terms = clean(self.complaint_terms);
        self.medicine_terms = clean(self.medicine_terms);
        self.chronic_terms = clean(self.chronic_terms);
        self
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_becomes_suggestions() {
        let a: ClinicalAnalysis = serde_json::from_value(serde_json::json!({
            "summary": "s",
            "possible_issues": [{ "name": "Acute cystitis", "likelihood": "High", "reason": "pus cells" }],
            "suggested_investigations": [{ "name": "Urine C/S", "reason": "organism" }],
            "advice_en": ["Drink plenty of water", " "],
            "advice_bn": ["প্রচুর পানি পান করবেন", "x", "শুধু বাংলা"],
            "next_plan": "Repeat urine R/E",
            "red_flags": ["Flank pain"]
        }))
        .unwrap();
        let s = a.suggestions();
        let sections: Vec<_> = s.iter().map(|x| x.section).collect();
        assert_eq!(
            sections,
            vec![
                Section::Diagnosis,
                Section::Investigation,
                Section::Advice,
                Section::Advice,
                Section::NextPlan,
                Section::RedFlag
            ]
        );
        assert_eq!(s[0].likelihood.as_deref(), Some("High"));
        assert_eq!(s[2].text_bn.as_deref(), Some("প্রচুর পানি পান করবেন"));
        assert_eq!(s[3].text_en, "শুধু বাংলা");
    }

    #[test]
    fn sanitises_search_filters() {
        let f = SearchFilters {
            sex: Some("X".into()),
            age_min: Some(-3),
            diagnosis_terms: vec!["UTI".into(), "uti".into(), " ".into()],
            ..Default::default()
        }
        .sanitise();
        assert_eq!(f.sex, None);
        assert_eq!(f.age_min, None);
        assert_eq!(f.diagnosis_terms, vec!["uti"]);
    }
}
