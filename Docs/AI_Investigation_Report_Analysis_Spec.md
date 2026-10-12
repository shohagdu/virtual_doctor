# AI-Based Investigation Report Analysis Module
### Feature Specification for Doctor Prescription Software

| Item | Detail |
|---|---|
| Document Type | Product & Functional Specification |
| Module | Investigation Report Upload + AI Findings |
| Version | 1.2 (Draft) — Claude API + tech stack |
| Tech Stack | React 19.2 (frontend) • Rust 1.99 (backend) • MySQL 9.7 LTS (database) |
| AI Provider | Anthropic Claude API (via Anthropic Console) |
| Date | 12 Oct 2026 |
| Target Users | Doctors / Consultants / Chamber Assistants |
| Language Support | English + Bangla (বাংলা) |

---

## 1. Purpose

Doctors spend significant time reading investigation reports (CBC, Urine R/E, USG, LFT, RFT, X-ray, ECG, etc.), finding abnormal values, and writing them into the prescription. This module lets the doctor **upload a patient's investigation report** (PDF / image / photo), and the AI will:

1. Read and extract all values from the report.
2. Detect abnormal findings (High / Low / Critical).
3. Explain each finding in simple, clinical language.
4. Correlate findings with the patient's Chief Complaints, Examination, and Diagnosis already entered in the prescription.
5. Suggest possible issues, differential diagnosis, further investigations, and advice.
6. Let the doctor **accept, edit, or reject** each suggestion and insert it into the prescription with one click.

> **Principle:** AI assists, the doctor decides. No AI suggestion goes into the final prescription without the doctor's confirmation.

---

## 2. Current System (Baseline)

The existing prescription screen contains:

- **Header:** Patient Name, Age, Sex, Date; Settings, PDF, Print buttons.
- **Left Panel (Clinical Sections):** Chief Complaints, On Examination, Past History, Drug History, Investigations, Diagnosis, Treatment Plan, Operation Notes, Advice, Next Plan, Hospitalizations.
- **Right Panel (℞):** Medicine list with dose pattern (e.g., 1+0+1), timing (খাবারের পরে), duration (5 দিন), generic name, and Next Visit date.

The new module plugs into this screen without changing the existing workflow.

---

## 3. Scope

### 3.1 In Scope
- Upload of investigation reports (single or multiple files).
- OCR + AI extraction of test names, values, units, and reference ranges.
- Abnormal value detection and severity grading.
- AI interpretation and patient-issue summary.
- Correlation with current prescription data.
- Suggestions for Diagnosis, Investigations, Advice, Next Plan.
- Report history and trend comparison (same patient, multiple visits).
- Doctor review and approval workflow.

### 3.2 Out of Scope (Version 1)
- Automatic prescribing of medicines without doctor input.
- Direct integration with lab machines (LIS/HL7) — planned for Version 2.
- Radiology image interpretation from raw DICOM images (only the written report is analysed in V1).

---

## 4. User Workflow

```
Doctor opens patient prescription
        │
        ▼
Clicks "Upload Report" (in Investigations section or top bar)
        │
        ▼
Selects file(s): PDF / JPG / PNG / camera photo
        │
        ▼
System runs OCR + AI extraction  (progress indicator shown)
        │
        ▼
"Extracted Values" screen → doctor verifies values (editable table)
        │
        ▼
AI Findings Panel opens:
   • Abnormal findings (colour coded)
   • Report summary
   • Possible issues / differential diagnosis
   • Suggested next investigations & advice
        │
        ▼
Doctor ticks the suggestions to keep → "Add to Prescription"
        │
        ▼
Selected items are inserted into Investigations / Diagnosis / Advice / Next Plan
        │
        ▼
Report + AI analysis saved in patient history
```

---

## 5. Functional Requirements

### 5.1 Report Upload

| ID | Requirement |
|---|---|
| FR-01 | Doctor can upload PDF, JPG, PNG, HEIC files (max 20 MB per file). |
| FR-02 | Multiple reports can be uploaded at once (e.g., CBC + Urine R/E + USG). |
| FR-03 | Mobile/tablet camera capture supported for printed reports. |
| FR-04 | Auto-crop, de-skew, and contrast enhancement for photographed reports. |
| FR-05 | Each upload is linked to the patient ID and visit date. |
| FR-06 | Doctor can tag the report date manually if the AI cannot read it. |

### 5.2 Data Extraction (OCR + AI)

