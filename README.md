# Virtual Doctor — AI-Based Doctor Prescription Software

React 19.2 • Rust 1.99 (Axum) • MySQL 9.7 LTS • Claude API

Specs live in [`Docs/`](Docs/README.md). Done so far (`Patient_Management_Spec.md` §14):

- **Phase 0** — project setup, full schema, Docker, CI.
- **Phase 1** — login with roles, patient registration (duplicate check) / edit / search,
  patient profile with medical profile (allergies, chronic conditions, histories) and visit
  timeline, visits linked to the prescription screen, versioned prescriptions (locked on
  print; edits create a new version; concurrent-edit protection), print / PDF pad with doctor
  and clinic header, follow-up record from Next Visit, basic allergy warning, Settings for
  doctor and clinic details.
- **Phase 2** — today's queue dashboard (counters, serials, add-to-queue search, status
  changes, drag / ▲▼ reorder, auto-refresh), vitals per visit with BMI, trend chart and
  table, vitals on the ℞ screen and print, 🕘 history drawer with copy (all or selected)
  medicines, Repeat ℞ (profile timeline and "Repeat last ℞"), Save & Next Patient, and
  allergy warnings by name **and drug class** (penicillins, cephalosporins, sulfonamides,
  NSAIDs, macrolides, quinolones, tetracyclines, opioids), enforced on save.
