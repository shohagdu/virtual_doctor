-- Remaining spec / roadmap items: patient merge (PM-19), prescription templates and
-- favourite medicines (roadmap 2.1, 2.3), clinic medicine catalogue (2.2), and
-- certificates / letters (2.9).

-- A merged-away patient keeps its row (soft-deleted) and points at the one kept.
ALTER TABLE patients
    ADD COLUMN merged_into BIGINT UNSIGNED NULL AFTER deleted_at;

-- Catalogue rows with clinic_id NULL are shared; a clinic's imported rows are its own.
ALTER TABLE medicines
    ADD COLUMN clinic_id BIGINT UNSIGNED NULL AFTER id,
    ADD KEY ix_medicines_clinic_brand (clinic_id, brand),
    ADD KEY ix_medicines_generic (generic);

CREATE TABLE rx_templates (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    clinic_id       BIGINT UNSIGNED NOT NULL,
    name            VARCHAR(100) NOT NULL,
    body            JSON NOT NULL,
    created_by      BIGINT UNSIGNED NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uq_rx_templates_name (clinic_id, name),
    CONSTRAINT fk_rx_templates_clinic FOREIGN KEY (clinic_id) REFERENCES clinics (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Each doctor's favourite medicines with their usual dose.
CREATE TABLE medicine_favorites (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    user_id         BIGINT UNSIGNED NOT NULL,
    line            JSON NOT NULL,
    brand           VARCHAR(150) NOT NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    KEY ix_favorites_user (user_id, brand),
    CONSTRAINT fk_favorites_user FOREIGN KEY (user_id) REFERENCES users (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Medical certificates, sick leave and referral letters written for a visit.
CREATE TABLE documents (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    clinic_id       BIGINT UNSIGNED NOT NULL,
    patient_id      BIGINT UNSIGNED NOT NULL,
    visit_id        BIGINT UNSIGNED NOT NULL,
    kind            ENUM('sick_leave','fitness','referral') NOT NULL,
    body            JSON NOT NULL,
    created_by      BIGINT UNSIGNED NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    KEY ix_documents_visit (visit_id),
    KEY ix_documents_patient (patient_id, created_at),
    CONSTRAINT fk_documents_patient FOREIGN KEY (patient_id) REFERENCES patients (id),
    CONSTRAINT fk_documents_visit FOREIGN KEY (visit_id) REFERENCES visits (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