| ID | Requirement |
|---|---|
| FR-07 | Identify report type automatically (CBC, Urine R/E, LFT, RFT, Lipid, Thyroid, HbA1c, Blood Sugar, Electrolytes, USG, X-ray, ECG, Culture & Sensitivity, etc.). |
| FR-08 | Extract: Test name, Result value, Unit, Reference range, Lab flag (H/L). |
| FR-09 | Extract lab name, report date, sample date, and referring doctor if present. |
| FR-10 | For narrative reports (USG, X-ray, CT), extract the "Impression / Conclusion" and key findings as text. |
| FR-11 | Normalise test names (e.g., "Hb", "Haemoglobin", "Hemoglobin" → Hemoglobin). |
| FR-12 | Normalise units (e.g., mmol/L ↔ mg/dL for glucose) where needed. |
| FR-13 | Show a confidence score per extracted value; low-confidence values highlighted for manual check. |
| FR-14 | Doctor can edit any extracted value before analysis. |

### 5.3 Abnormality Detection

| ID | Requirement |
|---|---|
| FR-15 | Compare each value against the lab's printed range; if absent, use a built-in reference range adjusted for **age and sex**. |
| FR-16 | Grade each value: Normal, Borderline, High, Low, Critical. |
| FR-17 | Critical values (e.g., K⁺ > 6.0, Hb < 7 g/dL, Platelet < 50,000) trigger a red alert banner. |
| FR-18 | Pregnancy-specific, paediatric, and elderly ranges supported. |

**Colour code:**

| Status | Colour | Icon |
|---|---|---|
| Normal | Green | ✔ |
| Borderline | Yellow | ● |
| High / Low | Orange | ▲ / ▼ |
| Critical | Red | ⚠ |

### 5.4 AI Interpretation & Patient Findings

| ID | Requirement |
|---|---|
| FR-19 | Generate a short **Report Summary** (3–5 lines) per report. |
| FR-20 | For each abnormal value, give a one-line clinical meaning (e.g., "Pus cells 20–25/HPF — suggests urinary tract infection"). |
| FR-21 | Generate an **Overall Patient Findings** section combining all reports. |
| FR-22 | Correlate with Chief Complaints, On Examination, and existing Diagnosis (e.g., Fever + Burning micturition + Pus cells ↑ → supports UTI). |
| FR-23 | Flag **conflicts** — e.g., diagnosis entered is not supported by reports, or a report finding is not addressed in the prescription. |
| FR-24 | Suggest possible issues / differential diagnoses with a likelihood label (High / Moderate / Low) and reasoning. |
| FR-25 | Suggest further investigations (e.g., Urine C/S if Urine R/E shows infection). |
| FR-26 | Suggest patient advice (e.g., drink plenty of water) in English and Bangla. |
| FR-27 | Suggest follow-up interval for Next Plan. |
| FR-28 | Every AI statement shows the source value it is based on (traceability). |

### 5.5 Prescription Safety Check (Linked to ℞ Panel)

| ID | Requirement |
|---|---|
| FR-29 | Check prescribed medicines against report findings (e.g., renal dose adjustment if Creatinine is high, caution with NSAIDs in low platelet / high creatinine). |
| FR-30 | Check drug–drug interactions among prescribed medicines. |
| FR-31 | Check drug–condition contraindications (pregnancy, liver disease, renal impairment). |
| FR-32 | Warnings are shown as advisory only; the doctor can dismiss with a reason. |

### 5.6 Doctor Review & Insert

| ID | Requirement |
|---|---|
| FR-33 | Each suggestion has a checkbox and an edit (✎) option. |
| FR-34 | "Add Selected to Prescription" inserts items into the matching left-panel section. |
| FR-35 | Inserted investigation results can be written in short form, e.g., `CBC: Hb 10.2 g/dL ▼, WBC 13,500 ▲`. |
| FR-36 | Doctor can choose whether report findings print on the prescription or remain internal. |
| FR-37 | Rejected suggestions are logged (used to improve AI quality). |

### 5.7 History & Trends

| ID | Requirement |
|---|---|
| FR-38 | All reports stored under the patient's profile, sorted by date. |
| FR-39 | Trend chart for repeated tests (Hb, HbA1c, Creatinine, Sugar, TSH, etc.). |
| FR-40 | AI comments on change since last report (e.g., "HbA1c improved from 8.4% to 7.1%"). |

---

## 6. User Interface Design

### 6.1 Entry Points
- **"📄 Upload Report"** button added to the top bar (next to Settings / PDF / Print).
- **Upload icon** next to the ➕ in the **Investigations** section.
- Drag-and-drop area on the AI panel.

