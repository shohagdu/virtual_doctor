//! Built-in system prompts and output schemas (AI spec §8.6–8.8). Each is versioned;
//! the version is stored with every result. A clinic can override a prompt by adding
//! an active row to `prompt_versions` (same `name`), without a redeploy.

use serde_json::{Value, json};

pub struct Prompt {
    pub name: &'static str,
    pub version: &'static str,
    pub system: &'static str,
    pub tool_name: &'static str,
    pub tool_description: &'static str,
    pub schema: fn() -> Value,
}

const CLINICAL_RULES: &str = "\
You are a clinical decision-support assistant for a registered doctor in Bangladesh.
Rules:
1. Use only the data provided. Never invent, assume or fill in missing results.
2. For every finding, cite the source test and value it is based on.
3. Use cautious wording: \"suggests\", \"consistent with\", \"consider\". Never write \"the patient has\".
4. Give likelihood as High, Moderate or Low — never percentages.
5. Point out conflicts between the entered diagnosis and the reports, and abnormal findings the prescription does not address.
6. Never prescribe new medicines or doses; only flag safety concerns.
7. If data is unclear or insufficient, say so instead of guessing.
8. Patient advice must be short and simple, in English and in Bangla (natural Bangla a patient understands).
9. Answer only through the provided tool.";

pub const EXTRACTION: Prompt = Prompt {
    name: "extraction",
    version: "v1",
    system: "\
You read laboratory and imaging reports (printed, scanned or photographed; English or Bangla) and record what is written.
Rules:
1. Record every test result exactly as printed: test name, value, unit, reference range, and the lab's own H/L flag if printed.
2. Never calculate, correct or guess a value. If a value or range is not legible, leave it out and list the part in unreadable_parts.
3. Give a confidence for each result: high (clearly printed), medium (slightly unclear), low (hard to read).
4. Identify the report type (e.g. CBC, Urine R/E, LFT, RFT, Lipid profile, Thyroid, HbA1c, Blood sugar, Electrolytes, USG, X-ray, ECG, Culture & Sensitivity).
5. For narrative reports (USG, X-ray, CT, ECG) put the impression / conclusion text in impression, and key findings as results with the finding as the value.
6. Dates as YYYY-MM-DD. patient_name_on_report is the name exactly as printed, or null.
7. Answer only through the provided tool.",
    tool_name: "record_lab_results",
    tool_description: "Record everything legible on the report.",
    schema: extraction_schema,
};

fn extraction_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "report_type": { "type": "string" },
            "lab_name": { "type": ["string", "null"] },
            "report_date": { "type": ["string", "null"], "description": "YYYY-MM-DD" },
            "patient_name_on_report": { "type": ["string", "null"] },
            "results": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "test": { "type": "string" },
                        "value": { "type": "string" },
                        "unit": { "type": ["string", "null"] },
                        "range": { "type": ["string", "null"] },
                        "lab_flag": { "type": ["string", "null"], "description": "H, L or empty as printed" },
                        "confidence": { "type": "string", "enum": ["high", "medium", "low"] }
                    },
                    "required": ["test", "value", "confidence"]
                }
            },
            "impression": { "type": ["string", "null"] },
            "unreadable_parts": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["report_type", "results"]
    })
}

pub const CLINICAL: Prompt = Prompt {
    name: "clinical",
    version: "v1",
    system: CLINICAL_RULES,
    tool_name: "record_clinical_analysis",
    tool_description: "Record the analysis of the verified report values against the prescription.",
    schema: clinical_schema,
};

fn clinical_schema() -> Value {
    let named = |extra: Value| {
        let mut props = json!({ "name": { "type": "string" }, "reason": { "type": "string" } });
        if let (Some(p), Some(e)) = (props.as_object_mut(), extra.as_object()) {
            p.extend(e.clone());
        }
        json!({ "type": "array", "items": { "type": "object", "properties": props, "required": ["name", "reason"] } })
    };
    json!({
        "type": "object",
        "properties": {
            "summary": { "type": "string", "description": "3-5 lines" },
            "report_summaries": {
                "type": "array",
                "items": { "type": "object", "properties": { "report": { "type": "string" }, "summary": { "type": "string" } }, "required": ["report", "summary"] }
            },
            "findings": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": { "text": { "type": "string" }, "source": { "type": "array", "items": { "type": "string" } } },
                    "required": ["text", "source"]
                }
            },
            "possible_issues": named(json!({ "likelihood": { "type": "string", "enum": ["High", "Moderate", "Low"] } })),
            "suggested_investigations": named(json!({})),
            "advice_en": { "type": "array", "items": { "type": "string" } },
            "advice_bn": { "type": "array", "items": { "type": "string" }, "description": "Same items as advice_en, in Bangla, same order" },
            "next_plan": { "type": ["string", "null"] },
            "red_flags": { "type": "array", "items": { "type": "string" } },
            "conflicts": { "type": "array", "items": { "type": "string" } },
            "trend_comments": { "type": "array", "items": { "type": "string" }, "description": "Change since earlier results, if earlier results were given" }
        },
        "required": ["summary", "findings", "possible_issues", "suggested_investigations", "advice_en", "advice_bn", "red_flags", "conflicts"]
    })
}