- **Phase 3** — appointments (day / week / list, slots from the chamber schedule, daily
  limit, book / reschedule / no-show / cancel / restore, Arrived → today's queue), follow-ups
  due (Today / Overdue / This week / Done, call, SMS, came / missed, overdue chronic patients
  highlighted, auto "came" when the patient is queued), SMS (Bangla / English templates,
  booking confirmation, day-before reminder, follow-up reminder, sent / failed with retry,
  sent by the background worker), billing (fees per visit type, discount by amount or % with
  reason, cash / bKash / Nagad / card, paid / due, receipt in English or Bangla) and the
  collection report. Settings for fees, chamber schedule, slots and SMS.
- **Phase 4** — AI (Claude API, backend only): report upload (PDF / photo, camera, drag and
  drop; photos shrunk in the browser; duplicates detected by hash), AI extraction of values,
  a deterministic rule engine for Normal / Borderline / High / Low / Critical (lab range,
  else seeded age/sex system ranges; critical alert banner; name-mismatch warning), doctor
  verification table, streamed clinical analysis with tick / edit / reject suggestions
  inserted into Diagnosis / Investigations / Advice (English + Bangla) / Next plan, report
  short form for Investigations, prescription safety check (built-in rules + AI, dismiss with
  reason), lab trends, Reports and AI Summary tabs, AI patient summary, AI smart search,
  AI SMS wording, AI usage report, AI settings. Every call is logged with model, prompt
  version and tokens; prompts are versioned in `prompt_versions`.

## Layout

```
backend/              Rust Cargo workspace (toolchain pinned in rust-toolchain.toml)
  migrations/         sqlx migrations: the full schema from both specs
  crates/api          Axum server (routes, auth extractor) — binary `vd-api`
  crates/domain       business rules (patient validation, IDs)
  crates/db           sqlx MySQL repositories
  crates/ai           Claude Messages API client
  crates/jobs         background worker over the `jobs` table
  crates/common       config, errors, JWT/argon2
frontend/             React + TypeScript + Vite SPA (Tailwind, TanStack Query, i18n en/bn)
deploy/               MySQL config, backup script
docker-compose.yml    production stack: nginx + backend + mysql + backup
docker-compose.dev.yml  MySQL only, for local development
```

## Local development

Requirements: Docker, Rust (rustup installs 1.99.0 automatically), Node 20.19+ or 22.

```bash
# 1. Database (MySQL 9.7 on port 3307)
docker compose -f docker-compose.dev.yml up -d

# 2. Backend — runs migrations on start
cd backend
cp .env.example .env
cargo run -p vd-api -- create-user doctor@example.com "Dr Demo" doctor <password>
cargo run -p vd-api            # http://127.0.0.1:8080, health at /health

# 3. Frontend — proxies /api to :8080
cd frontend
npm install
npm run dev                    # http://localhost:5173
```

Checks: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` and
`npm run lint && npm run build` (the same checks run in CI).

## API (so far)

| Method | Endpoint | Notes |
|---|---|---|
| GET | `/health` | DB connectivity |
| POST | `/api/v1/auth/login` | `{email, password}` → JWT |
| GET | `/api/v1/patients?q=&page=&per_page=` | Search by name (en/bn), phone prefix, patient code |
| POST | `/api/v1/patients[?force=true]` | Returns `duplicates` instead of saving unless `force` |
| GET / PATCH | `/api/v1/patients/{id}` | Profile / edit demographics |
| GET / PUT | `/api/v1/patients/{id}/medical-profile` | Doctor, Assistant |
| GET / POST | `/api/v1/patients/{id}/visits` | Timeline (clinical details for Doctor only) / start visit |
| GET | `/api/v1/visits/{id}` | Visit + patient + medical profile for the ℞ screen |
| GET / PUT | `/api/v1/visits/{id}/prescription` | Doctor; PUT sends `base: {id, revision}` from the last load, 409 if stale |
| POST | `/api/v1/visits/{id}/prescription/lock` | Doctor; finalise on print, marks visit seen |
| GET | `/api/v1/queue/today` | Today's queue + counters (all roles) |
| POST | `/api/v1/queue/today` | `{patient_id, visit_type}`; 409 if already queued today |
| PATCH | `/api/v1/queue/{visit_id}` | `{status?, position?}`; In consultation / Seen are Doctor only |
| POST | `/api/v1/queue/next` | Doctor; `{current_visit_id}` → marks it seen, calls the next waiting patient |
| GET / PUT | `/api/v1/visits/{id}/vitals` | Doctor, Assistant; BMI calculated on the server |
| GET | `/api/v1/patients/{id}/vitals/trend` | Doctor, Assistant |
| POST | `/api/v1/visits/{id}/allergy-check` | Doctor; `{medicines}` → matches. ℞ PUT needs `allergy_ack: true` when any match |
| GET / PUT | `/api/v1/settings` | Fees, chamber schedule, slots, SMS; PUT Doctor, Admin |
| GET | `/api/v1/appointments?from=&to=` | Clinic-local dates, up to 62 days |
| POST | `/api/v1/appointments` | `{patient_id, date, time, visit_type, notes}`; 409 if the slot / day / patient is taken |
| GET | `/api/v1/appointments/slots?date=` | Slots from the chamber schedule, with taken / past |
| PATCH | `/api/v1/appointments/{id}` | `{status}` (arrived → queue, no_show, cancelled, booked) or `{date, time}` to reschedule |
| GET | `/api/v1/patients/{id}/appointments` | From today on |
| GET | `/api/v1/followups?bucket=today\|overdue\|week\|done` | With counts; diagnosis for Doctor only |
| PATCH | `/api/v1/followups/{id}` | `{status: came\|missed\|pending}` |
| GET / POST | `/api/v1/followups/{id}/sms` | Draft from the template / send `{body}` |
| POST | `/api/v1/sms/{id}/retry` | Re-send a failed SMS |
| GET / PUT | `/api/v1/visits/{id}/bill` | Bill + suggested fee / save `{fee, discount, discount_reason, paid, method}` |
| GET | `/api/v1/reports/collection?from=&to=` | Doctor, Admin; totals by method + rows |
| POST | `/api/v1/visits/{id}/reports` | Raw file body + `X-File-Name`; Doctor, Assistant; doctor uploads are read by AI |
| GET | `/api/v1/visits/{id}/reports`, `/api/v1/patients/{id}/reports` | Values for Doctor only |
| GET / DELETE | `/api/v1/reports/{id}` · GET `/reports/{id}/file` | Report / soft delete · original file |
| POST | `/api/v1/reports/{id}/extract` | Doctor; (re)read with AI |
| PUT | `/api/v1/reports/{id}/results` | Doctor; checked or manually entered values, re-graded |
| POST | `/api/v1/visits/{id}/ai-analysis` | Doctor; SSE: `progress`, then `done` (analysis + suggestions) or `error` |
| GET | `/api/v1/visits/{id}/ai-analysis/latest` | Doctor |
| POST | `/api/v1/ai-suggestions/{id}/action` | `{action: accepted\|edited\|rejected, edited_text?, reason?}` |
| POST | `/api/v1/visits/{id}/prescription/safety-check` | Doctor; `{medicines}` → warnings (rules + AI) |
| GET | `/api/v1/patients/{id}/lab-trends?test=` | Doctor |
| POST · GET | `/api/v1/patients/{id}/ai-summary` (SSE) · `/ai-summary/latest` | Doctor |
| GET | `/api/v1/patients/smart-search?q=` | Doctor; returns the filters understood + patients |
| POST | `/api/v1/followups/{id}/sms/ai-draft` | Reworded reminder (name filled in locally) |
| GET | `/api/v1/ai/status`, `/api/v1/admin/ai-usage?from=&to=` | AI availability · usage (Doctor, Admin) |
| GET | `/api/v1/medicines/search?q=` | Catalogue + medicines this clinic prescribed before |
| GET / PATCH | `/api/v1/me`, PATCH `/api/v1/clinic` | Doctor/clinic details for the print header |

## Production

```bash
cp .env.example .env   # set strong secrets
docker compose up -d --build
docker compose exec backend vd-api create-user admin@clinic.bd "Admin" admin <password>
```

Put TLS (e.g. Let's Encrypt) in front of the `nginx` service. MySQL is not published to the host.
Backups are written daily to `./backups`; copy them off-server, encrypted.

## SMS

Set `SMS_PROVIDER` in `.env`: `log` (default; nothing is sent, sends are only logged) or
`bulksmsbd` with `SMS_API_KEY` and `SMS_SENDER_ID`. Then turn SMS on in Settings. Messages
are queued and sent by the background worker, with retries; failures show in Follow-ups.

## AI

Set `ANTHROPIC_API_KEY` in `.env` (model names: `CLAUDE_MODEL_DEFAULT`, `_COMPLEX`, `_FAST`).
Without a key the app runs in manual mode: reports can still be uploaded and values typed in,
and the built-in safety rules still run. AI can also be switched off per clinic in Settings.

- Claude reads; the Rust rule engine decides the flags, so they are reproducible.
- Patient name, phone, ID and address are never sent; only age and sex — plus, for report
  reading, the report file itself (which may show the name printed by the lab).
- The seeded reference ranges are common adult textbook values (children: critical limits
  only). Review them before clinical use; clinic-specific rows go in `reference_ranges`.
- Prompts are drafts to be validated against real de-identified reports (AI spec §17) before
  doctors rely on them. HEIC photos are not converted yet (the upload asks for JPEG).
