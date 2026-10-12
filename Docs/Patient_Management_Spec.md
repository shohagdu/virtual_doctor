# Patient Management Module
### Feature Specification for Doctor Prescription Software

| Item | Detail |
|---|---|
| Document Type | Product & Functional Specification |
| Module | Patient Management |
| Version | 1.1 (Draft) — Tech stack added |
| Tech Stack | React 19.2 (frontend) • Rust 1.99 (backend) • MySQL 9.7 LTS (database) |
| Date | 12 Oct 2026 |
| Related Documents | AI_Investigation_Report_Analysis_Spec.md |
| AI Provider | Anthropic Claude API (via Anthropic Console) |
| Target Users | Doctor, Chamber Assistant, Admin |
| Language Support | English + Bangla (বাংলা) |

---

## 1. Purpose

Today the prescription screen shows one patient (Name, Age, Sex, Date) for one visit. The Patient Management module adds the layer **around** it:

- Register a patient once and reuse their details on every visit.
- Find any patient quickly (name, phone, patient ID).
- See the full history: previous visits, prescriptions, reports, diagnoses, allergies.
- Manage today's queue, appointments, and follow-ups (Next Visit).
- Start a new prescription or repeat an old one in one click.

The UI must follow the **same visual style** as the existing prescription screen: white cards, light-blue patient info bar, grey left panel with section headings and ➕ buttons, coloured action buttons (grey Settings, green PDF, blue Print), and red ✕ for removing items.

---

## 2. Where It Fits

```
                 ┌──────────────────────┐
                 │   Dashboard / Queue  │
                 └──────────┬───────────┘
                            │
          ┌─────────────────┼──────────────────┐
          ▼                 ▼                  ▼
  ┌───────────────┐ ┌───────────────┐ ┌─────────────────┐
  │ Patient List  │ │ Appointments  │ │ Follow-ups Due  │
  │ & Search      │ │               │ │ (Next Visit)    │
  └───────┬───────┘ └───────┬───────┘ └────────┬────────┘
          └─────────────────┼──────────────────┘
                            ▼
                 ┌──────────────────────┐
                 │   Patient Profile    │
                 │ (history, reports,   │
                 │  allergies, AI notes)│
                 └──────────┬───────────┘
                            ▼
                 ┌──────────────────────┐
                 │ Patient Prescription │  ← existing screen
                 │ (+ AI Report module) │
                 └──────────────────────┘
```

---

## 3. Scope

### 3.1 In Scope
- Patient registration and editing.
- Patient search and patient list.
- Patient profile with full medical history.
- Visit (encounter) management linked to prescriptions.
- Today's queue / waiting list.
- Appointments and follow-up reminders.
- Allergies, chronic conditions, and current medicines (persistent).
- Vitals record and trend charts.
- Report storage per patient (connects to AI Report Analysis module).
- AI patient summary using Claude API.
- Basic billing / consultation fee record.
- SMS reminders (Bangla/English).

### 3.2 Out of Scope (Version 1)
- Full hospital management (IPD wards, beds, pharmacy stock).
- Insurance claim processing.
- Patient-facing mobile app (planned later).

---

## 4. User Roles & Permissions

| Action | Doctor | Assistant | Admin |
|---|:---:|:---:|:---:|
| Register / edit patient demographics | ✔ | ✔ | ✔ |
| Add to queue / book appointment | ✔ | ✔ | ✔ |
| Record vitals (BP, Pulse, Temp, Weight) | ✔ | ✔ | — |
| Upload reports | ✔ | ✔ | — |
| View full medical history | ✔ | Limited | — |
| Write / edit prescription | ✔ | — | — |
| Run AI analysis / AI summary | ✔ | — | — |
| Collect fee / billing | ✔ | ✔ | ✔ |
| Delete / merge patient records | — | — | ✔ |
| Settings, users, clinic profile | ✔ | — | ✔ |

---

## 5. Screens