### 6.2 AI Findings Panel (Slide-in Drawer, Right Side)

```
┌───────────────────────────────────────────────┐
│ 🤖 AI Report Analysis           [Collapse ✕]  │
├───────────────────────────────────────────────┤
│ Reports: CBC (05 Sep) • Urine R/E • USG       │
├───────────────────────────────────────────────┤
│ ⚠ ABNORMAL FINDINGS                           │
│  ▲ WBC 13,800 /cumm (4,000–11,000)            │
│  ▲ Neutrophil 82% (40–75)                     │
│  ▲ Pus cells 20–25 /HPF (0–5)                 │
│  ▲ RBC (urine) 5–8 /HPF (0–2)                 │
│  ● Albumin (urine) Trace                      │
├───────────────────────────────────────────────┤
│ 📝 SUMMARY                                    │
│  Findings indicate acute bacterial infection  │
│  of the urinary tract...                      │
├───────────────────────────────────────────────┤
│ 🩺 POSSIBLE ISSUES                            │
│  ☑ Acute UTI (Cystitis) — High                │
│  ☐ Acute Pyelonephritis — Low                 │
├───────────────────────────────────────────────┤
│ 🔬 SUGGESTED INVESTIGATIONS                   │
│  ☑ Urine C/S                                  │
│  ☐ S. Creatinine                              │
├───────────────────────────────────────────────┤
│ 💬 ADVICE                                     │
│  ☑ প্রচুর পানি পান করবেন                         │
├───────────────────────────────────────────────┤
│ 💊 PRESCRIPTION CHECK                         │
│  ✔ No interaction found                       │
├───────────────────────────────────────────────┤
│        [ Add Selected to Prescription ]       │
└───────────────────────────────────────────────┘
```

### 6.3 Extracted Values Table (Verification Screen)

| Test | Result | Unit | Ref. Range | Status | Confidence |
|---|---|---|---|---|---|
| Hemoglobin | 11.8 | g/dL | 12–15.5 | ▼ Low | 98% |
| WBC | 13,800 | /cumm | 4,000–11,000 | ▲ High | 97% |
| Platelet | 2,40,000 | /cumm | 1,50,000–4,50,000 | ✔ Normal | 95% |
| ESR | 38 | mm/1st hr | 0–20 | ▲ High | 72% ⚠ verify |

---

## 7. Sample AI Output (Using Current Patient)

**Patient:** Ms Maisha | 18 Years | Female | 07 Sep 2026
**Complaints:** Fever – 1 day; Burning sensation in urine – 2 days
**O/E:** Pulse normal, BP normal, Temp high
**Reports uploaded:** CBC, Urine R/E, USG Abdomen *(values below are illustrative)*

### 7.1 Report Summary

**CBC:** Leucocytosis with neutrophilia and mildly raised ESR — consistent with an acute bacterial infection. Mild low haemoglobin (borderline anaemia).

**Urine R/E:** Pus cells and RBCs raised, nitrite positive, trace albumin — strongly suggests urinary tract infection.

**USG Abdomen:** Both kidneys normal in size and echotexture, no hydronephrosis, no calculus. Urinary bladder wall mildly thickened — may suggest cystitis.

### 7.2 Overall Patient Findings

1. Acute lower urinary tract infection (cystitis) — supported by symptoms + Urine R/E + USG.
2. Systemic inflammatory response (fever, raised WBC, neutrophilia).
3. No evidence of upper tract involvement or obstruction on USG.
4. Mild anaemia — needs evaluation (diet / menstrual history).

### 7.3 Correlation With Prescription
- Diagnosis **"UTI"** — ✔ supported by reports.
- Mild anaemia — ⚠ not addressed in the current prescription.

### 7.4 Suggestions (Doctor to Approve)

| Section | Suggestion | Reason |
|---|---|---|
| Investigations | Urine C/S | Confirm organism and antibiotic sensitivity |
| Diagnosis | Acute cystitis; Mild anaemia | Report-based |
| Advice | Plenty of water (2.5–3 L/day), complete full course of medicine | Standard UTI care |
| Advice (Bangla) | প্রতিদিন পর্যাপ্ত পানি পান করবেন; ওষুধের কোর্স সম্পূর্ণ করবেন | Patient-friendly |
| Next Plan | Review with Urine R/E after treatment | Confirm resolution |
| Red flags | Return if flank pain, vomiting, or high fever persists | Possible pyelonephritis |

