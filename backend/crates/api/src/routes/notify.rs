//! Builds and queues patient SMS (PM-17). Sending itself happens in the jobs worker.

use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use vd_common::{AppError, AppResult};
use vd_domain::appointment::Appointment;
use vd_domain::settings::{ClinicSettings, Lang, Template};
use vd_domain::sms::{SmsKind, Vars, bd_msisdn, render};
use vd_domain::visit::QueuePatient;
use vd_domain::{clinic_to_utc, utc_to_clinic};

use crate::state::AppState;

/// Hour (clinic time) at which day-before reminders go out.
const REMINDER_HOUR: u32 = 10;

/// The patient's name as addressed in the chosen language.
fn patient_name(p: &QueuePatient, lang: Lang) -> String {
    match (lang, &p.name_bn) {
        (Lang::Bn, Some(bn)) => bn.clone(),
        _ => [p.title.as_deref(), Some(p.name_en.as_str())]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn time_text(t: NaiveTime, lang: Lang) -> String {
    match lang {
        Lang::En => t.format("%-I:%M %p").to_string(),
        Lang::Bn => t.format("%H:%M").to_string(),
    }
}

/// Fills `template` for `patient` with the clinic's name, phone and doctor.
pub async fn compose(
    state: &AppState,
    clinic_id: u64,
    settings: &ClinicSettings,
    template: &Template,
    patient: &QueuePatient,
    date: NaiveDate,
    time: Option<NaiveTime>,
) -> AppResult<String> {
    let clinic = vd_db::clinics::get(&state.pool, clinic_id)
        .await
        .map_err(AppError::internal)?;
    let doctor = vd_db::users::clinic_doctor_name(&state.pool, clinic_id)
        .await
        .map_err(AppError::internal)?
        .unwrap_or_default();
    let lang = settings.sms.language;
    Ok(render(
        template.get(lang),
        &Vars {
            name: &patient_name(patient, lang),
            date: &date.format("%d/%m/%Y").to_string(),
            time: &time.map(|t| time_text(t, lang)).unwrap_or_default(),
            doctor: &doctor,
            clinic: &clinic.name,
            phone: clinic.phone.as_deref().unwrap_or_default(),
        },
        lang,
    ))
}

pub struct Outgoing<'a> {
    pub clinic_id: u64,
    pub patient: &'a QueuePatient,
    pub body: &'a str,
    pub kind: SmsKind,
    pub ref_id: u64,
    pub created_by: Option<u64>,
    pub send_after: DateTime<Utc>,
}

/// Queues one message. Errors if the patient's phone is not a mobile number.
pub async fn queue(state: &AppState, m: Outgoing<'_>) -> AppResult<u64> {
    let phone = bd_msisdn(&m.patient.phone).ok_or_else(|| {
        AppError::Validation(format!("{} is not a mobile number", m.patient.phone))
    })?;
    vd_db::sms::queue(
        &state.pool,
        &vd_db::sms::NewSms {
            clinic_id: m.clinic_id,
            patient_id: Some(m.patient.id),
            phone: &phone,
            body: m.body,
            kind: m.kind,
            ref_id: Some(m.ref_id),
            created_by: m.created_by,
            send_after: m.send_after,
        },
    )
    .await
    .map_err(AppError::internal)
}

/// Confirmation now and a reminder the day before, as the clinic's settings say
/// (§5.6). Any not-yet-sent messages for this appointment are withdrawn first, so
/// a moved appointment never gets a reminder for its old time. Failures here never
/// undo the booking; they are logged.
pub async fn appointment_booked(state: &AppState, clinic_id: u64, user_id: u64, a: &Appointment) {
    if let Err(e) = try_appointment_booked(state, clinic_id, user_id, a).await {
        tracing::warn!(appointment = a.id, error = %e, "appointment SMS not queued");
    }
}

async fn try_appointment_booked(
    state: &AppState,
    clinic_id: u64,
    user_id: u64,
    a: &Appointment,
) -> AppResult<()> {
    appointment_withdrawn(state, a.id).await;
    let settings = vd_db::settings::get(&state.pool, clinic_id)
        .await
        .map_err(AppError::internal)?;
    let sms = &settings.sms;
    if !sms.enabled || bd_msisdn(&a.patient.phone).is_none() {
        return Ok(());
    }
    let now = Utc::now();
    if sms.confirm_on_book {
        queue_for_appointment(
            state,
            clinic_id,
            &settings,
            user_id,
            a,
            SmsKind::AppointmentConfirm,
            &sms.templates.appointment_confirm,
            now,
        )
        .await?;
    }
    if sms.remind_day_before {
        let at = clinic_to_utc(
            a.date - Duration::days(1),
            NaiveTime::from_hms_opt(REMINDER_HOUR, 0, 0).unwrap_or(NaiveTime::MIN),
        );
        // Booked too late for a day-before reminder: the confirmation covers it.
        if at > now {
            queue_for_appointment(
                state,
                clinic_id,
                &settings,
                user_id,
                a,
                SmsKind::AppointmentReminder,
                &sms.templates.appointment_reminder,
                at,
            )
            .await?;
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn queue_for_appointment(
    state: &AppState,
    clinic_id: u64,
    settings: &ClinicSettings,
    user_id: u64,
    a: &Appointment,
    kind: SmsKind,
    template: &Template,
    send_after: DateTime<Utc>,
) -> AppResult<u64> {
    let body = compose(
        state,
        clinic_id,
        settings,
        template,
        &a.patient,
        a.date,
        Some(a.time),
    )
    .await?;
    queue(
        state,
        Outgoing {
            clinic_id,
            patient: &a.patient,
            body: &body,
            kind,
            ref_id: a.id,
            created_by: Some(user_id),
            send_after,
        },
    )
    .await
}

/// Withdraws unsent messages for an appointment that was cancelled, moved or
/// marked no-show.
pub async fn appointment_withdrawn(state: &AppState, appointment_id: u64) {
    for kind in [SmsKind::AppointmentConfirm, SmsKind::AppointmentReminder] {
        if let Err(e) = vd_db::sms::cancel_queued(&state.pool, kind, appointment_id).await {
            tracing::warn!(appointment = appointment_id, error = %e, "could not withdraw SMS");
        }
    }
}

/// Clinic-local "now", for checking requested appointment times.
pub fn clinic_now() -> chrono::NaiveDateTime {
    utc_to_clinic(Utc::now())
}