### 5.1 Dashboard (Home)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Dashboard                       [🔍 Search patient...]  [➕ New Patient]  │
├──────────────────────────────────────────────────────────────────────────┤
│ ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐              │
│ │ Today      │ │ Waiting    │ │ Seen       │ │ Follow-ups │              │
│ │ 32         │ │ 9          │ │ 23         │ │ Due: 6     │              │
│ └────────────┘ └────────────┘ └────────────┘ └────────────┘              │
├──────────────────────────────────────────────────────────────────────────┤
│ TODAY'S QUEUE                                                            │
│ #  Name          Age/Sex  Phone        Type       Status      Action     │
│ 1  Ms Maisha     18 / F   017xxxxxxxx  New        ✔ Seen      [Open]     │
│ 2  Mr Rahim      45 / M   018xxxxxxxx  Follow-up  ● Waiting   [Start ℞]  │
│ 3  Mrs Salma     32 / F   019xxxxxxxx  Report     ● Waiting   [Start ℞]  │
└──────────────────────────────────────────────────────────────────────────┘
```

**Features**
- Counter cards: Today's patients, Waiting, Seen, Follow-ups due.
- Queue status: Waiting → In Consultation → Seen (colour badges).
- Visit type: New, Follow-up, Report Showing.
- Drag to reorder queue; serial number auto-generated.
- "Start ℞" opens the existing Patient Prescription screen with the patient's details pre-filled.

### 5.2 Patient Registration (New Patient)

Opened from **➕ New Patient** as a modal or page.

| Field | Required | Notes |
|---|:---:|---|
| Full Name | ✔ | English; Bangla name optional |
| Title | — | Mr / Ms / Mrs / Baby / Master |
| Mobile Number | ✔ | Bangladesh format (01XXXXXXXXX); used for duplicate check and SMS |
| Age **or** Date of Birth | ✔ | If only age is given, store an estimated DOB so age updates automatically |
| Sex | ✔ | Male / Female / Other |
| Address / Area | — | Upazila / District |
| Blood Group | — | |
| Marital Status | — | |
| Pregnancy status | — | Shown for female patients; affects AI and drug warnings |
| Guardian name & phone | — | Required if patient is a minor |
| Referred by | — | Doctor / Self |
| NID / Birth Reg. No. | — | Optional |
| Known Allergies | — | Drug / food allergies |
| Chronic Conditions | — | DM, HTN, Asthma, CKD, etc. |
| Photo | — | Optional |

**Rules**
- **Patient ID** auto-generated, e.g., `P-2026-000123`.
- **Duplicate check** on mobile number + name; show "Possible existing patient" before saving.
- One mobile number can belong to several family members (show a family list).
- After saving: options **[Add to Queue]**, **[Start Prescription]**, **[Book Appointment]**.

### 5.3 Patient List & Search

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Patients            [🔍 Name / Phone / ID ]  Filter: [Sex▾][Age▾][Dx▾]   │
├──────────────────────────────────────────────────────────────────────────┤
│ ID            Name        Age/Sex  Phone        Last Visit   Last Dx     │
│ P-2026-000123 Ms Maisha   18 / F   017xxxxxxxx  07 Sep 2026  UTI    [›]  │
│ P-2026-000098 Mr Rahim    45 / M   018xxxxxxxx  02 Oct 2026  HTN    [›]  │
└──────────────────────────────────────────────────────────────────────────┘
```

- Instant search (results while typing) by name, phone, or patient ID; Bangla name search supported.
- Filters: sex, age range, diagnosis, last visit date, chronic condition, follow-up due.
- Sort by last visit, name, or registration date.
- Pagination / infinite scroll.
- Export list (CSV) — Admin only.

### 5.4 Patient Profile