> ⚠ *AI-generated suggestions. For clinical decision support only. Final decision rests with the treating physician.*

---

## 8. AI Engine — Claude API (Anthropic Console)

All AI features in this module are powered by the **Claude API**, managed through the **Anthropic Console** (console.anthropic.com). No separate OCR service is required: Claude reads report images and PDFs directly and returns structured data.

### 8.1 Why Claude API

| Need | Claude API Capability |
|---|---|
| Read photographed / scanned reports | **Vision** — images (JPG, PNG, WEBP, GIF) sent directly in the request |
| Read lab PDFs (multi-page) | **PDF support** — PDF sent as a `document` content block (text + page images read together) |
| Mixed Bangla + English reports | Multilingual understanding, Bangla output for advice |
| Reliable JSON for the values table | **Tool use / structured output** with a fixed JSON schema |
| Fixed clinical rules and instructions | **System prompt**, reused across calls |
| Lower cost on repeated instructions | **Prompt caching** for the long system prompt and reference tables |
| Fast feedback in the UI | **Streaming** responses into the AI Findings Panel |
| Overnight re-analysis / bulk history | **Message Batches API** (asynchronous, discounted) |
| Re-using one uploaded report across calls | **Files API** (upload once, reference by file ID) |

### 8.2 Model Selection

| Task | Recommended Model | Reason |
|---|---|---|
| Report extraction (OCR → JSON) | `claude-sonnet-5-5` | Strong vision accuracy, good speed/cost balance |
| Clinical reasoning, correlation, suggestions | `claude-sonnet-5-5` (default) / `claude-opus-5-5` (complex multi-report cases) | Deeper reasoning when many reports and comorbidities are involved |
| Short tasks (report type detection, Bangla advice wording, name normalisation) | `claude-haiku-5-5` | Fastest and lowest cost |

> Model names must be kept configurable in Settings / environment config, because Anthropic releases new model versions over time. Always check the current model list in the Claude docs before release.

### 8.3 Processing Pipeline

```
Doctor uploads report (PDF / image)
        │
        ▼
[Backend] Pre-check: file type, size, image resize/compress, de-skew
        │
        ▼
CALL 1 — Extraction  (claude-sonnet-5-5 + vision/PDF + tool schema)
   Input : report file + patient age/sex
   Output: JSON { report_type, lab_name, date, results[], impression }
        │
        ▼
[Backend] Rule Engine (deterministic, NOT AI)
   • Compare value vs reference range (age/sex adjusted)
   • Set Normal / Borderline / High / Low / Critical
   • Critical value alert
        │
        ▼
Doctor verifies the extracted values table (edits saved)
        │
        ▼
CALL 2 — Clinical Analysis  (claude-sonnet-5-5, streamed)
   Input : verified values + flags + complaints + O/E
           + diagnosis + current medicines
   Output: JSON { summary, findings[], possible_issues[],
                  suggested_investigations[], advice_en[], advice_bn[],
                  next_plan, red_flags[], conflicts[] }
        │
        ▼
CALL 3 — Prescription Safety Check (optional, claude-haiku-5-5 / sonnet)
   Input : medicines (brand + generic) + key lab values
   Output: JSON { warnings[] }   ← cross-checked with drug database
        │
        ▼
AI Findings Panel → Doctor ticks → Insert into prescription
```

Key design rule: **Claude reads and interprets; the backend rule engine decides High/Low numerically.** This keeps abnormal flags 100% reproducible.

### 8.4 System Architecture

```
┌──────────────┐     HTTPS      ┌──────────────────────┐    HTTPS     ┌───────────────┐
│ Doctor App   │ ─────────────► │ Rust Backend (Axum)  │ ───────────► │  Claude API   │
│ React 19 SPA │ ◄───────────── │ (API key stored here)│ ◄─────────── │ (Anthropic)   │
└──────────────┘   streamed     │ • Rule engine        │              └───────────────┘
                   results      │ • De-identification  │
                                │ • Drug database      │
                                │ • Audit log          │
                                │ • MySQL 9.7 + files  │
                                └──────────────────────┘
```

- The **API key never goes to the browser or mobile app**; only the backend calls Claude.
- The backend removes patient name / phone / ID before sending (age and sex are kept because ranges depend on them).

### 8.5 Request Design (Illustrative)

**Call 1 — Extraction (structure only, not code to deploy):**

