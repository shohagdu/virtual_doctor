-- Core tables: clinics, users, counters, audit log, background jobs.
-- Conventions (Patient_Management_Spec §9.1): InnoDB, utf8mb4_0900_ai_ci, UTC DATETIME,
-- BIGINT UNSIGNED keys, clinic_id on main tables, deleted_at for soft delete.

CREATE TABLE clinics (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name            VARCHAR(150) NOT NULL,
    address         VARCHAR(255) NULL,
    phone           VARCHAR(15) NULL,
    logo_path       VARCHAR(255) NULL,
    settings        JSON NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE users (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    clinic_id       BIGINT UNSIGNED NOT NULL,
    name            VARCHAR(150) NOT NULL,
    email           VARCHAR(190) NOT NULL,
    phone           VARCHAR(15) NULL,
    password_hash   VARCHAR(255) NOT NULL,
    role            ENUM('doctor','assistant','admin') NOT NULL,
    degrees         VARCHAR(255) NULL,
    bmdc_reg_no     VARCHAR(30) NULL,
    signature_path  VARCHAR(255) NULL,
    is_active       TINYINT(1) NOT NULL DEFAULT 1,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uq_users_email (email),
    CONSTRAINT fk_users_clinic FOREIGN KEY (clinic_id) REFERENCES clinics (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Per-clinic, per-year sequences (e.g. patient codes P-2026-000123)
CREATE TABLE clinic_counters (
    clinic_id       BIGINT UNSIGNED NOT NULL,
    name            VARCHAR(30) NOT NULL,
    year            SMALLINT UNSIGNED NOT NULL,
    value           BIGINT UNSIGNED NOT NULL DEFAULT 0,
    PRIMARY KEY (clinic_id, name, year),
    CONSTRAINT fk_counters_clinic FOREIGN KEY (clinic_id) REFERENCES clinics (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Append-only audit trail (PM-21)
CREATE TABLE audit_logs (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    clinic_id       BIGINT UNSIGNED NOT NULL,
    user_id         BIGINT UNSIGNED NULL,
    entity          VARCHAR(50) NOT NULL,
    entity_id       BIGINT UNSIGNED NULL,
    action          VARCHAR(30) NOT NULL,
    ip              VARCHAR(45) NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    KEY ix_audit_entity (entity, entity_id),
    KEY ix_audit_clinic_time (clinic_id, created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Background job queue (SMS reminders, AI batch jobs, retries)
CREATE TABLE jobs (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    type            VARCHAR(50) NOT NULL,
    payload         JSON NOT NULL,
    status          ENUM('pending','running','done','failed') NOT NULL DEFAULT 'pending',
    attempts        INT UNSIGNED NOT NULL DEFAULT 0,
    run_at          DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_error      TEXT NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    KEY ix_jobs_status_run (status, run_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