Uses the **same layout** as the prescription screen: patient info bar on top, grey left panel, white main card on the right.

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Patient Profile                 [✏ Edit] [➕ New ℞] [📄 Upload Report]   │
├──────────────────────────────────────────────────────────────────────────┤
│ ID: P-2026-000123 │ Name: Ms Maisha │ Age: 18 Years │ Sex: Female │      │
│ Phone: 017xxxxxxxx │ Blood: B+ │ ⚠ Allergy: Penicillin                   │
├────────────────────────┬─────────────────────────────────────────────────┤
│ Allergies          ➕  │ [Visits] [Prescriptions] [Reports] [Vitals]     │
│ • Penicillin       ✕  │ [AI Summary] [Billing]                          │
│ Chronic Conditions ➕  │─────────────────────────────────────────────────│
│ • None                 │ VISIT TIMELINE                                  │
│ Current Medicines  ➕  │ 07 Sep 2026 — UTI                               │
│ • —                    │   Fever, burning micturition                    │
│ Past History       ➕  │   ℞ Furotil Plus, Visral 50, Napa Extra         │
│ Family History     ➕  │   Reports: CBC, Urine R/E, USG   [View ℞][Repeat]│
│ Social History     ➕  │ 14 Mar 2026 — Viral fever                       │
│ Gynae/Obs History  ➕  │   ℞ Napa                       [View ℞][Repeat]│
│ Vaccination        ➕  │                                                 │
└────────────────────────┴─────────────────────────────────────────────────┘
```

**Left panel (persistent patient data, carried into every new prescription):**
Allergies, Chronic Conditions, Current/Long-term Medicines, Past History, Family History, Social History (smoking, etc.), Gynae/Obs History (female), Vaccination (children).

**Tabs (right panel):**

| Tab | Content |
|---|---|
| Visits | Timeline of all visits: date, complaints, diagnosis, medicines, reports; actions: View ℞, Repeat ℞, Print |
| Prescriptions | All prescriptions as list; open, print, download PDF, compare two |
| Reports | All uploaded reports by date and type; AI analysis results; trend link |
| Vitals | BP, Pulse, Temp, Weight, Height, BMI, SpO₂, RBS — table + trend chart |
| AI Summary | Claude-generated summary of the patient's full history (see section 7) |
| Billing | Consultation fees, payments, dues |

### 5.5 Changes to the Existing Prescription Screen

| Area | Change |
|---|---|
| Patient info bar | Add **Patient ID** and **Phone**; click name → opens Patient Profile |
| Patient info bar | Show **⚠ Allergy** badge in red if any allergy exists |
| Patient info bar | Add **Visit No.** (e.g., "Visit 3") and **Last visit** date |
| Top buttons | Add **[🕘 History]** (slide-in drawer of previous visits) and **[📄 Upload Report]** |
| Past History / Drug History | Auto-filled from the patient profile; editable |
| ℞ panel | **Repeat last prescription** button; warn if a medicine matches an allergy |
| Next Visit | Saving the date automatically creates a follow-up reminder |
| Save | **[💾 Save]** / **[Save & Next Patient]** — marks queue status as Seen and opens the next waiting patient |

**History drawer (inside prescription screen):**

```
┌───────────────────────────────┐
│ 🕘 Previous Visits       [✕]  │
├───────────────────────────────┤
│ 14 Mar 2026 — Viral fever     │
│  ℞ Napa 500 1+1+1 — 3 দিন     │
│  [Copy medicines] [View]      │
├───────────────────────────────┤
│ Reports: CBC (12 Mar)  [View] │
└───────────────────────────────┘
```

### 5.6 Appointments

- Calendar view (day / week) and list view.
- Book by patient search or new registration.
- Time slots based on doctor's chamber schedule (set in Settings).
- Status: Booked → Arrived (moves to queue) → Seen / No-show / Cancelled.
- SMS confirmation and reminder (one day before).

### 5.7 Follow-up Management

- Every **Next Visit** date in a prescription creates a follow-up record.
- "Follow-ups Due" list: Today / Overdue / This week.
- Actions: Call, send SMS reminder, mark as Came / Missed.
- Overdue chronic patients (e.g., diabetes, hypertension) highlighted.

### 5.8 Billing (Basic)

| Field | Notes |
|---|---|
| Visit type & fee | New / Follow-up / Report showing (fees set in Settings) |
| Discount | Amount or % with reason |
| Payment method | Cash, bKash, Nagad, Card |
| Status | Paid / Due |
| Receipt | Print / PDF in English or Bangla |

Daily collection report for the doctor/admin.

---

## 6. Functional Requirements

| ID | Requirement |
|---|---|
| PM-01 | Register a patient with auto-generated unique Patient ID. |
| PM-02 | Detect duplicates by phone + name before saving. |
| PM-03 | Allow multiple patients under one phone number (family). |
| PM-04 | Auto-calculate age from DOB on every visit. |
| PM-05 | Search patients by name, phone, or ID in under 1 second. |
| PM-06 | Create a visit record each time a prescription is started. |
| PM-07 | Pre-fill prescription patient bar from the patient record. |
| PM-08 | Carry allergies, chronic conditions, and long-term medicines into each new prescription. |
| PM-09 | Show allergy warning when a matching medicine/generic is added to ℞. |
| PM-10 | Repeat a previous prescription (all or selected medicines). |
| PM-11 | Store every printed/saved prescription as a locked version; later edits create a new version. |
| PM-12 | Link uploaded reports to both patient and visit. |
| PM-13 | Record vitals per visit and show trend charts. |
| PM-14 | Manage today's queue with status and serial numbers. |
| PM-15 | Book, reschedule, and cancel appointments. |
| PM-16 | Create follow-up reminders from Next Visit date. |
| PM-17 | Send SMS reminders (configurable templates, Bangla/English). |
| PM-18 | Record consultation fees and payments. |
| PM-19 | Merge duplicate patient records (Admin), keeping all history. |
| PM-20 | Generate AI patient summary via Claude API on doctor request. |
| PM-21 | All patient data changes logged in audit trail. |

---

## 7. AI Features (Claude API)

Uses the same Claude API setup described in *AI_Investigation_Report_Analysis_Spec.md* (backend-only API key, Anthropic Console workspaces, de-identification).

### 7.1 AI Patient Summary

**Trigger:** "AI Summary" tab or **[🤖 Summarise History]** button in the prescription screen.

**Input sent to Claude (de-identified):** age, sex, allergies, chronic conditions, last N visits (complaints, diagnosis, medicines), key lab values with dates, vitals trends.

**Output (structured JSON → shown as card):**

| Section | Example |
|---|---|
| One-line summary | 18-year-old female; one recent episode of UTI (Sep 2026); otherwise healthy. |
| Active problems | UTI (Sep 2026) — review pending |
| Recurring issues | Fever episodes: Mar 2026, Sep 2026 |
| Important lab trends | Hb 11.8 g/dL (Sep 2026) — mildly low |
| Medicines history | Antibiotics used: … ; responses noted |
| Alerts | Penicillin allergy |
| Points to check today | Repeat Urine R/E after treatment; evaluate mild anaemia |

**Model:** `claude-sonnet-5-5` (configurable). For very long histories, only the latest visits plus a stored previous summary are sent to save tokens.

### 7.2 Smart Patient Search (Optional)

Natural language search for doctors, e.g., *"female patients with UTI in last 3 months"*.
`claude-haiku-5-5` converts the sentence into **search filters** (sex, diagnosis, date range); the backend runs the database query. Patient data itself is not sent to Claude for this feature.

### 7.3 Follow-up Message Drafting (Optional)

`claude-haiku-5-5` drafts a short, polite Bangla/English SMS reminder from a template; the assistant reviews before sending. No diagnosis is included in SMS text.

### 7.4 AI Rules

- AI output is advisory; the doctor confirms anything copied into a prescription.
- Every AI summary shows date generated, model name, and prompt version.
- Patient name, phone, NID, and address are **never** sent to the Claude API.
- AI features can be turned off per clinic in Settings.

---

## 8. Technology Stack & Architecture

### 8.1 Technology Stack

| Layer | Technology | Version (as of Oct 2026) | Purpose |
|---|---|---|---|
| Frontend | **React** | 19.2.x (latest stable line) | User interface |
| Frontend language | TypeScript | Latest stable | Type safety for forms, API data |
| Build tool | Vite | Latest stable | Fast dev server and production build (single-page app) |
| Backend language | **Rust** | 1.99.0 stable, Edition 2024 | API server, business rules, AI orchestration |
| Web framework | Axum (on Tokio async runtime) | Latest stable | REST API, file upload, streaming (SSE) |
| Database | **MySQL** | **9.7 LTS** (recommended) | Main data store |
| AI | Claude API (Anthropic Console) | — | Report analysis, patient summary |
| Web server / proxy | Nginx | Latest stable | HTTPS, static files, reverse proxy |
| OS / Deployment | Ubuntu Server LTS + Docker | Latest LTS | Hosting |

**Version policy**
- "Latest" means the latest **stable** release at project start; exact versions are **pinned** (`package-lock.json`, `Cargo.lock`, Docker image tags) and upgraded on a planned schedule, never automatically in production.
- **React:** stay on the 19.x line; minor/patch updates every quarter after testing.
- **Rust:** new stable every 6 weeks; pin the toolchain with a `rust-toolchain.toml` file and upgrade every 1–2 releases.
- **MySQL:** use **9.7 LTS** for production (long support period, no feature removals inside the LTS line). MySQL's quarterly **Innovation** releases (now calendar-versioned, e.g. 26.7) are supported only until the next Innovation release, so they are not recommended for medical production data. Use them only in a test environment if a new feature is needed.

### 8.2 System Architecture

```
┌─────────────────────────────┐
│  React 19 SPA (Browser /    │
│  Tablet / Mobile browser)   │
│  • Prescription screen      │
│  • Patient management       │
│  • AI panels (streamed)     │
└──────────────┬──────────────┘
               │ HTTPS (JSON REST + SSE)
               ▼
