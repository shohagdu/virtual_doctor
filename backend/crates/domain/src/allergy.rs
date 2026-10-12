//! Allergy check for the ℞ panel (PM-09, §13 "Allergy matches a prescribed generic").

use serde::Serialize;

use crate::prescription::MedicineLine;

/// Drug classes whose members should all be flagged when the patient is allergic to the
/// class or to any one member. `keys` are words a doctor writes in the allergy list;
/// `members` are generic-name stems.
struct DrugClass {
    name: &'static str,
    keys: &'static [&'static str],
    members: &'static [&'static str],
}

const CLASSES: &[DrugClass] = &[
    DrugClass {
        name: "Penicillins",
        keys: &["penicillin"],
        members: &[
            "penicillin",
            "amoxicillin",
            "amoxycillin",
            "ampicillin",
            "cloxacillin",
            "flucloxacillin",
            "piperacillin",
            "pivmecillinam",
            "co-amoxiclav",
        ],
    },
    DrugClass {
        name: "Cephalosporins",
        keys: &["cephalosporin"],
        members: &[
            "cefixime",
            "ceftriaxone",
            "cefuroxime",
            "cefpodoxime",
            "cefaclor",
            "cefadroxil",
            "cefotaxime",
            "ceftazidime",
            "cefepime",
            "cefprozil",
            "cephradine",
            "cephalexin",
            "cefalexin",
        ],
    },
    DrugClass {
        name: "Sulfonamides",
        keys: &["sulfa", "sulpha", "sulfonamide", "sulphonamide"],
        members: &[
            "sulfamethoxazole",
            "sulphamethoxazole",
            "cotrimoxazole",
            "co-trimoxazole",
            "sulfasalazine",
            "sulphasalazine",
            "sulfadiazine",
        ],
    },
    DrugClass {
        name: "NSAIDs",
        keys: &["nsaid"],
        members: &[
            "aspirin",
            "acetylsalicylic",
            "ibuprofen",
            "diclofenac",
            "aceclofenac",
            "naproxen",
            "ketorolac",
            "indomethacin",
            "mefenamic",
            "tolfenamic",
            "piroxicam",
            "meloxicam",
            "etoricoxib",
            "celecoxib",
            "ketoprofen",
            "flurbiprofen",
        ],
    },
    DrugClass {
        name: "Macrolides",
        keys: &["macrolide"],
        members: &[
            "azithromycin",
            "clarithromycin",
            "erythromycin",
            "roxithromycin",
        ],
    },
    DrugClass {
        name: "Quinolones",
        keys: &["quinolone"],
        members: &[
            "ciprofloxacin",
            "levofloxacin",
            "ofloxacin",
            "moxifloxacin",
            "norfloxacin",
            "gatifloxacin",
            "nalidixic",
        ],
    },
    DrugClass {
        name: "Tetracyclines",
        keys: &["tetracycline"],
        members: &["tetracycline", "doxycycline", "minocycline"],
    },
    DrugClass {
        name: "Opioids",
        keys: &["opioid", "opiate"],
        members: &[
            "morphine",
            "codeine",
            "tramadol",
            "pethidine",
            "tapentadol",
            "fentanyl",
            "nalbuphine",
        ],
    },
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AllergyMatch {
    /// The medicine line as written (brand).
    pub brand: String,
    /// The allergy entry it matched.
    pub allergy: String,
    /// The shared drug class, or `None` for a direct name match.
    pub class: Option<&'static str>,
}

fn classes_in(text: &str, with_keys: bool) -> impl Iterator<Item = &'static DrugClass> + '_ {
    CLASSES.iter().filter(move |c| {
        c.members.iter().any(|m| text.contains(m))
            || (with_keys && c.keys.iter().any(|k| text.contains(k)))
    })
}

/// Drug classes a medicine name (brand or generic, any case) belongs to.
pub fn drug_classes(name: &str) -> Vec<&'static str> {
    let n = name.trim().to_lowercase();
    if n.chars().count() < 3 {
        return vec![];
    }
    classes_in(&n, false).map(|c| c.name).collect()
}

/// Medicines that match a recorded allergy, by name or by drug class. An allergy to
/// one member (e.g. "Amoxicillin") flags the whole class (e.g. Flucloxacillin).
/// `generic` should be filled in from the medicine catalogue before calling.
pub fn check(medicines: &[MedicineLine], allergies: &[String]) -> Vec<AllergyMatch> {
    let mut out = Vec::new();
    for m in medicines {
        let names: Vec<String> = [Some(&m.brand), m.generic.as_ref()]
            .into_iter()
            .flatten()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| s.chars().count() >= 3)
            .collect();
        if names.is_empty() {
            continue;
        }
        for allergy in allergies {
            let a = allergy.trim().to_lowercase();
            if a.chars().count() < 3 {
                continue;
            }
            let class = if names
                .iter()
                .any(|n| n.contains(&a) || a.contains(n.as_str()))
            {
                Some(None)
            } else {
                classes_in(&a, true)
                    .find(|c| {
                        names
                            .iter()
                            .any(|n| classes_in(n, false).any(|mc| mc.name == c.name))
                    })
                    .map(|c| Some(c.name))
            };
            if let Some(class) = class {
                out.push(AllergyMatch {
                    brand: m.brand.clone(),
                    allergy: allergy.clone(),
                    class,
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(brand: &str, generic: Option<&str>) -> MedicineLine {
        MedicineLine {
            brand: brand.into(),
            generic: generic.map(Into::into),
            ..Default::default()
        }
    }

    fn allergies(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn direct_name_match() {
        let m = check(
            &[line("Napa", Some("Paracetamol"))],
            &allergies(&["paracetamol"]),
        );
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].class, None);
    }

    #[test]
    fn class_keyword_matches_member_generic() {
        let m = check(
            &[line("Moxacil", Some("Amoxicillin Trihydrate"))],
            &allergies(&["Penicillin"]),
        );
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].class, Some("Penicillins"));
        let m = check(
            &[line("Flubex", Some("Flucloxacillin"))],
            &allergies(&["Amoxicillin"]),
        );
        assert_eq!(m[0].class, Some("Penicillins"));
        let m = check(&[line("Ibu", Some("Ibuprofen"))], &allergies(&["NSAIDs"]));
        assert_eq!(m[0].class, Some("NSAIDs"));
        let m = check(
            &[line("Cotrim", Some("Co-trimoxazole"))],
            &allergies(&["Sulfa drugs"]),
        );
        assert_eq!(m[0].class, Some("Sulfonamides"));
    }

    #[test]
    fn unrelated_medicines_pass() {
        let m = check(
            &[
                line("Napa", Some("Paracetamol")),
                line("Seclo", Some("Omeprazole")),
            ],
            &allergies(&["Penicillin", "Sulfa", "Dust"]),
        );
        assert!(m.is_empty());
    }

    #[test]
    fn names_drug_classes() {
        assert_eq!(drug_classes("Diclofenac Sodium"), vec!["NSAIDs"]);
        assert!(drug_classes("Paracetamol").is_empty());
    }

    #[test]
    fn ignores_short_text() {
        let m = check(&[line("Ab", None)], &allergies(&["ab"]));
        assert!(m.is_empty());
    }
}