```
model:        claude-sonnet-5-5
max_tokens:   4000
system:       [Extraction rules — cached]
tools:        [record_lab_results  ← JSON schema of section 8.7]
tool_choice:  force record_lab_results
messages:
  user:
    - document/image block  (the uploaded report)
    - text: "Patient: Female, 18 years. Extract all results."
```

**Call 2 — Clinical Analysis:**

```
model:        claude-sonnet-5-5
max_tokens:   3000
stream:       true
system:       [Clinical assistant rules — cached]
tools:        [record_clinical_analysis ← JSON schema]
messages:
  user:
    - text: verified results JSON + flags
    - text: complaints, O/E, diagnosis, medicines
```

### 8.6 System Prompt Rules (Clinical Assistant)

The system prompt (stored in the backend and versioned) will instruct Claude to:

1. Act as a clinical decision-support assistant for a registered doctor in Bangladesh.
2. Use **only** the values provided; never invent or assume missing results.
3. Cite the source test/value for every finding.
4. Use cautious wording: "suggests", "consistent with", "consider" — never "the patient has".
5. Give likelihood labels (High / Moderate / Low), not percentages.
6. Point out conflicts between the entered diagnosis and the reports.
7. Point out abnormal findings not addressed in the prescription.
8. Write patient advice in simple English **and** Bangla.
9. Never prescribe new medicines or doses on its own; only flag safety concerns.
10. If report quality is poor or the value is unclear, say so instead of guessing.
11. Return output only through the provided tool schema.

Prompts should be drafted and tested in the **Anthropic Console Workbench** before being moved into the backend.

### 8.7 Structured Output Schema — Extraction

```json
{
  "report_type": "Urine R/E",
  "lab_name": "ABC Diagnostic Centre",
  "report_date": "2026-09-05",
  "patient_name_on_report": "Maisha",
  "results": [
    { "test": "Pus Cells", "value": "20-25", "unit": "/HPF", "range": "0-5", "lab_flag": "H", "confidence": "high" },
    { "test": "Nitrite", "value": "Positive", "unit": "", "range": "Negative", "lab_flag": "", "confidence": "high" }
  ],
  "impression": null,
  "unreadable_parts": []
}
```

`patient_name_on_report` is only compared locally by the backend for the name-mismatch warning.

### 8.8 Structured Output Schema — Clinical Analysis

```json
{
  "summary": "Findings are consistent with acute lower urinary tract infection...",
  "findings": [
    { "text": "Raised pus cells and positive nitrite suggest UTI", "source": ["Pus Cells 20-25/HPF", "Nitrite Positive"] }
  ],
  "possible_issues": [
    { "name": "Acute cystitis", "likelihood": "High", "reason": "..." }
  ],
  "suggested_investigations": [ { "name": "Urine C/S", "reason": "..." } ],
  "advice_en": ["Drink plenty of water"],
  "advice_bn": ["প্রচুর পানি পান করবেন"],
  "next_plan": "Review with Urine R/E after treatment",
  "red_flags": ["Flank pain", "Persistent high fever", "Vomiting"],
  "conflicts": ["Mild anaemia not addressed in prescription"]
}
```

### 8.9 Anthropic Console Setup

| Item | Setup |
|---|---|
| Organisation | Create organisation in the Anthropic Console; add billing |
| Workspaces | Separate workspaces for **Development**, **Staging**, **Production** |
| API Keys | One key per workspace; stored in server secret manager / environment variables; rotate regularly |
| Spend limits | Set monthly spend limits per workspace to control cost |
| Rate limits | Check your usage tier limits; request higher limits before large rollout |
| Usage & cost | Monitor via Console Usage / Cost pages; also log tokens per call in your own DB |
| Workbench | Prompt design and testing with sample (de-identified) reports |
| Data handling | Review Anthropic's commercial terms, data retention and privacy policy; ask Anthropic sales about zero-data-retention or BAA-type arrangements if required |

### 8.10 Cost Control

- **Prompt caching** on the long system prompts and reference-range tables (repeated on every call).
- Resize large phone photos before sending (fewer image tokens, same readability).
- Send only relevant PDF pages when a lab file contains ads/cover pages.
- Use `claude-haiku-5-5` for small helper tasks.
- Use the **Message Batches API** for non-urgent work (re-analysing old reports, trend summaries).
- Cache results: the same report file is never analysed twice unless the doctor requests it.
- Track token usage per clinic for billing your own customers (if SaaS).

> Pricing per model changes over time; check the official pricing page in the Claude docs when estimating per-report cost.

