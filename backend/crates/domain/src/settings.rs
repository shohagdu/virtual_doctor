//! Per-clinic settings kept in `clinics.settings` (Patient_Management_Spec §12): visit
//! fees, chamber schedule and appointment slots, SMS options and templates.

use chrono::{Datelike, Duration, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use vd_common::AppError;

use crate::text::check_len;
use crate::visit::VisitType;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ClinicSettings {
    pub fees: Fees,
    /// Chamber sessions; a day may have more than one (e.g. morning and evening).
    pub schedule: Vec<ChamberSession>,
    pub slot_minutes: u16,
    /// Most appointments booked for one day; `None` means no limit.
    pub daily_limit: Option<u16>,
    pub sms: SmsSettings,
}

/// Consultation fees in taka by visit type (§5.8).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Fees {
    pub new: u32,
    pub followup: u32,
    pub report: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChamberSession {
    /// 0 = Sunday … 6 = Saturday.
    pub weekday: u8,
    #[serde(with = "hhmm")]
    pub start: NaiveTime,
    #[serde(with = "hhmm")]
    pub end: NaiveTime,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    En,
    Bn,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SmsSettings {
    pub enabled: bool,
    pub language: Lang,
    /// Send a confirmation when an appointment is booked or moved.
    pub confirm_on_book: bool,
    /// Send a reminder the day before an appointment (§5.6).
    pub remind_day_before: bool,
    pub templates: SmsTemplates,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SmsTemplates {
    pub appointment_confirm: Template,
    pub appointment_reminder: Template,
    pub followup_reminder: Template,
}

/// One message in both languages. Placeholders: {name} {date} {time} {doctor}
/// {clinic} {phone}. There is deliberately no placeholder for clinical details:
/// SMS text never carries a diagnosis (§7.3).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Template {
    pub en: String,
    pub bn: String,
}

impl Template {
    fn new(en: &str, bn: &str) -> Self {
        Self {
            en: en.into(),
            bn: bn.into(),
        }
    }

    pub fn get(&self, lang: Lang) -> &str {
        match lang {
            Lang::En => &self.en,
            Lang::Bn => &self.bn,
        }
    }
}

impl Default for Fees {
    fn default() -> Self {
        Self {
            new: 800,
            followup: 500,
            report: 300,
        }
    }
}

impl Fees {
    pub fn for_type(&self, t: VisitType) -> u32 {
        match t {
            VisitType::New => self.new,
            VisitType::Followup => self.followup,
            VisitType::Report => self.report,
        }
    }
}

impl Default for SmsSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            language: Lang::Bn,
            confirm_on_book: true,
            remind_day_before: true,
            templates: SmsTemplates::default(),
        }
    }
}

impl Default for SmsTemplates {
    fn default() -> Self {
        Self {
            appointment_confirm: Template::new(
                "Dear {name}, your appointment with {doctor} at {clinic} is booked for {date}, {time}.",
                "প্রিয় {name}, {clinic}-এ {doctor}-এর সাথে আপনার অ্যাপয়েন্টমেন্ট {date}, {time}-এ নিশ্চিত হয়েছে।",
            ),
            appointment_reminder: Template::new(
                "Reminder: {name}, you have an appointment with {doctor} tomorrow ({date}) at {time}. {clinic}",
                "স্মরণিকা: {name}, আগামীকাল ({date}) {time}-এ {doctor}-এর সাথে আপনার অ্যাপয়েন্টমেন্ট। {clinic}",
            ),
            followup_reminder: Template::new(
                "Dear {name}, your follow-up visit with {doctor} is due on {date}. For a serial call {phone}. {clinic}",
                "প্রিয় {name}, {date} তারিখে {doctor}-এর কাছে আপনার ফলো-আপ ভিজিট। সিরিয়ালের জন্য কল করুন {phone}। {clinic}",
            ),
        }
    }
}

impl Default for ClinicSettings {
    fn default() -> Self {
        let t = |h| NaiveTime::from_hms_opt(h, 0, 0).expect("valid hour");
        // Saturday to Thursday evenings; Friday off.
        let schedule = [6, 0, 1, 2, 3, 4]
            .into_iter()
            .map(|weekday| ChamberSession {
                weekday,
                start: t(17),
                end: t(21),
            })
            .collect();
        Self {
            fees: Fees::default(),
            schedule,
            slot_minutes: 10,
            daily_limit: None,
            sms: SmsSettings::default(),
        }
    }
}

const MAX_FEE: u32 = 100_000;
pub const MAX_SMS_CHARS: usize = 480;

