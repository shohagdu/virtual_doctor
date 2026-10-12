# Virtual Doctor — AI-Based Doctor Prescription Software

React 19.2 • Rust 1.99 (Axum) • MySQL 9.7 LTS • Claude API

Specs live in [`Docs/`](Docs/README.md). This repo is at **Phase 0** (project setup) of
`Patient_Management_Spec.md` §14, plus a first slice of Phase 1: login, patient registration
(with duplicate check), patient search/list and a basic profile page.

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
| GET | `/api/v1/patients/{id}` | Profile |

## Production

```bash
cp .env.example .env   # set strong secrets
docker compose up -d --build
docker compose exec backend vd-api create-user admin@clinic.bd "Admin" admin <password>
```

Put TLS (e.g. Let's Encrypt) in front of the `nginx` service. MySQL is not published to the host.
Backups are written daily to `./backups`; copy them off-server, encrypted.

## Next (Phase 1 → 2)

Medical profile (allergies, chronic conditions), visits + prescription screen, queue dashboard,
vitals, history drawer, repeat ℞, allergy warnings, role checks per route.