┌─────────────────────────────┐
│ Nginx (TLS, gzip, static)   │
└──────────────┬──────────────┘
               ▼
┌─────────────────────────────────────────────────────────┐
│ Rust Backend (Axum + Tokio)                             │
│  • Auth & roles (JWT / session)                         │
│  • Patient, Visit, Prescription, Report, Billing APIs   │
│  • Rule engine (reference ranges, allergy check)        │
│  • AI service → Claude API (HTTPS, API key on server)   │
│  • Background jobs (SMS, AI batch, cleanup)             │
│  • Audit logger                                         │
└───────┬───────────────────┬───────────────────┬─────────┘
        ▼                   ▼                   ▼
┌───────────────┐   ┌───────────────┐   ┌────────────────┐
│ MySQL 9.7 LTS │   │ File Storage  │   │ External APIs  │
│ (InnoDB,      │   │ (reports, PDFs│   │ • Claude API   │
│  utf8mb4)     │   │  photos)      │   │ • SMS gateway  │
└───────────────┘   └───────────────┘   └────────────────┘
```

### 8.3 Frontend (React)

| Concern | Choice | Notes |
|---|---|---|
| App type | Single-page app built with Vite | No server-side rendering needed (logged-in clinic app) |
| Routing | React Router | `/dashboard`, `/patients`, `/patients/:id`, `/visits/:id/prescription`, `/appointments`, `/followups`, `/settings` |
| Server data | TanStack Query | Caching, background refresh, retries |
| Local UI state | Zustand (or React context) | Queue, open drawers, current visit |
| Forms & validation | React Hook Form + Zod | Registration, vitals, billing |
| Styling | Tailwind CSS | Match existing prescription look (light-blue bar, grey left panel, white cards) |
| Charts | Recharts | Vitals and lab trends |
| Language | react-i18next | English / Bangla UI text; Bangla font (e.g., Noto Sans Bengali / Hind Siliguri) |
| Dates | date-fns | Asia/Dhaka display |
| Print / PDF | Browser print CSS + server PDF | Keep current print layout |
| Offline | Service worker + IndexedDB | Queue, registration, draft prescriptions; sync later |
| AI streaming | `fetch` stream / EventSource (SSE) | Shows AI summary as it is generated |

**Folder structure (proposed):**

```
frontend/
 └─ src/
    ├─ app/            (router, providers, layout)
    ├─ features/
    │   ├─ dashboard/
    │   ├─ patients/       (list, registration, profile)
    │   ├─ prescription/   (existing screen + history drawer)
    │   ├─ reports/        (upload, AI analysis panel)
    │   ├─ appointments/
    │   ├─ followups/
    │   ├─ billing/
    │   └─ settings/
    ├─ components/     (shared UI: InfoBar, SectionPanel, ➕ / ✕ items)
    ├─ api/            (typed API client)
    ├─ i18n/           (en.json, bn.json)
    └─ utils/
