-- Patient Management module (Patient_Management_Spec §9.2)

CREATE TABLE patients (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    clinic_id       BIGINT UNSIGNED NOT NULL,
    patient_code    VARCHAR(20) NOT NULL,
    title           VARCHAR(10) NULL,
    name_en         VARCHAR(150) NOT NULL,
    name_bn         VARCHAR(150) NULL,
    phone           VARCHAR(15) NOT NULL,
    dob             DATE NOT NULL,
    dob_estimated   TINYINT(1) NOT NULL DEFAULT 0,
    sex             ENUM('M','F','O') NOT NULL,
    blood_group     VARCHAR(5) NULL,
    marital_status  VARCHAR(20) NULL,
    address         VARCHAR(255) NULL,
    district        VARCHAR(100) NULL,
    nid             VARBINARY(255) NULL COMMENT 'Encrypted at application level',
    guardian_name   VARCHAR(150) NULL,
    guardian_phone  VARCHAR(15) NULL,
    referred_by     VARCHAR(150) NULL,
    photo_path      VARCHAR(255) NULL,
    created_by      BIGINT UNSIGNED NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at      DATETIME NULL,
    UNIQUE KEY uq_patients_code (patient_code),
    KEY ix_patients_phone (clinic_id, phone),
    KEY ix_patients_name (clinic_id, name_en),
    FULLTEXT KEY ft_patients_name (name_en, name_bn) WITH PARSER ngram,
    CONSTRAINT fk_patients_clinic FOREIGN KEY (clinic_id) REFERENCES clinics (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE patient_medical_profile (
    patient_id          BIGINT UNSIGNED NOT NULL PRIMARY KEY,
    allergies           JSON NULL,
    chronic_conditions  JSON NULL,
    long_term_medicines JSON NULL,
    past_history        TEXT NULL,
    family_history      TEXT NULL,
    social_history      TEXT NULL,
    gynae_obs_history   TEXT NULL,
    pregnancy_status    VARCHAR(30) NULL,
    vaccination         JSON NULL,
    updated_at          DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    CONSTRAINT fk_pmp_patient FOREIGN KEY (patient_id) REFERENCES patients (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE visits (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    clinic_id       BIGINT UNSIGNED NOT NULL,
    patient_id      BIGINT UNSIGNED NOT NULL,
    visit_no        INT UNSIGNED NOT NULL,
    visit_date      DATE NOT NULL,
    visit_type      ENUM('new','followup','report') NOT NULL DEFAULT 'new',
    queue_serial    INT UNSIGNED NULL,
    status          ENUM('waiting','in_consultation','seen','cancelled') NOT NULL DEFAULT 'waiting',
    doctor_id       BIGINT UNSIGNED NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    deleted_at      DATETIME NULL,
    KEY ix_visits_patient_date (patient_id, visit_date),
    KEY ix_visits_queue (clinic_id, visit_date, status),
    CONSTRAINT fk_visits_patient FOREIGN KEY (patient_id) REFERENCES patients (id),
    CONSTRAINT fk_visits_doctor FOREIGN KEY (doctor_id) REFERENCES users (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE vitals (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    visit_id        BIGINT UNSIGNED NOT NULL,
    bp_sys          SMALLINT UNSIGNED NULL,
    bp_dia          SMALLINT UNSIGNED NULL,
    pulse           SMALLINT UNSIGNED NULL,
    temp            DECIMAL(4,1) NULL,
    weight          DECIMAL(5,2) NULL,
    height          DECIMAL(5,2) NULL,
    bmi             DECIMAL(4,1) NULL,
    spo2            TINYINT UNSIGNED NULL,
    rbs             DECIMAL(5,2) NULL,
    recorded_by     BIGINT UNSIGNED NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    KEY ix_vitals_visit (visit_id),
    CONSTRAINT fk_vitals_visit FOREIGN KEY (visit_id) REFERENCES visits (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- Each saved/printed version is a separate locked row (PM-11)
CREATE TABLE prescriptions (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    visit_id        BIGINT UNSIGNED NOT NULL,
    version         INT UNSIGNED NOT NULL DEFAULT 1,
    complaints      JSON NULL,
    examination     JSON NULL,
    investigations  JSON NULL,
    diagnosis       JSON NULL,
    treatment_plan  JSON NULL,
    advice          JSON NULL,
    next_plan       JSON NULL,
    operation_notes TEXT NULL,
    next_visit_date DATE NULL,
    locked          TINYINT(1) NOT NULL DEFAULT 0,
    pdf_path        VARCHAR(255) NULL,
    created_by      BIGINT UNSIGNED NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uq_prescriptions_version (visit_id, version),
    CONSTRAINT fk_prescriptions_visit FOREIGN KEY (visit_id) REFERENCES visits (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE medicines (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    brand           VARCHAR(150) NOT NULL,
    generic         VARCHAR(255) NOT NULL,
    strength        VARCHAR(100) NULL,
    form            VARCHAR(50) NULL,
    company         VARCHAR(150) NULL,
    price           DECIMAL(10,2) NULL,
    is_active       TINYINT(1) NOT NULL DEFAULT 1,
    KEY ix_medicines_brand (brand),
    FULLTEXT KEY ft_medicines (brand, generic)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE prescription_medicines (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    prescription_id BIGINT UNSIGNED NOT NULL,
    sort_order      SMALLINT UNSIGNED NOT NULL DEFAULT 0,
    medicine_id     BIGINT UNSIGNED NULL,
    brand           VARCHAR(150) NOT NULL,
    generic         VARCHAR(255) NULL,
    strength        VARCHAR(100) NULL,
    form            VARCHAR(50) NULL,
    dose_pattern    VARCHAR(30) NULL,
    timing          VARCHAR(100) NULL,
    duration        VARCHAR(50) NULL,
    notes           VARCHAR(255) NULL,
    KEY ix_pm_prescription (prescription_id, sort_order),
    CONSTRAINT fk_pm_prescription FOREIGN KEY (prescription_id) REFERENCES prescriptions (id),
    CONSTRAINT fk_pm_medicine FOREIGN KEY (medicine_id) REFERENCES medicines (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE appointments (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    clinic_id       BIGINT UNSIGNED NOT NULL,
    patient_id      BIGINT UNSIGNED NOT NULL,
    start_at        DATETIME NOT NULL,
    status          ENUM('booked','arrived','seen','no_show','cancelled') NOT NULL DEFAULT 'booked',
    source          VARCHAR(30) NULL,
    sms_sent_at     DATETIME NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    KEY ix_appointments_calendar (clinic_id, start_at),
    CONSTRAINT fk_appointments_patient FOREIGN KEY (patient_id) REFERENCES patients (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE followups (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    clinic_id       BIGINT UNSIGNED NOT NULL,
    patient_id      BIGINT UNSIGNED NOT NULL,
    prescription_id BIGINT UNSIGNED NULL,
    due_date        DATE NOT NULL,
    status          ENUM('pending','came','missed') NOT NULL DEFAULT 'pending',
    reminder_sent_at DATETIME NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    KEY ix_followups_due (status, due_date),
    CONSTRAINT fk_followups_patient FOREIGN KEY (patient_id) REFERENCES patients (id),
    CONSTRAINT fk_followups_prescription FOREIGN KEY (prescription_id) REFERENCES prescriptions (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE bills (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    visit_id        BIGINT UNSIGNED NOT NULL,
    fee             DECIMAL(10,2) NOT NULL DEFAULT 0,
    discount        DECIMAL(10,2) NOT NULL DEFAULT 0,
    discount_reason VARCHAR(150) NULL,
    paid            DECIMAL(10,2) NOT NULL DEFAULT 0,
    method          ENUM('cash','bkash','nagad','card') NOT NULL DEFAULT 'cash',
    status          ENUM('paid','due') NOT NULL DEFAULT 'due',
    created_by      BIGINT UNSIGNED NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    KEY ix_bills_visit (visit_id),
    CONSTRAINT fk_bills_visit FOREIGN KEY (visit_id) REFERENCES visits (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE ai_summaries (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    patient_id      BIGINT UNSIGNED NOT NULL,
    content         JSON NOT NULL,
    claude_model    VARCHAR(50) NOT NULL,
    prompt_version  VARCHAR(20) NOT NULL,
    input_tokens    INT UNSIGNED NOT NULL DEFAULT 0,
    output_tokens   INT UNSIGNED NOT NULL DEFAULT 0,
    created_by      BIGINT UNSIGNED NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    KEY ix_ai_summaries_patient (patient_id, created_at),
    CONSTRAINT fk_ai_summaries_patient FOREIGN KEY (patient_id) REFERENCES patients (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