impl ClinicSettings {
    pub fn validate(&mut self) -> Result<(), AppError> {
        let bad = |m: &str| Err(AppError::Validation(m.into()));
        let f = self.fees;
        if [f.new, f.followup, f.report].iter().any(|&x| x > MAX_FEE) {
            return bad("fees: at most 100000 taka");
        }
        if !(5..=120).contains(&self.slot_minutes) {
            return bad("slot length: 5 to 120 minutes");
        }
        if matches!(self.daily_limit, Some(n) if n == 0 || n > 500) {
            return bad("daily limit: 1 to 500");
        }
        if self.schedule.len() > 21 {
            return bad("schedule: at most 21 sessions");
        }
        for s in &self.schedule {
            if s.weekday > 6 {
                return bad("schedule: weekday must be 0 (Sunday) to 6 (Saturday)");
            }
            if s.start >= s.end {
                return bad("schedule: a session must end after it starts");
            }
        }
        self.schedule.sort_by_key(|s| (s.weekday, s.start));
        for w in self.schedule.windows(2) {
            if w[0].weekday == w[1].weekday && w[1].start < w[0].end {
                return bad("schedule: sessions on the same day overlap");
            }
        }
        let t = &mut self.sms.templates;
        for (name, tpl) in [
            ("appointment confirmation", &mut t.appointment_confirm),
            ("appointment reminder", &mut t.appointment_reminder),
            ("follow-up reminder", &mut t.followup_reminder),
        ] {
            for text in [&mut tpl.en, &mut tpl.bn] {
                *text = text.trim().to_string();
                if text.is_empty() {
                    return Err(AppError::Validation(format!(
                        "SMS template ({name}) is empty"
                    )));
                }
                check_len(text, "SMS template", MAX_SMS_CHARS)?;
            }
        }
        Ok(())
    }

    /// Appointment start times on `date`, from the chamber sessions on that weekday.
    pub fn slots(&self, date: NaiveDate) -> Vec<NaiveTime> {
        let weekday = date.weekday().num_days_from_sunday() as u8;
        let step = Duration::minutes(self.slot_minutes.max(5) as i64);
        let mut out = Vec::new();
        for s in self.schedule.iter().filter(|s| s.weekday == weekday) {
            let mut t = s.start;
            // `overflowing_add_signed` guards against wrapping past midnight.
            while let (next, 0) = t.overflowing_add_signed(step) {
                if next > s.end || next <= t {
                    break;
                }
                out.push(t);
                t = next;
            }
        }
        out.sort();
        out
    }
}

/// Times as "HH:MM" in JSON.
mod hhmm {
    use chrono::NaiveTime;
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<S: Serializer>(t: &NaiveTime, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&t.format("%H:%M").to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<NaiveTime, D::Error> {
        let s = String::deserialize(d)?;
        NaiveTime::parse_from_str(&s, "%H:%M")
            .or_else(|_| NaiveTime::parse_from_str(&s, "%H:%M:%S"))
            .map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hm(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).unwrap()
    }

    #[test]
    fn slots_follow_the_schedule() {
        let mut s = ClinicSettings {
            schedule: vec![
                ChamberSession {
                    weekday: 1,
                    start: hm(17, 0),
                    end: hm(18, 0),
                },
                ChamberSession {
                    weekday: 1,
                    start: hm(9, 0),
                    end: hm(9, 25),
                },
            ],
            slot_minutes: 10,
            ..Default::default()
        };
        s.validate().unwrap();
        // 2026-10-12 is a Monday.
        let monday = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
        let slots = s.slots(monday);
        assert_eq!(slots.first(), Some(&hm(9, 0)));
        assert_eq!(slots[1], hm(9, 10));
        // 09:20 + 10 min would end after 09:25, so it is not offered.
        assert_eq!(slots[2], hm(17, 0));
        assert_eq!(slots.last(), Some(&hm(17, 50)));
        assert_eq!(slots.len(), 8);
        assert!(s.slots(monday.succ_opt().unwrap()).is_empty());
    }

    #[test]
    fn rejects_overlapping_sessions() {
        let mut s = ClinicSettings {
            schedule: vec![
                ChamberSession {
                    weekday: 2,
                    start: hm(17, 0),
                    end: hm(20, 0),
                },
                ChamberSession {
                    weekday: 2,
                    start: hm(19, 0),
                    end: hm(21, 0),
                },
            ],
            ..Default::default()
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn partial_json_takes_defaults() {
        let s: ClinicSettings =
            serde_json::from_str(r#"{"fees":{"new":1000},"slot_minutes":15}"#).unwrap();
        assert_eq!(s.fees.new, 1000);
        assert_eq!(s.fees.followup, 500);
        assert_eq!(s.slot_minutes, 15);
        assert!(!s.sms.templates.followup_reminder.bn.is_empty());
        let json = serde_json::to_string(&ClinicSettings::default()).unwrap();
        assert!(json.contains(r#""start":"17:00""#));
    }
}