```

React 19 features to use: **Actions / `useActionState`** for form submissions, **`useOptimistic`** for quick add/remove of ➕/✕ items, and the **React Compiler** (optional) to reduce manual memoisation.

### 8.4 Backend (Rust)

| Concern | Crate / Approach | Notes |
|---|---|---|
| HTTP server | `axum` + `tokio` | Async, fast, low memory |
| Middleware | `tower-http` | CORS, compression, request ID, timeouts, body size limit |
| Database | `sqlx` (MySQL driver) | Async, compile-time checked SQL, built-in migrations |
| JSON | `serde`, `serde_json` | API and AI JSON |
| Validation | `validator` (or custom) | Phone, age, required fields |
| Auth | `argon2` (password hash) + `jsonwebtoken` (JWT) | Access + refresh tokens; role checks per route |
| HTTP client | `reqwest` | Claude API and SMS gateway calls |
| Logging | `tracing` + `tracing-subscriber` | Structured logs; no patient data in logs |
| Errors | `thiserror` / `anyhow` | Consistent JSON error responses |
| Config | `config` / env variables | DB URL, Claude API key, SMS keys from environment / secret store |
| API docs | `utoipa` (OpenAPI) | Auto-generated API documentation for the frontend team |
| Images | `image` crate | Resize/compress report photos before AI |
| Background jobs | Tokio tasks + MySQL `jobs` table | SMS reminders, AI batch jobs, retries |
| Testing | `cargo test` + test MySQL in Docker | Unit + integration tests |

All crates use their latest stable versions at project start, pinned in `Cargo.lock`.

**Project structure (proposed, Cargo workspace):**

```
backend/
 ├─ Cargo.toml          (workspace)
 ├─ rust-toolchain.toml (pins Rust 1.99.0)
 ├─ migrations/         (sqlx SQL migrations)
 └─ crates/
    ├─ api/       (Axum routes, handlers, middleware)
    ├─ domain/    (patients, visits, prescriptions, rules)
    ├─ db/        (sqlx repositories)
    ├─ ai/        (Claude API client, prompts, schemas)
    ├─ jobs/      (background workers)
    └─ common/    (config, errors, auth)