### 8.11 Error & Limit Handling

| Situation | Handling |
|---|---|
| Rate limit (HTTP 429) | Retry with exponential backoff; show "Analysing, please wait" |
| Overloaded / server error (5xx) | Retry up to 3 times, then offer manual entry |
| Response not matching schema | Validate JSON on backend; retry once; otherwise show raw text to doctor |
| File too large / too many pages | Compress or split; analyse in parts and merge |
| Timeout | Keep partial results; "Retry analysis" button |
| API key invalid / billing issue | Admin alert; app falls back to manual mode |

### 8.12 Quality Monitoring

- Store prompt version + model name with every AI analysis (for audit and comparison).
- Track doctor accept / edit / reject rate per suggestion type.
- Monthly review of rejected suggestions to improve prompts.
- Re-test the full validation set (section 17) whenever the model or prompt changes.

### 8.13 Reference Links

- Claude API documentation: https://docs.claude.com/en/api/overview
- Anthropic Console: https://console.anthropic.com
- Topics to read before development: Vision, PDF support, Tool use, Prompt caching, Streaming, Message Batches, Files API, Rate limits, Pricing.

---
## 9. Technical Implementation (React + Rust + MySQL)

Full stack details (versions, architecture, folder structure, deployment) are in *Patient_Management_Spec.md*, section 8. This section covers only what is specific to the AI report module.

| Layer | Technology |
|---|---|
| Frontend | React 19.2.x + TypeScript + Vite |
| Backend | Rust 1.99.0 (Edition 2024), Axum + Tokio |
| Database | MySQL 9.7 LTS |
| AI | Claude API, called only from the Rust backend |

### 9.1 Frontend (React) Components

| Component | Role |
|---|---|
| `ReportUploadButton` | Top bar and Investigations ➕ entry point |
| `ReportDropzone` | Drag-and-drop / camera capture; shows upload progress |
| `ExtractedValuesTable` | Editable table with status colours and confidence warnings |
| `AIFindingsDrawer` | Slide-in panel; renders streamed analysis section by section |
| `SuggestionItem` | Checkbox + ✎ edit for each suggestion |
| `CriticalValueBanner` | Red alert for critical results |
| `PrescriptionSafetyAlerts` | Warnings shown under the ℞ panel |
| `LabTrendChart` | Recharts line chart for repeated tests |

- Image preview and basic client-side compression before upload (saves mobile data).
- Upload uses `multipart/form-data`; progress shown with a progress bar.
- AI analysis is received as **Server-Sent Events (SSE)** so the drawer fills in while Claude is still writing.
- "Add Selected to Prescription" updates the prescription state locally (optimistic), then saves through the prescription API.

### 9.2 Backend (Rust) — AI Service

**Claude API client**
- Calls the Claude **Messages API** over HTTPS with `reqwest`, using typed Rust structs (`serde`) for requests and responses. (Check the Claude docs for any official or community Rust SDK at project start; a thin in-house client is enough.)
- API key read from environment / secret store at startup; never sent to the frontend; never logged.
- Model names, `max_tokens`, and prompt versions come from config, not hard-coded.
- Streaming responses from Claude are forwarded to the browser as SSE via Axum.
- Tool-use output is parsed into Rust structs; if it fails validation, retry once, then return a clear error.
- Retries with exponential backoff on HTTP 429 / 5xx; request timeout configured.
- Token usage from every response saved to MySQL for cost tracking.

**File handling**
- Accepted types checked by file signature (not only extension); size limit enforced by Axum body limit.
- Photos resized/compressed with the `image` crate. **HEIC** photos (iPhone) are converted to JPEG on the server before sending to Claude.
- Files stored in file storage (local volume or S3-compatible object storage) with random names; MySQL keeps only the path and a SHA-256 hash.
- Hash used to detect re-uploads of the same report (no repeat AI cost).

**Rule engine (Rust, deterministic)**
- Reference ranges loaded from MySQL (`reference_ranges` table) by test, sex, and age band.
- Unit conversion table for common tests.
- Critical value thresholds configurable per clinic.

**Processing flow**

