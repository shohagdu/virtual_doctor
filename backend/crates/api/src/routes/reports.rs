//! Investigation report upload, AI extraction and doctor verification (AI spec §5.1–5.3,
//! §5.7, §9.3).

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use vd_ai::Tier;
use vd_ai::prompts::EXTRACTION;
use vd_common::auth::Role;
use vd_common::{AppError, AppResult};
use vd_domain::lab;
use vd_domain::patient::{Patient, Sex};
use vd_domain::report::{CleanResult, Extraction, MAX_RESULTS, Report, ReportPatch, ReportStatus};
use vd_domain::text::opt;

use crate::auth::AuthUser;
use crate::routes::ai::{self, Subject};
use crate::state::AppState;

/// Claude accepts images up to 5 MB each (base64 counts the raw size here).
const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;
const MAX_PDF_BYTES: usize = 20 * 1024 * 1024;

/// File type from its first bytes, not its name (AI spec §9.2).
fn sniff(b: &[u8]) -> Result<(&'static str, &'static str), AppError> {
    if b.starts_with(b"%PDF") {
        return Ok(("application/pdf", "pdf"));
    }
    if b.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(("image/jpeg", "jpg"));
    }
    if b.starts_with(&[0x89, b'P', b'N', b'G']) {
        return Ok(("image/png", "png"));
    }
    if b.len() > 12 && &b[..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        return Ok(("image/webp", "webp"));
    }
    if b.len() > 12
        && &b[4..8] == b"ftyp"
        && matches!(&b[8..12], b"heic" | b"heix" | b"mif1" | b"msf1" | b"hevc")
    {
        return Err(AppError::Validation(
            "HEIC photos are not supported yet — set the phone camera to JPEG (\"Most Compatible\") or upload a screenshot".into(),
        ));
    }
    Err(AppError::Validation(
        "upload a PDF, JPG, PNG or WEBP file".into(),
    ))
}

/// The browser sends the file name percent-encoded in `X-File-Name`.
fn file_name(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get("x-file-name")?.to_str().ok()?;
    let mut bytes = Vec::with_capacity(raw.len());
    let mut it = raw.bytes();
    while let Some(c) = it.next() {
        if c == b'%' {
            let hex: Vec<u8> = it.by_ref().take(2).collect();
            let s = std::str::from_utf8(&hex).ok()?;
            bytes.push(u8::from_str_radix(s, 16).ok()?);
        } else {
            bytes.push(c);
        }
    }
    let name: String = String::from_utf8(bytes)
        .ok()?
        .chars()
        .filter(|c| !c.is_control())
        .take(150)
        .collect();
    Some(name).filter(|n| !n.trim().is_empty())
}

async fn patient_of(state: &AppState, clinic_id: u64, patient_id: u64) -> AppResult<Patient> {
    vd_db::patients::get(&state.pool, clinic_id, patient_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)
}

async fn load(state: &AppState, clinic_id: u64, id: u64) -> AppResult<Report> {
    vd_db::reports::get(&state.pool, clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)
}

#[derive(Serialize)]
pub struct Uploaded {
    report: Report,
    /// The same file was already uploaded for this patient; that report is returned.
    duplicate: bool,
}

