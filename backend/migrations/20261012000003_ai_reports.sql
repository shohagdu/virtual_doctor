-- AI Investigation Report Analysis module (AI_Investigation_Report_Analysis_Spec §10)

CREATE TABLE reports (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    clinic_id       BIGINT UNSIGNED NOT NULL,
    patient_id      BIGINT UNSIGNED NOT NULL,
    visit_id        BIGINT UNSIGNED NULL,
    report_type     VARCHAR(50) NULL,
    lab_name        VARCHAR(150) NULL,
    report_date     DATE NULL,
    file_path       VARCHAR(255) NOT NULL,
    file_hash       CHAR(64) NOT NULL,
    mime_type       VARCHAR(50) NOT NULL,
    status          ENUM('uploaded','extracting','extracted','verified','failed') NOT NULL DEFAULT 'uploaded',
    impression      TEXT NULL,
    extraction_raw  JSON NULL,
    uploaded_by     BIGINT UNSIGNED NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at      DATETIME NULL,
    KEY ix_reports_hash (clinic_id, file_hash),
    KEY ix_reports_patient (patient_id, report_date),
    CONSTRAINT fk_reports_patient FOREIGN KEY (patient_id) REFERENCES patients (id),
    CONSTRAINT fk_reports_visit FOREIGN KEY (visit_id) REFERENCES visits (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE report_results (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    report_id       BIGINT UNSIGNED NOT NULL,
    test_code       VARCHAR(50) NOT NULL,
    test_name       VARCHAR(100) NOT NULL,
    value_text      VARCHAR(50) NULL,
    value_num       DECIMAL(12,4) NULL,
    unit            VARCHAR(30) NULL,
    ref_range_text  VARCHAR(50) NULL,
    range_source    ENUM('lab','system') NULL,
    status          ENUM('normal','borderline','high','low','critical','abnormal') NULL,
    confidence      ENUM('high','medium','low') NULL,
    edited_by_doctor TINYINT(1) NOT NULL DEFAULT 0,
    KEY ix_results_report (report_id),
    KEY ix_results_test (test_code),
    CONSTRAINT fk_results_report FOREIGN KEY (report_id) REFERENCES reports (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE reference_ranges (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    clinic_id       BIGINT UNSIGNED NULL COMMENT 'NULL = system default',
    test_code       VARCHAR(50) NOT NULL,
    sex             ENUM('M','F','ANY') NOT NULL DEFAULT 'ANY',
    age_min_days    INT UNSIGNED NOT NULL DEFAULT 0,
    age_max_days    INT UNSIGNED NOT NULL DEFAULT 54750,
    low             DECIMAL(12,4) NULL,
    high            DECIMAL(12,4) NULL,
    critical_low    DECIMAL(12,4) NULL,
    critical_high   DECIMAL(12,4) NULL,
    unit            VARCHAR(30) NOT NULL,
    KEY ix_ranges_lookup (test_code, sex, clinic_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE ai_analyses (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    clinic_id       BIGINT UNSIGNED NOT NULL,
    visit_id        BIGINT UNSIGNED NULL,
    analysis_type   ENUM('extraction','clinical','safety') NOT NULL,
    content         JSON NULL,
    claude_model    VARCHAR(50) NOT NULL,
    prompt_version  VARCHAR(20) NOT NULL,
    input_tokens    INT UNSIGNED NOT NULL DEFAULT 0,
    output_tokens   INT UNSIGNED NOT NULL DEFAULT 0,
    cache_read_tokens INT UNSIGNED NOT NULL DEFAULT 0,
    latency_ms      INT UNSIGNED NULL,
    status          ENUM('running','done','failed') NOT NULL DEFAULT 'running',
    error           TEXT NULL,
    created_by      BIGINT UNSIGNED NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    KEY ix_analyses_visit (visit_id),
    KEY ix_analyses_usage (clinic_id, created_at),
    CONSTRAINT fk_analyses_visit FOREIGN KEY (visit_id) REFERENCES visits (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE ai_suggestions (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    analysis_id     BIGINT UNSIGNED NOT NULL,
    section         ENUM('diagnosis','investigation','advice','next_plan','red_flag') NOT NULL,
    text_en         TEXT NOT NULL,
    text_bn         TEXT NULL,
    likelihood      VARCHAR(20) NULL,
    source_refs     JSON NULL,
    action          ENUM('pending','accepted','edited','rejected') NOT NULL DEFAULT 'pending',
    edited_text     TEXT NULL,
    reason          VARCHAR(255) NULL,
    acted_by        BIGINT UNSIGNED NULL,
    acted_at        DATETIME NULL,
    KEY ix_suggestions_analysis (analysis_id),
    CONSTRAINT fk_suggestions_analysis FOREIGN KEY (analysis_id) REFERENCES ai_analyses (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE prompt_versions (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name            VARCHAR(50) NOT NULL,
    version         VARCHAR(20) NOT NULL,
    system_prompt   MEDIUMTEXT NOT NULL,
    `schema`        JSON NULL,
    is_active       TINYINT(1) NOT NULL DEFAULT 0,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uq_prompt_versions (name, version)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
