//! AI patient summary, smart patient search and SMS wording (PM spec §7.1–7.3).

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::json;
use vd_ai::Tier;
use vd_ai::prompts::{SEARCH, SMS_DRAFT, SUMMARY};
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_domain::clinic_today;
use vd_domain::patient::Patient;
use vd_domain::report::{PatientSummary, SavedSummary, SearchFilters};
use vd_domain::settings::Lang;
use vd_domain::sms::{Vars, render, segments};

use crate::auth::AuthUser;
use crate::routes::ai::{self, Subject};
use crate::state::AppState;

/// Visits sent in full; older history is carried by the previous summary (§7.1).
const RECENT_VISITS: usize = 12;

/// Generates a summary of the patient's whole history, streamed as SSE (§7.1).
/// Sends only de-identified data: age, sex, allergies, conditions, visits, labs, vitals.
pub async fn generate(
    State(state): State<AppState>,
    user: AuthUser,
    Path(patient_id): Path<u64>,
) -> AppResult<impl IntoResponse> {
    user.require(&[Role::Doctor])?;
    ai::ready(&state, user.clinic_id).await?;
    let patient = vd_db::patients::get(&state.pool, user.clinic_id, patient_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    let profile = vd_db::medical_profile::get(&state.pool, user.clinic_id, patient_id)
        .await
        .map_err(AppError::internal)?
        .unwrap_or_default();
    let visits = vd_db::visits::timeline(&state.pool, user.clinic_id, patient_id, true)
        .await
        .map_err(AppError::internal)?;
    let labs = vd_db::reports::abnormal_history(&state.pool, user.clinic_id, patient_id)
        .await
        .map_err(AppError::internal)?;
    let vitals = vd_db::vitals::trend(&state.pool, user.clinic_id, patient_id)
        .await
        .map_err(AppError::internal)?;
    let previous = if visits.len() > RECENT_VISITS {
        vd_db::ai::latest_summary(&state.pool, user.clinic_id, patient_id)
            .await
            .map_err(AppError::internal)?
    } else {
        None
    };

    let (years, _) = ai::age(patient.dob);
    let input = json!({
        "today": clinic_today(),
        "patient": {
            "age_years": years,
            "sex": format!("{:?}", patient.sex),
            "blood_group": patient.blood_group,
            "allergies": profile.allergies,
            "chronic_conditions": profile.chronic_conditions,
            "long_term_medicines": profile.long_term_medicines,
            "past_history": profile.past_history,
            "family_history": profile.family_history,
        },
        "visits_newest_first": visits.iter().filter(|v| v.prescription.is_some()).take(RECENT_VISITS).map(|v| {
            let rx = v.prescription.as_ref();
            json!({
                "date": v.visit.visit_date,
                "type": v.visit.visit_type,
                "complaints": rx.map(|r| r.complaints.clone()),
                "diagnosis": rx.map(|r| r.diagnosis.clone()),
                "medicines": rx.map(|r| r.medicines.iter().map(|m| match &m.generic {
                    Some(g) if !g.is_empty() => format!("{} ({g})", m.brand),
                    _ => m.brand.clone(),
                }).collect::<Vec<_>>()),
            })
        }).collect::<Vec<_>>(),
        "older_visit_count": visits.len().saturating_sub(RECENT_VISITS),
        "abnormal_lab_results": labs.iter().map(|(d, t, v, u, s)| json!({ "date": d, "test": t, "value": v, "unit": u, "status": s })).collect::<Vec<_>>(),
        "vitals": vitals.iter().rev().take(6).map(|v| json!({ "date": v.visit_date, "values": v.vitals })).collect::<Vec<_>>(),
        "previous_summary": previous.as_ref().map(|s| json!({ "date": s.created_at.date_naive(), "summary": s.content })),
    });
    let content = json!([{ "type": "text", "text": format!("Summarise this patient's history.\n\n{}", serde_json::to_string_pretty(&input).unwrap_or_default()) }]);

    Ok(ai::sse(move |progress| async move {
        let (answer, _) = ai::call(
            &state,
            &user,
            &SUMMARY,
            "summary",
            Subject {
                visit_id: None,
                patient_id: Some(patient_id),
                report_id: None,
            },
            Tier::Default,
            2000,
            content,
            Some(progress),
        )
        .await?;
        let summary: PatientSummary = serde_json::from_value(answer.input).map_err(|_| {
            AppError::Conflict("the AI answer could not be read — please try again".into())
        })?;
        let id = vd_db::ai::save_summary(
            &state.pool,
            user.clinic_id,
            patient_id,
            &summary,
            &answer.model,
            &ai::prompt(&state, &SUMMARY).await?.0,
            answer.usage.input_tokens,
            answer.usage.output_tokens,
            user.id,
        )
        .await
        .map_err(AppError::internal)?;
        user.audit(&state, "ai_summary", id, "create").await?;
        vd_db::ai::latest_summary(&state.pool, user.clinic_id, patient_id)
            .await
            .map_err(AppError::internal)?
            .ok_or(AppError::NotFound)
    }))
}

pub async fn latest(
    State(state): State<AppState>,
    user: AuthUser,
    Path(patient_id): Path<u64>,
) -> AppResult<Json<Option<SavedSummary>>> {
    user.require(&[Role::Doctor])?;
    let s = vd_db::ai::latest_summary(&state.pool, user.clinic_id, patient_id)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(s))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

#[derive(Serialize)]
pub struct SearchResult {
    /// How the sentence was understood, shown so the doctor can judge the results.
    filters: SearchFilters,
    items: Vec<Patient>,
}

/// Natural-language patient search (§7.2): Claude turns the sentence into filters;
/// no patient data is sent to it.
pub async fn smart_search(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<SearchQuery>,
) -> AppResult<Json<SearchResult>> {
    user.require(&[Role::Doctor])?;
    ai::ready(&state, user.clinic_id).await?;
    let text = q.q.trim();
    if text.chars().count() < 3 || text.chars().count() > 300 {
        return Err(AppError::Validation(
            "describe the patients in 3 to 300 characters".into(),
        ));
    }
    let today = clinic_today();
    let content = json!([{ "type": "text", "text": format!("Today is {today}. Request: {text}") }]);
    let (answer, _) = ai::call(
        &state,
        &user,
        &SEARCH,
        "search",
        Subject::default(),
        Tier::Fast,
        500,
        content,
        None,
    )
    .await?;
    let filters = serde_json::from_value::<SearchFilters>(answer.input)
        .unwrap_or_default()
        .sanitise();
    if filters.is_empty() {
        return Err(AppError::Validation(
            "could not turn that into a search — try e.g. \"female patients with UTI in the last 3 months\"".into(),
        ));
    }
    let items = vd_db::patients::filter(&state.pool, user.clinic_id, &filters, today)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(SearchResult { filters, items }))
}

