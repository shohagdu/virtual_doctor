-- Phase 3: appointments, follow-ups, SMS, billing (Patient_Management_Spec §5.6–5.8)

ALTER TABLE appointments
    ADD COLUMN visit_type ENUM('new','followup','report') NOT NULL DEFAULT 'new' AFTER start_at,
    ADD COLUMN notes VARCHAR(255) NULL AFTER source,
    ADD COLUMN visit_id BIGINT UNSIGNED NULL AFTER notes,
    ADD COLUMN created_by BIGINT UNSIGNED NULL AFTER sms_sent_at,
    ADD KEY ix_appointments_patient (patient_id, start_at),
    ADD CONSTRAINT fk_appointments_visit FOREIGN KEY (visit_id) REFERENCES visits (id);

ALTER TABLE followups
    ADD COLUMN updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    ADD KEY ix_followups_clinic_due (clinic_id, status, due_date);

-- One bill per visit; receipt numbers come from clinic_counters ('receipt').
ALTER TABLE bills
    ADD COLUMN clinic_id BIGINT UNSIGNED NULL AFTER id,
    ADD COLUMN receipt_no VARCHAR(20) NULL AFTER visit_id,
    ADD COLUMN updated_by BIGINT UNSIGNED NULL AFTER created_by,
    ADD COLUMN updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP;
UPDATE bills b JOIN visits v ON v.id = b.visit_id SET b.clinic_id = v.clinic_id;
ALTER TABLE bills
    MODIFY clinic_id BIGINT UNSIGNED NOT NULL,
    DROP INDEX ix_bills_visit,
    ADD UNIQUE KEY uq_bills_visit (visit_id),
    ADD UNIQUE KEY uq_bills_receipt (clinic_id, receipt_no);

-- Every SMS the clinic sends, with its delivery state (§13 "SMS failed: retry; show
-- failed status"). Sending happens in the jobs worker.
CREATE TABLE sms_messages (
    id              BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    clinic_id       BIGINT UNSIGNED NOT NULL,
    patient_id      BIGINT UNSIGNED NULL,
    phone           VARCHAR(15) NOT NULL,
    body            TEXT NOT NULL,
    kind            VARCHAR(30) NOT NULL,
    ref_id          BIGINT UNSIGNED NULL,
    status          ENUM('queued','sent','failed','cancelled') NOT NULL DEFAULT 'queued',
    attempts        INT UNSIGNED NOT NULL DEFAULT 0,
    error           VARCHAR(255) NULL,
    provider_ref    VARCHAR(100) NULL,
    send_after      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by      BIGINT UNSIGNED NULL,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sent_at         DATETIME NULL,
    KEY ix_sms_ref (kind, ref_id),
    KEY ix_sms_clinic (clinic_id, created_at),
    CONSTRAINT fk_sms_clinic FOREIGN KEY (clinic_id) REFERENCES clinics (id),
    CONSTRAINT fk_sms_patient FOREIGN KEY (patient_id) REFERENCES patients (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
