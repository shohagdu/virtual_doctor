//! AI clinical analysis of a visit's reports, the doctor's review of its suggestions,
//! and the prescription safety check (AI spec §5.4–5.6).

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use vd_ai::Tier;
use vd_ai::prompts::{CLINICAL, SAFETY};
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_domain::allergy::drug_classes;
use vd_domain::lab::Status;
use vd_domain::prescription::MedicineLine;
use vd_domain::report::{
    AnalysisView, ClinicalAnalysis, NewSuggestion, Report, ReportStatus, SafetyAnswer,
    SafetyWarning, Section,
};
use vd_domain::settings::AdviceLanguage;
use vd_domain::text::opt;
use vd_domain::visit::Visit;

use crate::auth::AuthUser;
use crate::routes::ai::{self, Subject};
use crate::state::AppState;

async fn visit_of(state: &AppState, clinic_id: u64, id: u64) -> AppResult<Visit> {
    vd_db::visits::get(&state.pool, clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)
}

/// What is on the ℞ screen now (possibly unsaved), for correlation (FR-22).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ScreenRx {
    complaints: Vec<String>,
    examination: Vec<String>,
    diagnosis: Vec<String>,
    investigations: Vec<String>,
    medicines: Vec<MedicineLine>,
}

fn medicine_text(m: &MedicineLine) -> String {
    let mut s = m.brand.clone();
    if let Some(g) = m.generic.as_deref().filter(|g| !g.is_empty()) {
        s.push_str(&format!(" ({g})"));
    }
    for part in [&m.strength, &m.dose_pattern, &m.duration]
        .into_iter()
        .flatten()
    {
        if !part.is_empty() {
            s.push_str(&format!(" {part}"));
        }
    }
    s
}

fn report_json(r: &Report) -> Value {
    json!({
        "report": r.report_type.clone().unwrap_or_else(|| "Report".into()),
        "date": r.report_date,
        "results": r.results.iter().map(|x| json!({
            "test": x.test_name,
            "value": x.value_text,
            "unit": x.unit,
            "range": x.ref_range_text,
            "status": x.status.map(Status::as_str),
        })).collect::<Vec<_>>(),
        "impression": r.impression,
    })
}