/// Uploads one report file for the visit (FR-01, FR-05). Doctor and assistant. A
/// doctor's upload is read by AI straight away when the clinic allows it.
pub async fn upload(
    State(state): State<AppState>,
    user: AuthUser,
    Path(visit_id): Path<u64>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<(StatusCode, Json<Uploaded>)> {
    user.require(&[Role::Doctor, Role::Assistant])?;
    let visit = vd_db::visits::get(&state.pool, user.clinic_id, visit_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    if body.len() < 64 {
        return Err(AppError::Validation("the file is empty".into()));
    }
    let (mime, ext) = sniff(&body)?;
    let hash = hex(&Sha256::digest(&body));

    if let Some(existing) =
        vd_db::reports::find_by_hash(&state.pool, user.clinic_id, visit.patient_id, &hash)
            .await
            .map_err(AppError::internal)?
    {
        let report = load(&state, user.clinic_id, existing).await?;
        return Ok((
            StatusCode::OK,
            Json(Uploaded {
                report,
                duplicate: true,
            }),
        ));
    }

    // Stored under a name derived from the contents; the original name is only metadata.
    let dir = std::path::Path::new(&state.config.storage_dir)
        .join("reports")
        .join(user.clinic_id.to_string());
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(AppError::internal)?;
    let path = dir.join(format!("{hash}.{ext}"));
    if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
        tokio::fs::write(&path, &body)
            .await
            .map_err(AppError::internal)?;
    }
    let id = vd_db::reports::create(
        &state.pool,
        &vd_db::reports::NewReport {
            clinic_id: user.clinic_id,
            patient_id: visit.patient_id,
            visit_id,
            file_path: &path.to_string_lossy(),
            file_hash: &hash,
            mime_type: mime,
            original_name: file_name(&headers).as_deref(),
            uploaded_by: user.id,
        },
    )
    .await
    .map_err(AppError::internal)?;
    user.audit(&state, "report", id, "upload").await?;

    // Extraction is AI work, which only the doctor runs (§4).
    if user.role == Role::Doctor
        && let Ok(settings) = ai::ready(&state, user.clinic_id).await
        && settings.ai.auto_extract
    {
        // A failed read leaves the report for manual entry; the upload still succeeds.
        let _ = extract_report(&state, &user, id).await;
    }
    let report = load(&state, user.clinic_id, id).await?;
    Ok((
        StatusCode::CREATED,
        Json(Uploaded {
            report,
            duplicate: false,
        }),
    ))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// CALL 1 — extraction (AI spec §8.3): Claude reads the file, the rule engine grades
/// the values. On failure the report is marked failed with the reason.
async fn extract_report(state: &AppState, user: &AuthUser, id: u64) -> AppResult<()> {
    let report = load(state, user.clinic_id, id).await?;
    let patient = patient_of(state, user.clinic_id, report.patient_id).await?;
    let (path, mime, _) = vd_db::reports::file(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    vd_db::reports::set_status(&state.pool, id, ReportStatus::Extracting, None)
        .await
        .map_err(AppError::internal)?;

    let result = async {
        let bytes = tokio::fs::read(&path).await.map_err(AppError::internal)?;
        let limit = if mime == "application/pdf" { MAX_PDF_BYTES } else { MAX_IMAGE_BYTES };
        if bytes.len() > limit {
            return Err(AppError::Validation(
                "the file is too large for AI reading — upload a smaller photo or fewer pages".into(),
            ));
        }
        let data = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let block_type = if mime == "application/pdf" { "document" } else { "image" };
        let (years, _) = ai::age(patient.dob);
        // Only age and sex go with the file; the record's name, phone and ID do not (§12).
        let content = json!([
            { "type": block_type, "source": { "type": "base64", "media_type": mime, "data": data } },
            { "type": "text", "text": format!("Patient: {}, {years} years. Extract all results.", sex_word(patient.sex)) }
        ]);
        let (answer, _) = ai::call(
            state,
            user,
            &EXTRACTION,
            "extraction",
            Subject { visit_id: report.visit_id, patient_id: Some(patient.id), report_id: Some(id) },
            Tier::Default,
            8000,
            content,
            None,
        )
        .await?;
        let extraction: Extraction = serde_json::from_value(answer.input.clone())
            .map_err(|_| AppError::Conflict("the AI answer could not be read — please try again".into()))?;
        store_extraction(state, user, &report, &patient, extraction, &answer.input).await
    }
    .await;

    if let Err(e) = &result {
        let msg = match e {
            AppError::Internal(_) => "something went wrong".to_string(),
            other => other.to_string(),
        };
        vd_db::reports::set_status(&state.pool, id, ReportStatus::Failed, Some(&msg))
            .await
            .map_err(AppError::internal)?;
    }
    result
}

fn sex_word(s: Sex) -> &'static str {
    match s {
        Sex::M => "Male",
        Sex::F => "Female",
        Sex::O => "Other",
    }
}

async fn store_extraction(
    state: &AppState,
    user: &AuthUser,
    report: &Report,
    patient: &Patient,
    x: Extraction,
    raw: &serde_json::Value,
) -> AppResult<()> {
    let mismatch = x.patient_name_on_report.as_deref().is_some_and(|n| {
        let mut names = vec![patient.name_en.as_str()];
        if let Some(bn) = &patient.name_bn {
            names.push(bn);
        }
        !lab::name_matches(n, &names)
    });
    vd_db::reports::set_extraction(&state.pool, report.id, raw, mismatch)
        .await
        .map_err(AppError::internal)?;
    let results: Vec<CleanResult> = x
        .results
        .into_iter()
        .filter_map(|r| r.into_clean())
        .take(MAX_RESULTS)
        .collect();
    let cut = |s: Option<String>, n: usize| {
        s.map(|v| v.trim().chars().take(n).collect::<String>())
            .filter(|v| !v.is_empty())
    };
    let report_type = cut(x.report_type, 50);
    let lab_name = cut(x.lab_name, 150);
    let impression = cut(x.impression, 5000);
    let report_date = x
        .report_date
        .as_deref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
    let (_, days) = ai::age(patient.dob);
    vd_db::reports::save_results(
        &state.pool,
        user.clinic_id,
        report.id,
        &vd_db::reports::Header {
            report_type: report_type.as_deref(),
            lab_name: lab_name.as_deref(),
            report_date,
            impression: impression.as_deref(),
        },
        &results,
        patient.sex,
        days,
        ReportStatus::Extracted,
    )
    .await
    .map_err(AppError::internal)
}

/// Runs (or re-runs) AI reading of a report. Doctor only.
pub async fn extract(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<Json<Report>> {
    user.require(&[Role::Doctor])?;
    ai::ready(&state, user.clinic_id).await?;
    extract_report(&state, &user, id).await?;
    user.audit(&state, "report", id, "extract").await?;
    Ok(Json(load(&state, user.clinic_id, id).await?))
}

/// Saves doctor-verified (or manually entered) values; the rule engine re-grades
/// them (FR-14).
pub async fn save_results(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
    Json(req): Json<ReportPatch>,
) -> AppResult<Json<Report>> {
    user.require(&[Role::Doctor])?;
    let report = load(&state, user.clinic_id, id).await?;
    let patient = patient_of(&state, user.clinic_id, report.patient_id).await?;
    if req.results.len() > MAX_RESULTS {
        return Err(AppError::Validation(format!(
            "at most {MAX_RESULTS} results"
        )));
    }
    let mut rows = Vec::with_capacity(req.results.len());
    for r in req.results {
        if let Some(c) = r.clean()? {
            rows.push(c);
        }
    }
    let report_type = opt(req.report_type, "report type", 50)?;
    let lab_name = opt(req.lab_name, "lab name", 150)?;
    let impression = opt(req.impression, "impression", 5000)?;
    let (_, days) = ai::age(patient.dob);
    vd_db::reports::save_results(
        &state.pool,
        user.clinic_id,
        id,
        &vd_db::reports::Header {
            report_type: report_type.as_deref(),
            lab_name: lab_name.as_deref(),
            report_date: req.report_date,
            impression: impression.as_deref(),
        },
        &rows,
        patient.sex,
        days,
        ReportStatus::Verified,
    )
    .await
    .map_err(AppError::internal)?;
    user.audit(&state, "report", id, "verify").await?;
    Ok(Json(load(&state, user.clinic_id, id).await?))
}

/// Reports and their values: doctor; assistants see the list without values (§4
/// limited history).
fn limit_for(user: &AuthUser, mut reports: Vec<Report>) -> Vec<Report> {
    if user.role != Role::Doctor {
        for r in &mut reports {
            r.results.clear();
            r.impression = None;
        }
    }
    reports
}

pub async fn show(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<Json<Report>> {
    user.require(&[Role::Doctor, Role::Assistant])?;
    let r = load(&state, user.clinic_id, id).await?;
    Ok(Json(limit_for(&user, vec![r]).remove(0)))
}

pub async fn for_visit(
    State(state): State<AppState>,
    user: AuthUser,
    Path(visit_id): Path<u64>,
) -> AppResult<Json<Vec<Report>>> {
    user.require(&[Role::Doctor, Role::Assistant])?;
    let items = vd_db::reports::for_visit(&state.pool, user.clinic_id, visit_id)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(limit_for(&user, items)))
}

pub async fn for_patient(
    State(state): State<AppState>,
    user: AuthUser,
    Path(patient_id): Path<u64>,
) -> AppResult<Json<Vec<Report>>> {
    user.require(&[Role::Doctor, Role::Assistant])?;
    let items = vd_db::reports::for_patient(&state.pool, user.clinic_id, patient_id)
        .await
        .map_err(AppError::internal)?;
    Ok(Json(limit_for(&user, items)))
}

/// The original file, for viewing (authorised; never cached by the browser).
pub async fn file(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<Response> {
    user.require(&[Role::Doctor, Role::Assistant])?;
    let (path, mime, _) = vd_db::reports::file(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::NotFound)?;
    let bytes = tokio::fs::read(&path).await.map_err(AppError::internal)?;
    user.audit(&state, "report", id, "view").await?;
    Ok((
        [
            (header::CONTENT_TYPE, mime),
            (header::CACHE_CONTROL, "private, no-store".into()),
            (header::CONTENT_DISPOSITION, "inline".into()),
        ],
        bytes,
    )
        .into_response())
}

pub async fn delete(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<u64>,
) -> AppResult<StatusCode> {
    user.require(&[Role::Doctor])?;
    if !vd_db::reports::delete(&state.pool, user.clinic_id, id)
        .await
        .map_err(AppError::internal)?
    {
        return Err(AppError::NotFound);
    }
    user.audit(&state, "report", id, "delete").await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct TrendQuery {
    test: Option<String>,
}

#[derive(Serialize)]
pub struct Trends {
    tests: Vec<vd_db::reports::TrendTest>,
    test: Option<String>,
    points: Vec<vd_db::reports::TrendPoint>,
}

/// Repeated tests and the values of one of them over time (FR-39). Doctor only.
pub async fn trends(
    State(state): State<AppState>,
    user: AuthUser,
    Path(patient_id): Path<u64>,
    Query(q): Query<TrendQuery>,
) -> AppResult<Json<Trends>> {
    user.require(&[Role::Doctor])?;
    let tests = vd_db::reports::trend_tests(&state.pool, user.clinic_id, patient_id)
        .await
        .map_err(AppError::internal)?;
    let test = q
        .test
        .or_else(|| tests.first().map(|t| t.test_code.clone()));
    let points = match &test {
        Some(code) => vd_db::reports::trend(&state.pool, user.clinic_id, patient_id, code)
            .await
            .map_err(AppError::internal)?,
        None => vec![],
    };
    Ok(Json(Trends {
        tests,
        test,
        points,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_file_types() {
        assert_eq!(sniff(b"%PDF-1.7 ...").unwrap().0, "application/pdf");
        assert_eq!(
            sniff(&[0xFF, 0xD8, 0xFF, 0xE0, 0, 0]).unwrap().0,
            "image/jpeg"
        );
        assert!(sniff(b"\0\0\0\x18ftypheic\0\0\0\0").is_err());
        assert!(sniff(b"<html>").is_err());
    }

    #[test]
    fn decodes_file_names() {
        let mut h = HeaderMap::new();
        h.insert(
            "x-file-name",
            "CBC%20%E0%A6%B0%E0%A6%BF%E0%A6%AA%E0%A7%8B%E0%A6%B0%E0%A7%8D%E0%A6%9F.pdf"
                .parse()
                .unwrap(),
        );
        assert_eq!(file_name(&h).as_deref(), Some("CBC রিপোর্ট.pdf"));
    }
}