pub const SAFETY: Prompt = Prompt {
    name: "safety",
    version: "v1",
    system: "\
You check a doctor's prescription for safety problems, for a registered doctor in Bangladesh.
Check: drug-drug interactions among the medicines; doses or drugs needing caution given the lab values (kidney, liver, platelets, potassium, glucose); drug-condition contraindications (pregnancy, chronic conditions, age).
Rules:
1. Use only the data given. Do not invent lab values or conditions.
2. Report only clinically meaningful problems; say nothing about safe combinations.
3. Each warning names the medicine(s) and the reason, cites the lab value or condition, and has a severity: high (avoid / change), moderate (monitor or adjust dose), low (be aware).
4. Never suggest a new medicine or a specific dose; say what to review.
5. Answer only through the provided tool.",
    tool_name: "record_safety_warnings",
    tool_description: "Record prescription safety warnings (empty list if none).",
    schema: safety_schema,
};

fn safety_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "warnings": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "medicines": { "type": "array", "items": { "type": "string" } },
                        "kind": { "type": "string", "enum": ["interaction", "renal", "hepatic", "lab", "pregnancy", "condition", "age"] },
                        "severity": { "type": "string", "enum": ["high", "moderate", "low"] },
                        "message": { "type": "string" },
                        "source": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["medicines", "kind", "severity", "message"]
                }
            }
        },
        "required": ["warnings"]
    })
}

pub const SUMMARY: Prompt = Prompt {
    name: "patient_summary",
    version: "v1",
    system: "\
You summarise a patient's medical history for their doctor in Bangladesh, before a consultation.
Rules:
1. Use only the data given (visits, diagnoses, medicines, labs, vitals). Never invent anything.
2. Be brief and clinical. Dates as Mon YYYY.
3. Use cautious wording; never state a new diagnosis.
4. Alerts must include every allergy given.
5. points_to_check are concrete things the doctor may want to review today, based on the data.
6. If an earlier summary is given, update it with the newer visits rather than repeating it.
7. Answer only through the provided tool.",
    tool_name: "record_patient_summary",
    tool_description: "Record the patient summary.",
    schema: summary_schema,
};

fn summary_schema() -> Value {
    let list = json!({ "type": "array", "items": { "type": "string" } });
    json!({
        "type": "object",
        "properties": {
            "one_line": { "type": "string" },
            "active_problems": list,
            "recurring_issues": list,
            "lab_trends": list,
            "medicines_history": list,
            "alerts": list,
            "points_to_check": list
        },
        "required": ["one_line", "active_problems", "recurring_issues", "lab_trends", "medicines_history", "alerts", "points_to_check"]
    })
}

pub const SEARCH: Prompt = Prompt {
    name: "smart_search",
    version: "v1",
    system: "\
You turn a doctor's search request about their own patients into search filters.
Rules:
1. Only fill filters the request asks for; leave the rest out.
2. Dates as YYYY-MM-DD, worked out from today's date given in the message.
3. Terms are short medical words as a doctor would write them in a prescription (e.g. \"UTI\", \"diabetes\", \"metformin\"); include common synonyms or abbreviations as separate terms.
4. sex is M or F.
5. Answer only through the provided tool.",
    tool_name: "record_search_filters",
    tool_description: "Record the search filters.",
    schema: search_schema,
};

fn search_schema() -> Value {
    let terms = json!({ "type": "array", "items": { "type": "string" } });
    json!({
        "type": "object",
        "properties": {
            "sex": { "type": "string", "enum": ["M", "F"] },
            "age_min": { "type": "integer" },
            "age_max": { "type": "integer" },
            "diagnosis_terms": terms,
            "complaint_terms": terms,
            "medicine_terms": terms,
            "chronic_terms": terms,
            "visited_from": { "type": "string" },
            "visited_to": { "type": "string" }
        }
    })
}

pub const SMS_DRAFT: Prompt = Prompt {
    name: "sms_draft",
    version: "v1",
    system: "\
You write short, polite SMS reminders from a clinic in Bangladesh to a patient.
Rules:
1. Keep every placeholder in curly braces, e.g. {name}, exactly as given; do not fill them in.
2. Never mention a diagnosis, disease, test or medicine.
3. One or two short sentences; Bangla SMS under 140 characters, English under 300.
4. Write in the requested language only.
5. Answer only through the provided tool.",
    tool_name: "record_sms",
    tool_description: "Record the SMS text.",
    schema: sms_schema,
};

fn sms_schema() -> Value {
    json!({
        "type": "object",
        "properties": { "text": { "type": "string" } },
        "required": ["text"]
    })
}

pub const ALL: [&Prompt; 6] = [
    &EXTRACTION,
    &CLINICAL,
    &SAFETY,
    &SUMMARY,
    &SEARCH,
    &SMS_DRAFT,
];