```
React                     Rust (Axum)                         MySQL / Storage      Claude API
  │ POST /reports (file)    │                                      │                    │
  │────────────────────────►│ validate, hash, resize, store ──────►│                    │
  │                         │ create report (status=extracting) ──►│                    │
  │                         │ Call 1: extraction ─────────────────────────────────────►│
  │                         │◄──────────────────────────────────── structured JSON ─────│
  │                         │ rule engine → flags; save results ──►│                    │
  │◄── results + flags ─────│                                      │                    │
  │ doctor edits/verifies   │                                      │                    │
  │ POST /analyze (SSE)     │                                      │                    │
  │────────────────────────►│ Call 2: clinical analysis (stream) ─────────────────────►│
  │◄── SSE stream ──────────│◄──────────────────────────────────── streamed output ────│
  │                         │ save ai_analyses + tokens ──────────►│                    │
```

### 9.3 API Endpoints (AI Module)

| Method | Endpoint | Purpose |
|---|---|---|
| POST | `/api/v1/visits/{id}/reports` | Upload one or more report files |
| GET | `/api/v1/reports/{id}` | Report details, extraction status, results |
| GET | `/api/v1/reports/{id}/file` | Secure file download (signed / authorised) |
| PUT | `/api/v1/reports/{id}/results` | Save doctor-verified values |
| POST | `/api/v1/visits/{id}/ai-analysis` | Run clinical analysis (SSE stream) |
| GET | `/api/v1/visits/{id}/ai-analysis/latest` | Last analysis for this visit |
| POST | `/api/v1/ai-suggestions/{id}/action` | Accept / edit / reject a suggestion |
| POST | `/api/v1/visits/{id}/prescription/safety-check` | Medicine safety check |
| GET | `/api/v1/patients/{id}/lab-trends?test=` | Trend data for a test |
| GET | `/api/v1/admin/ai-usage?from=&to=` | Token usage and cost report |

---

## 10. Database Design — AI Module (MySQL 9.7 LTS)

Uses the general rules in *Patient_Management_Spec.md* section 9 (InnoDB, `utf8mb4`, UTC times, `clinic_id` on main tables). `patients`, `visits`, and `prescriptions` are defined there.

**reports**

| Column | Type | Notes |
|---|---|---|
| id | BIGINT UNSIGNED PK | |
| clinic_id | BIGINT UNSIGNED | |
| patient_id | BIGINT UNSIGNED | FK patients |
| visit_id | BIGINT UNSIGNED | FK visits |
| report_type | VARCHAR(50) | CBC, Urine R/E, USG… |
| lab_name | VARCHAR(150) NULL | |
| report_date | DATE NULL | |
| file_path | VARCHAR(255) | Storage path |
| file_hash | CHAR(64) | SHA-256; indexed for duplicate detection |
| mime_type | VARCHAR(50) | |
| status | ENUM('uploaded','extracting','extracted','verified','failed') | |
| impression | TEXT NULL | Narrative reports (USG, X-ray) |
| extraction_raw | JSON | Full Claude extraction output |
| uploaded_by | BIGINT UNSIGNED | FK users |
| created_at, updated_at, deleted_at | DATETIME | |

**report_results**

| Column | Type | Notes |
|---|---|---|
| id | BIGINT UNSIGNED PK | |
| report_id | BIGINT UNSIGNED | FK reports |
| test_code | VARCHAR(50) | Normalised name (e.g., HB, WBC) — indexed with patient for trends |
| test_name | VARCHAR(100) | As printed |
| value_text | VARCHAR(50) | Original value ("20-25", "Positive") |
| value_num | DECIMAL(12,4) NULL | Numeric value for rules and charts |
| unit | VARCHAR(30) | |
| ref_range_text | VARCHAR(50) | |
| range_source | ENUM('lab','system') | |
| status | ENUM('normal','borderline','high','low','critical','abnormal') | Set by Rust rule engine |
| confidence | ENUM('high','medium','low') | From extraction |
| edited_by_doctor | TINYINT(1) | |

**reference_ranges** — `id`, `test_code`, `sex` ENUM('M','F','ANY'), `age_min_days`, `age_max_days`, `low`, `high`, `critical_low`, `critical_high`, `unit`, `clinic_id` NULL (NULL = system default).

**ai_analyses** — `id`, `visit_id`, `analysis_type` ENUM('extraction','clinical','safety'), `content` JSON, `claude_model` VARCHAR(50), `prompt_version` VARCHAR(20), `input_tokens` INT, `output_tokens` INT, `cache_read_tokens` INT, `latency_ms` INT, `status`, `error` TEXT NULL, `created_by`, `created_at`.

**ai_suggestions** — `id`, `analysis_id`, `section` ENUM('diagnosis','investigation','advice','next_plan','red_flag'), `text_en`, `text_bn` NULL, `likelihood` NULL, `source_refs` JSON, `action` ENUM('pending','accepted','edited','rejected'), `edited_text` NULL, `reason` NULL, `acted_by`, `acted_at`.

