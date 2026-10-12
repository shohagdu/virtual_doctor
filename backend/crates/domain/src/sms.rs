//! SMS text helpers (PM-17): template filling, segment counting, phone numbers.

use serde::Serialize;

use crate::settings::Lang;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SmsKind {
    AppointmentConfirm,
    AppointmentReminder,
    FollowupReminder,
}

impl SmsKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AppointmentConfirm => "appointment_confirm",
            Self::AppointmentReminder => "appointment_reminder",
            Self::FollowupReminder => "followup_reminder",
        }
    }
}

/// Values for a template's placeholders.
#[derive(Clone, Debug, Default)]
pub struct Vars<'a> {
    pub name: &'a str,
    pub date: &'a str,
    pub time: &'a str,
    pub doctor: &'a str,
    pub clinic: &'a str,
    pub phone: &'a str,
}

/// Fills a template. Unknown `{…}` text is left as written.
pub fn render(template: &str, v: &Vars, lang: Lang) -> String {
    let digits = |s: &str| match lang {
        Lang::En => s.to_string(),
        Lang::Bn => bn_digits(s),
    };
    template
        .replace("{name}", v.name)
        .replace("{date}", &digits(v.date))
        .replace("{time}", &digits(v.time))
        .replace("{doctor}", v.doctor)
        .replace("{clinic}", v.clinic)
        .replace("{phone}", &digits(v.phone))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Western digits to Bangla digits (০–৯).
pub fn bn_digits(s: &str) -> String {
    s.chars()
        .map(|c| match c.to_digit(10) {
            Some(d) => char::from_u32('০' as u32 + d).unwrap_or(c),
            None => c,
        })
        .collect()
}

/// How many SMS parts a message needs: 160/153 characters for plain text, 70/67 for
/// Unicode (Bangla). Plain text is approximated as ASCII.
pub fn segments(text: &str) -> usize {
    let n = text.chars().count();
    let (single, multi) = if text.is_ascii() {
        (160, 153)
    } else {
        (70, 67)
    };
    if n <= single { 1 } else { n.div_ceil(multi) }
}

/// A Bangladeshi mobile number as 8801XXXXXXXXX, the form SMS gateways expect.
pub fn bd_msisdn(phone: &str) -> Option<String> {
    let digits: String = phone.chars().filter(char::is_ascii_digit).collect();
    let local = digits
        .strip_prefix("880")
        .map(|d| format!("0{d}"))
        .unwrap_or(digits);
    let valid =
        local.len() == 11 && local.starts_with("01") && matches!(local.as_bytes()[2], b'3'..=b'9');
    valid.then(|| format!("88{local}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_templates() {
        let v = Vars {
            name: "Mr Rahim",
            date: "13/10/2026",
            time: "17:30",
            doctor: "Dr Demo",
            clinic: "My Chamber",
            phone: "01711000000",
        };
        assert_eq!(
            render(
                "Dear {name}, {date} {time} with {doctor}.  {clinic}",
                &v,
                Lang::En
            ),
            "Dear Mr Rahim, 13/10/2026 17:30 with Dr Demo. My Chamber"
        );
        assert_eq!(
            render("{date} {time} {phone}", &v, Lang::Bn),
            "১৩/১০/২০২৬ ১৭:৩০ ০১৭১১০০০০০০"
        );
    }

    #[test]
    fn counts_segments() {
        assert_eq!(segments(&"a".repeat(160)), 1);
        assert_eq!(segments(&"a".repeat(161)), 2);
        assert_eq!(segments(&"ক".repeat(70)), 1);
        assert_eq!(segments(&"ক".repeat(71)), 2);
        assert_eq!(segments(&"ক".repeat(135)), 3);
    }

    #[test]
    fn normalises_phone_numbers() {
        assert_eq!(bd_msisdn("01711-000000").as_deref(), Some("8801711000000"));
        assert_eq!(
            bd_msisdn("+880 1911 000000").as_deref(),
            Some("8801911000000")
        );
        assert_eq!(bd_msisdn("01211000000"), None);
        assert_eq!(bd_msisdn("0171100000"), None);
    }
}