/// CALL 2 — clinical analysis (AI spec §8.3), streamed to the browser as SSE. Uses
/// the visit's read reports, the ℞ on screen, and the patient's age, sex, allergies
/// and chronic conditions — never name, phone or ID.
pub async fn run(
    State(state): State<AppState>,
    user: AuthUser,
    Path(visit_id): Path<u64>,
    body: Option<Json<ScreenRx>>,
) -> AppResult<impl IntoResponse> {
    user.require(&[Role::Doctor])?;
    let settings = ai::ready(&state, user.clinic_id).await?;
    let visit = visit_of(&state, user.clinic_id, visit_id).await?;
    let reports: Vec<Report> = vd_db::reports::for_visit(&state.pool, user.clinic_id, visit_id)
        .await
        .map_err(AppError::internal)?
        .into_iter()
        .filter(|r| {
            matches!(r.status, ReportStatus::Extracted | ReportStatus::Verified)
                && (!r.results.is_empty() || r.impression.is_some())
        })
        .collect();
    if reports.is_empty() {
        return Err(AppError::Conflict(
            "there are no read reports for this visit yet — upload and check them first".into(),
        ));
    }
    let rx = match body {
        Some(Json(b)) => b,
        None => {
            let saved = vd_db::prescriptions::latest(&state.pool, visit_id).await?;
            ScreenRx {
                complaints: saved.body.complaints,
                examination: saved.body.examination,
                diagnosis: saved.body.diagnosis,
                investigations: saved.body.investigations,
                medicines: saved.body.medicines,
            }
        }
    };
    let patient = vd_db::patients::get(&state.pool, user.clinic_id, visit.patient_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    let profile = vd_db::medical_profile::get(&state.pool, user.clinic_id, patient.id)
        .await
        .map_err(AppError::internal)?
        .unwrap_or_default();
    let codes: Vec<String> = reports
        .iter()
        .flat_map(|r| {
            r.results
                .iter()
                .filter(|x| x.value_num.is_some())
                .map(|x| x.test_code.clone())
        })
        .collect();
    let earlier =
        vd_db::reports::earlier_results(&state.pool, user.clinic_id, patient.id, visit_id, &codes)
            .await
            .map_err(AppError::internal)?;
    let vitals = vd_db::vitals::for_visit(&state.pool, visit_id)
        .await
        .map_err(AppError::internal)?;

    let (years, _) = ai::age(patient.dob);
    let language = match settings.ai.advice_language {
        AdviceLanguage::En => "English only (leave advice_bn empty)",
        AdviceLanguage::Bn => "Bangla only (advice_en may be empty)",
        AdviceLanguage::Both => "English and Bangla",
    };
    let input = json!({
        "patient": {
            "age_years": years,
            "sex": format!("{:?}", patient.sex),
            "allergies": profile.allergies,
            "chronic_conditions": profile.chronic_conditions,
            "long_term_medicines": profile.long_term_medicines,
        },
        "prescription_on_screen": {
            "chief_complaints": rx.complaints,
            "on_examination": rx.examination,
            "diagnosis": rx.diagnosis,
            "investigations": rx.investigations,
            "medicines": rx.medicines.iter().filter(|m| !m.brand.trim().is_empty()).map(medicine_text).collect::<Vec<_>>(),
        },
        "vitals": vitals.map(|v| serde_json::to_value(v.vitals).unwrap_or_default()),
        "reports": reports.iter().map(report_json).collect::<Vec<_>>(),
        "earlier_results": earlier.iter().map(|(t, d, v, u)| json!({ "test": t, "date": d, "value": v, "unit": u })).collect::<Vec<_>>(),
        "advice_language": language,
    });
    let content = json!([{
        "type": "text",
        "text": format!("Analyse these verified report values for this visit.\n\n{}", serde_json::to_string_pretty(&input).unwrap_or_default())
    }]);
    // Deeper reasoning for many reports at once (AI spec §8.2).
    let tier = if reports.len() >= 3 {
        Tier::Complex
    } else {
        Tier::Default
    };

    Ok(ai::sse(move |progress| async move {
        let (answer, id) = ai::call(
            &state,
            &user,
            &CLINICAL,
            "clinical",
            Subject {
                visit_id: Some(visit_id),
                patient_id: Some(patient.id),
                report_id: None,
            },
            tier,
            4000,
            content,
            Some(progress),
        )
        .await?;
        let analysis: ClinicalAnalysis = serde_json::from_value(answer.input).map_err(|_| {
            AppError::Conflict("the AI answer could not be read — please try again".into())
        })?;
        vd_db::ai::add_suggestions(&state.pool, id, &analysis.suggestions())
            .await
            .map_err(AppError::internal)?;
        user.audit(&state, "ai_analysis", id, "create").await?;
        vd_db::ai::latest_for_visit(&state.pool, user.clinic_id, visit_id, "clinical")
            .await
            .map_err(AppError::internal)?
            .ok_or(AppError::NotFound)
    }))
}

/// The last analysis of the visit with the doctor's decisions so far.
pub async fn latest(
    State(state): State<AppState>,
    user: AuthUser,
    Path(visit_id): Path<u64>,
) -> AppResult<Json<Option<AnalysisView>>> {
    user.require(&[Role::Doctor])?;
    visit_of(&state, user.clinic_id, visit_id).await?;
    let a = vd_db::ai::latest_for_visit(&state.pool, user.clinic_id, visit_id, "clinical")
        .await
        .map_err(AppError::internal)?;
    Ok(Json(a))
}

#[derive(Deserialize)]
pub struct Action {
    action: String,
    edited_text: Option<String>,
    reason: Option<String>,
}

/// Accept / edit / reject a suggestion, or dismiss a safety warning (FR-32, FR-33,
/// FR-37). Rejections are kept to improve the prompts.
pub async fn act(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
    Json(req): Json<Action>,
) -> AppResult<StatusCode> {
    user.require(&[Role::Doctor])?;
    if !matches!(
        req.action.as_str(),
        "accepted" | "edited" | "rejected" | "pending"
    ) {
        return Err(AppError::Validation("unknown action".into()));
    }
    let edited = opt(req.edited_text, "edited text", 2000)?;
    if req.action == "edited" && edited.is_none() {
        return Err(AppError::Validation("give the edited text".into()));
    }
    let reason = opt(req.reason, "reason", 255)?;
    if !vd_db::ai::act(
        &state.pool,
        user.clinic_id,
        id,
        &req.action,
        edited.as_deref(),
        reason.as_deref(),
        user.id,
    )
    .await
    .map_err(AppError::internal)?
    {
        return Err(AppError::NotFound);
    }
    user.audit(&state, "ai_suggestion", id, &req.action).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct SafetyRequest {
    medicines: Vec<MedicineLine>,
}

#[derive(Serialize)]
pub struct WarningView {
    /// Suggestion id, used to dismiss the warning with a reason.
    id: u64,
    #[serde(flatten)]
    warning: SafetyWarning,
    /// "rules" (built-in checks) or "ai".
    origin: &'static str,
}

#[derive(Serialize)]
pub struct SafetyResult {
    warnings: Vec<WarningView>,
    /// Whether Claude was consulted (otherwise only the built-in rules ran).
    ai_used: bool,
}

/// Built-in checks that need no AI: NSAIDs with raised creatinine or low platelets.
fn rule_warnings(
    medicines: &[MedicineLine],
    labs: &[(String, String, Option<String>, Status)],
) -> Vec<SafetyWarning> {
    let nsaids: Vec<String> = medicines
        .iter()
        .filter(|m| {
            [Some(m.brand.as_str()), m.generic.as_deref()]
                .into_iter()
                .flatten()
                .any(|n| drug_classes(n).contains(&"NSAIDs"))
        })
        .map(|m| m.brand.clone())
        .collect();
    if nsaids.is_empty() {
        return vec![];
    }
    let find = |code: &str, bad: &[Status]| {
        labs.iter()
            .find(|(c, _, _, s)| c == code && bad.contains(s))
            .map(|(_, v, u, _)| {
                format!(
                    "{} {v}{}",
                    code_name(code),
                    u.as_deref().map(|u| format!(" {u}")).unwrap_or_default()
                )
            })
    };
    let mut out = vec![];
    if let Some(src) = find("CREAT", &[Status::High, Status::Critical]) {
        out.push(SafetyWarning {
            medicines: nsaids.clone(),
            kind: "renal".into(),
            severity: "high".into(),
            message: "NSAIDs can worsen kidney function — creatinine is raised. Review the need for an NSAID.".into(),
            source: vec![src],
        });
    }
    if let Some(src) = find("PLT", &[Status::Low, Status::Critical]) {
        out.push(SafetyWarning {
            medicines: nsaids,
            kind: "lab".into(),
            severity: "high".into(),
            message: "NSAIDs raise bleeding risk — platelet count is low.".into(),
            source: vec![src],
        });
    }
    out
}

fn code_name(code: &str) -> &'static str {
    match code {
        "CREAT" => "Creatinine",
        "PLT" => "Platelet",
        _ => "Result",
    }
}

/// CALL 3 — prescription safety check (FR-29–FR-32): built-in rules always, plus
/// Claude when AI is available. Advisory only; each warning can be dismissed.
pub async fn safety(
    State(state): State<AppState>,
    user: AuthUser,
    Path(visit_id): Path<u64>,
    Json(mut req): Json<SafetyRequest>,
) -> AppResult<Json<SafetyResult>> {
    user.require(&[Role::Doctor])?;
    let visit = visit_of(&state, user.clinic_id, visit_id).await?;
    req.medicines.retain(|m| !m.brand.trim().is_empty());
    if req.medicines.is_empty() {
        return Err(AppError::Validation("add medicines to check".into()));
    }
    req.medicines.truncate(40);
    vd_db::medicines::fill_generics(&state.pool, user.clinic_id, &mut req.medicines)
        .await
        .map_err(AppError::internal)?;

    let patient = vd_db::patients::get(&state.pool, user.clinic_id, visit.patient_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    let profile = vd_db::medical_profile::get(&state.pool, user.clinic_id, patient.id)
        .await
        .map_err(AppError::internal)?
        .unwrap_or_default();
    // This visit's results first, then recent abnormal ones from earlier reports.
    let mut labs: Vec<(String, String, Option<String>, Status)> = vec![];
    for r in vd_db::reports::for_visit(&state.pool, user.clinic_id, visit_id)
        .await
        .map_err(AppError::internal)?
    {
        for x in r.results {
            if let Some(s) = x.status {
                labs.push((x.test_code, x.value_text, x.unit, s));
            }
        }
    }
    let history = vd_db::reports::abnormal_history(&state.pool, user.clinic_id, patient.id)
        .await
        .map_err(AppError::internal)?;

    let mut warnings: Vec<(SafetyWarning, &'static str)> = rule_warnings(&req.medicines, &labs)
        .into_iter()
        .map(|w| (w, "rules"))
        .collect();

    let mut ai_used = false;
    let mut log: Option<(u64, Value)> = None;
    if ai::ready(&state, user.clinic_id).await.is_ok() {
        let (years, _) = ai::age(patient.dob);
        let input = json!({
            "patient": {
                "age_years": years,
                "sex": format!("{:?}", patient.sex),
                "allergies": profile.allergies,
                "chronic_conditions": profile.chronic_conditions,
                "gynae_obs_history": profile.gynae_obs_history,
            },
            "medicines": req.medicines.iter().map(medicine_text).collect::<Vec<_>>(),
            "lab_values_this_visit": labs.iter().map(|(c, v, u, s)| json!({ "test": c, "value": v, "unit": u, "status": s.as_str() })).collect::<Vec<_>>(),
            "recent_abnormal_results": history.iter().take(20).map(|(d, t, v, u, s)| json!({ "date": d, "test": t, "value": v, "unit": u, "status": s })).collect::<Vec<_>>(),
        });
        let content = json!([{ "type": "text", "text": format!("Check this prescription.\n\n{}", serde_json::to_string_pretty(&input).unwrap_or_default()) }]);
        let (answer, id) = ai::call(
            &state,
            &user,
            &SAFETY,
            "safety",
            Subject {
                visit_id: Some(visit_id),
                patient_id: Some(patient.id),
                report_id: None,
            },
            Tier::Default,
            2000,
            content,
            None,
        )
        .await?;
        let parsed: SafetyAnswer = serde_json::from_value(answer.input.clone()).unwrap_or_default();
        warnings.extend(
            parsed
                .warnings
                .into_iter()
                .filter(|w| !w.message.trim().is_empty())
                .map(|w| (w, "ai")),
        );
        ai_used = true;
        log = Some((id, answer.input));
    }

    // Built-in checks are logged too (no tokens), so every warning can be dismissed.
    let id = match log {
        Some((id, _)) => id,
        None => vd_db::ai::log_call(
            &state.pool,
            &vd_db::ai::NewCall {
                clinic_id: user.clinic_id,
                visit_id: Some(visit_id),
                patient_id: Some(patient.id),
                report_id: None,
                kind: "safety",
                content: Some(
                    &json!({ "warnings": warnings.iter().map(|(w, _)| w).collect::<Vec<_>>() }),
                ),
                model: "rules",
                prompt_version: "rules-v1",
                input_tokens: 0,
                output_tokens: 0,
                cache_read_tokens: 0,
                latency_ms: None,
                error: None,
                created_by: Some(user.id),
            },
        )
        .await
        .map_err(AppError::internal)?,
    };
    let items: Vec<NewSuggestion> = warnings
        .iter()
        .map(|(w, _)| NewSuggestion {
            section: Section::Safety,
            text_en: w.message.clone(),
            text_bn: None,
            likelihood: Some(w.severity.clone()),
            source_refs: w.source.clone(),
        })
        .collect();
    vd_db::ai::add_suggestions(&state.pool, id, &items)
        .await
        .map_err(AppError::internal)?;
    let ids: Vec<u64> = vd_db::ai::suggestions(&state.pool, id)
        .await
        .map_err(AppError::internal)?
        .into_iter()
        .map(|s| s.id)
        .collect();
    Ok(Json(SafetyResult {
        warnings: warnings
            .into_iter()
            .zip(ids)
            .map(|((warning, origin), id)| WarningView {
                id,
                warning,
                origin,
            })
            .collect(),
        ai_used,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nsaid_with_raised_creatinine_is_flagged() {
        let meds = vec![
            MedicineLine {
                brand: "Clofenac".into(),
                generic: Some("Diclofenac".into()),
                ..Default::default()
            },
            MedicineLine {
                brand: "Napa".into(),
                generic: Some("Paracetamol".into()),
                ..Default::default()
            },
        ];
        let labs = vec![(
            "CREAT".to_string(),
            "2.4".to_string(),
            Some("mg/dL".to_string()),
            Status::High,
        )];
        let w = rule_warnings(&meds, &labs);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].medicines, vec!["Clofenac"]);
        assert_eq!(w[0].source, vec!["Creatinine 2.4 mg/dL"]);
        assert!(rule_warnings(&meds[1..], &labs).is_empty());
    }
}