**prompt_versions** — `id`, `name`, `version`, `system_prompt` MEDIUMTEXT, `schema` JSON, `is_active`, `created_at` (lets the team update prompts without redeploying).

---

## 11. Non-Functional Requirements

| Category | Requirement |
|---|---|
| Performance | Single report analysed in < 15 seconds; 3 reports < 30 seconds |
| Accuracy | Extraction accuracy ≥ 95% on printed reports; abnormal flag accuracy ≥ 99% (rule-based) |
| Availability | 99.5% uptime; graceful fallback to manual entry if AI is unavailable |
| Offline | Upload queue works offline; analysis runs when connection returns |
| Usability | Max 3 clicks from upload to "Add to Prescription" |
| Devices | Desktop, tablet, mobile browser |
| Language | English and Bangla UI, Bangla advice output |

---

## 12. Security, Privacy & Compliance

- All patient data encrypted in transit (TLS 1.2+) and at rest (AES-256).
- Role-based access: Doctor, Assistant, Admin.
- Uploaded reports are visible only to the treating doctor / clinic.
- Patient identifiers (name, phone, ID) removed or masked before sending data to the Claude API; only age and sex are sent.
- Claude API key kept only on the backend server, never in the app or browser.
- Audit log of every view, edit, and AI suggestion action.
- Patient consent option for AI-assisted analysis.
- Data retention policy configurable per clinic.
- Follow applicable Bangladesh data-protection rules and medical-practice guidance from BMDC; review with a legal advisor before launch.
- Clear on-screen and printed disclaimer: *"AI-assisted clinical decision support. Not a substitute for clinical judgment."*

---

## 13. Settings (Under ⚙️ Settings)

- Enable / disable AI analysis.
- Auto-analyse on upload (on/off).
- Show report findings on printed prescription (on/off).
- Preferred output language (English / Bangla / Both).
- Custom reference ranges per clinic/lab.
- Specialty mode (Medicine, Gynae, Paediatrics, Surgery, Cardiology) — tunes suggestions.

---

## 14. Error Handling

| Situation | System Behaviour |
|---|---|
| Blurry / unreadable image | Show "Report unclear — please re-upload or enter values manually" |
| Unknown report type | Extract raw text; ask doctor to select type |
| Missing reference range | Use built-in age/sex range, mark as "System range" |
| Conflicting values | Highlight both, ask doctor to confirm |
| AI service timeout | Keep extracted values; offer "Retry analysis" |
| Report of a different patient (name mismatch) | Warning: "Name on report does not match patient" |

---

## 15. Development Phases

| Phase | Deliverables | Duration (Est.) |
|---|---|---|
| Phase 0 | Anthropic Console setup, prompt testing in Workbench; Rust Claude client, MySQL AI tables, React upload/drawer skeleton | 2–3 weeks |
| Phase 1 — MVP | Upload, Claude vision/PDF extraction, verification table, abnormal flags, basic summary | 6–8 weeks |
| Phase 2 | Correlation with complaints/diagnosis, suggestions, insert into prescription | 4–6 weeks |
| Phase 3 | Drug safety checks, Bangla advice, trends & history | 4–6 weeks |
| Phase 4 | LIS/HL7 lab integration, specialty modes, analytics on suggestion acceptance | Ongoing |

---

## 16. Success Metrics

- Average time to write investigation findings reduced by ≥ 50%.
- ≥ 70% of AI suggestions accepted (with or without edit).
- Extraction correction rate by doctors < 5%.
- Zero critical values missed in validation testing.
- Doctor satisfaction score ≥ 4/5.

---

## 17. Testing & Validation Plan

1. Collect 500+ de-identified reports from common Bangladeshi labs (mixed quality).
2. Measure extraction accuracy per report type.
3. Clinical review of AI interpretations by a panel of doctors (Medicine, Gynae, Paediatrics).
4. Test edge cases: handwritten reports, Bangla-only reports, multi-page PDFs, rotated photos.
5. Pilot with 5–10 chambers before full release; collect feedback.

---

## 18. Future Enhancements

- Voice dictation for complaints and examination.
- AI-drafted full prescription from complaints + reports (doctor approval required).
- Patient-friendly report explanation via SMS/WhatsApp in Bangla.
- Direct radiology image (X-ray/ECG) AI reading.
- Integration with pharmacy and lab booking.

---

*End of Document*