#[derive(Serialize)]
pub struct Draft {
    body: String,
    segments: usize,
}

/// AI wording for a follow-up reminder (§7.3). Claude sees the template with its
/// placeholders, never the patient's name; the name is filled in here. The
/// assistant still reviews the text before sending.
pub async fn sms_draft(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<Json<Draft>> {
    let settings = ai::ready(&state, user.clinic_id).await?;
    let (patient, due) = vd_db::followups::for_reminder(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    let lang = settings.sms.language;
    let template = settings.sms.templates.followup_reminder.get(lang);
    let content = json!([{
        "type": "text",
        "text": format!(
            "Language: {}. Purpose: remind the patient that their follow-up visit is due on {{date}}. Clinic template:\n{template}",
            match lang { Lang::En => "English", Lang::Bn => "Bangla" }
        )
    }]);
    let (answer, _) = ai::call(
        &state,
        &user,
        &SMS_DRAFT,
        "sms_draft",
        Subject {
            patient_id: Some(patient.id),
            ..Default::default()
        },
        Tier::Fast,
        400,
        content,
        None,
    )
    .await?;
    let wording = answer.input["text"]
        .as_str()
        .unwrap_or_default()
        .trim()
        .to_string();
    if wording.is_empty() {
        return Err(AppError::Conflict(
            "the AI answer was empty — please try again".into(),
        ));
    }
    let clinic = vd_db::clinics::get(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?;
    let doctor = vd_db::users::clinic_doctor_name(&state.pool, user.clinic_id)
        .await
        .map_err(AppError::internal)?
        .unwrap_or_default();
    let name = match (lang, &patient.name_bn) {
        (Lang::Bn, Some(bn)) => bn.clone(),
        _ => [patient.title.as_deref(), Some(patient.name_en.as_str())]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" "),
    };
    let body = render(
        &wording,
        &Vars {
            name: &name,
            date: &due.format("%d/%m/%Y").to_string(),
            time: "",
            doctor: &doctor,
            clinic: &clinic.name,
            phone: clinic.phone.as_deref().unwrap_or_default(),
        },
        lang,
    );
    Ok(Json(Draft {
        segments: segments(&body),
        body,
    }))
}