```

### 8.5 REST API (Main Endpoints)

Base path: `/api/v1` — JSON, authenticated with Bearer token.

| Method | Endpoint | Purpose |
|---|---|---|
| POST | `/auth/login` | Login |
| POST | `/auth/refresh` | Refresh token |
| GET | `/patients?q=&sex=&dx=&page=` | Search / list patients |
| POST | `/patients` | Register patient (returns duplicates warning if any) |
| GET | `/patients/{id}` | Patient profile |
| PATCH | `/patients/{id}` | Edit patient |
| GET / PUT | `/patients/{id}/medical-profile` | Allergies, chronic conditions, histories |
| GET | `/patients/{id}/visits` | Visit timeline |
| POST | `/patients/{id}/visits` | Start new visit |
| GET / PUT | `/visits/{id}/prescription` | Load / save prescription |
| POST | `/visits/{id}/prescription/lock` | Finalise (print/PDF) and lock version |
| POST | `/visits/{id}/prescription/repeat/{prev_id}` | Repeat previous prescription |
| POST | `/visits/{id}/vitals` | Save vitals |
| GET | `/patients/{id}/vitals/trend` | Vitals trend data |
| GET / POST | `/queue/today` | Today's queue / add to queue |
| PATCH | `/queue/{id}` | Change status / order |
| GET / POST / PATCH | `/appointments` | Appointments |
| GET / PATCH | `/followups` | Follow-ups due / update status |
| POST | `/visits/{id}/bill` | Billing |
| POST | `/patients/{id}/ai-summary` | Generate AI summary (SSE stream) |
| GET | `/patients/{id}/ai-summary/latest` | Last saved summary |
| POST | `/admin/patients/merge` | Merge duplicates (Admin) |
| GET | `/medicines/search?q=` | Brand / generic search for ℞ |

Report upload and AI analysis endpoints are listed in *AI_Investigation_Report_Analysis_Spec.md*.

### 8.6 Deployment

```
Ubuntu Server (VPS or clinic server)
 └─ Docker Compose
     ├─ nginx      (TLS via Let's Encrypt, serves React build)
     ├─ backend    (Rust release binary, small image)
     ├─ mysql      (MySQL 9.7 LTS, data on persistent volume)
     └─ backup     (scheduled dump job)
```

- Separate **dev / staging / production** environments (match Anthropic Console workspaces).
- CI pipeline: `cargo fmt`, `cargo clippy`, `cargo test`, frontend lint + tests, then build Docker images.
- Database migrations run automatically on deploy (sqlx migrations), with a backup taken first.

---

## 9. Database Design (MySQL 9.7 LTS)

### 9.1 General Rules

| Rule | Setting |
|---|---|
| Engine | InnoDB for all tables |
| Character set | `utf8mb4` with `utf8mb4_0900_ai_ci` collation (full Bangla + emoji support) |
| Primary keys | `BIGINT UNSIGNED AUTO_INCREMENT` internally; public codes (e.g., `P-2026-000123`) shown in UI |
| Time | Store `DATETIME` in UTC; display in Asia/Dhaka |
| Flexible data | `JSON` columns for AI output and dynamic lists |
| Soft delete | `deleted_at` column instead of hard delete for clinical records |
| Multi-clinic | Every main table has `clinic_id` (ready for SaaS) |
| Migrations | Versioned SQL files managed by `sqlx` |

### 9.2 Main Tables

**patients**

| Column | Type | Notes |
|---|---|---|
| id | BIGINT UNSIGNED PK | |
| clinic_id | BIGINT UNSIGNED | FK clinics |
| patient_code | VARCHAR(20) UNIQUE | P-2026-000123 |
| title | VARCHAR(10) | Mr / Ms / Mrs / Baby |
| name_en | VARCHAR(150) | Indexed |
| name_bn | VARCHAR(150) NULL | |
| phone | VARCHAR(15) | Indexed (family share allowed) |
| dob | DATE | |
| dob_estimated | TINYINT(1) | 1 if only age was given |
| sex | ENUM('M','F','O') | |
| blood_group | VARCHAR(5) NULL | |
| address, district | VARCHAR | |
| nid | VARCHAR(20) NULL | Encrypted at application level |
| guardian_name, guardian_phone | VARCHAR NULL | |
| referred_by | VARCHAR(150) NULL | |
| photo_path | VARCHAR(255) NULL | File storage path |
| created_at, updated_at, deleted_at | DATETIME | |

**patient_medical_profile** — `patient_id` (PK/FK), `allergies` JSON, `chronic_conditions` JSON, `long_term_medicines` JSON, `past_history` TEXT, `family_history` TEXT, `social_history` TEXT, `gynae_obs_history` TEXT, `vaccination` JSON, `updated_at`.

**visits** — `id`, `clinic_id`, `patient_id`, `visit_no`, `visit_date`, `visit_type` ENUM('new','followup','report'), `queue_serial`, `status` ENUM('waiting','in_consultation','seen','cancelled'), `doctor_id`, timestamps.

**vitals** — `id`, `visit_id`, `bp_sys`, `bp_dia`, `pulse`, `temp` DECIMAL(4,1), `weight` DECIMAL(5,2), `height` DECIMAL(5,2), `bmi` DECIMAL(4,1), `spo2`, `rbs` DECIMAL(5,2), `recorded_by`, `created_at`.

**prescriptions** — `id`, `visit_id`, `version`, `complaints` JSON, `examination` JSON, `investigations` JSON, `diagnosis` JSON, `treatment_plan` JSON, `advice` JSON, `next_plan` JSON, `operation_notes` TEXT, `next_visit_date` DATE, `locked` TINYINT(1), `pdf_path`, `created_by`, timestamps.
*(Sections are JSON arrays because each matches a ➕/✕ list on the screen.)*

**prescription_medicines** — `id`, `prescription_id`, `sort_order`, `medicine_id` NULL, `brand`, `generic`, `strength`, `form`, `dose_pattern` (e.g., 1+0+1), `timing` (খাবারের পরে), `duration` (5 দিন), `notes`.

**medicines** (master list) — `id`, `brand`, `generic`, `strength`, `form`, `company`, `is_active`; FULLTEXT index on brand + generic.

**appointments** — `id`, `clinic_id`, `patient_id`, `start_at`, `status`, `source`, `sms_sent_at`.

**followups** — `id`, `patient_id`, `prescription_id`, `due_date`, `status` ENUM('pending','came','missed'), `reminder_sent_at`.

**bills** — `id`, `visit_id`, `fee`, `discount`, `paid` DECIMAL(10,2), `method` ENUM('cash','bkash','nagad','card'), `status`, `created_by`, `created_at`.

**ai_summaries** — `id`, `patient_id`, `content` JSON, `claude_model` VARCHAR(50), `prompt_version` VARCHAR(20), `input_tokens`, `output_tokens`, `created_by`, `created_at`.

**users** — `id`, `clinic_id`, `name`, `email`, `phone`, `password_hash`, `role` ENUM('doctor','assistant','admin'), `bmdc_reg_no`, `signature_path`, `is_active`.

**audit_logs** — `id`, `user_id`, `entity`, `entity_id`, `action`, `ip`, `created_at` (append-only).

**jobs** — `id`, `type`, `payload` JSON, `status`, `attempts`, `run_at`, `last_error` (background queue).

### 9.3 Key Indexes

| Table | Index | Purpose |
|---|---|---|
| patients | (clinic_id, phone) | Duplicate check, phone search |
| patients | (clinic_id, name_en) | Name search (prefix) |
| patients | FULLTEXT (name_en, name_bn) | Partial name search |
| visits | (patient_id, visit_date) | Timeline |
| visits | (clinic_id, visit_date, status) | Today's queue |
| followups | (status, due_date) | Follow-ups due list |
| appointments | (clinic_id, start_at) | Calendar |

### 9.4 Backup & Security (Database)

- Daily full logical backup (MySQL Shell dump utility or `mysqldump`) + **binary logs** for point-in-time recovery.
- Backups encrypted and copied off-server; restore tested monthly.
- InnoDB data-at-rest encryption enabled (MySQL keyring component).
- Separate DB users: application user (limited rights), migration user, read-only reporting user.
- MySQL not exposed to the internet; only the backend container can reach it.

---

## 10. Non-Functional Requirements

| Category | Requirement |
|---|---|
| Performance | Patient search < 1 s; profile load < 2 s with 100+ visits |
| Scale | 100,000+ patients per clinic without slowdown |
| Availability | 99.5% uptime; daily automatic backup |
| Offline | Queue, registration, and prescription writing work offline; sync when online |
| Usability | Register + start prescription in under 60 seconds |
| Devices | Desktop, tablet, mobile browser; print layout unchanged |
| Language | English/Bangla UI toggle; Bangla number and date formats optional |

---

## 11. Security & Privacy

- Login with password + optional OTP; auto logout after inactivity.
- Role-based access (section 4).
- Encryption in transit (TLS) and at rest.
- Locked prescriptions cannot be silently changed; all edits versioned.
- Audit log for view / edit / print / delete / AI calls.
- Patient consent record for SMS and AI-assisted features.
- Data export and deletion on patient request (Admin).
- Follow applicable Bangladesh data-protection and BMDC guidance; review with a legal advisor.

---

## 12. Settings Additions

- Clinic profile: name, address, logo, chamber timings.
- Doctor profile: name, degrees, BMDC reg. no., signature (used on prescription print).
- Patient ID format and starting number.
- Visit types and fees.
- Appointment slot length and daily limit.
- SMS gateway, sender name, templates.
- Required registration fields.
- AI features on/off; output language.

---

## 13. Error & Edge Cases

| Situation | Behaviour |
|---|---|
| Duplicate patient found | Show matches; choose "Use existing" or "Create new" |
| Age only, no DOB | Store estimated DOB; mark as estimated |
| Patient record opened by two users | Last-save warning; prevent overwrite of prescriptions |
| Allergy matches a prescribed generic | Red warning; doctor must confirm to continue |
| SMS failed | Retry; show failed status in follow-up list |
| Offline save | Saved locally with "Pending sync" badge |
| Merge patients | Admin preview of both histories before merge; merge is logged |

---

## 14. Development Phases

| Phase | Deliverables | Duration (Est.) |
|---|---|---|
| Phase 0 | Project setup: React + Vite app, Rust/Axum workspace, MySQL 9.7 schema & migrations, CI/CD, Docker | 2 weeks |
| Phase 1 | Registration, patient list/search, profile, visit linking with prescription screen | 4–5 weeks |
| Phase 2 | Queue dashboard, vitals, history drawer, repeat prescription, allergy warnings | 3–4 weeks |
| Phase 3 | Appointments, follow-ups, SMS, billing | 3–4 weeks |
| Phase 4 | AI patient summary, smart search, report module integration | 3–4 weeks |

---

## 15. Success Metrics

- New patient registration + prescription start under 60 seconds.
- Duplicate patient records < 1%.
- Follow-up return rate improved after SMS reminders.
- ≥ 80% of doctors use History / Repeat ℞ weekly.
- AI summary rated useful by doctors ≥ 4/5.

---

*End of Document*
