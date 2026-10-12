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

## Next (Phase 3)

Appointments, follow-up list and reminders, SMS, billing (`Patient_Management_Spec.md` §14).
Still open from the roadmap: medicine brand database (the allergy check gets generics from
it), prescription templates.
