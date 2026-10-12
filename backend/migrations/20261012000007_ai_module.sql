-- Phase 4: AI report analysis, AI patient summary, smart search (AI spec §10, PM spec §7)

-- System reference ranges (AI spec §5.3). Common adult textbook values, used only when
-- the lab did not print a range; clinics should review them and can add their own
-- rows with clinic_id set. For children only the critical limits are seeded, since
-- normal ranges change with age.
INSERT INTO reference_ranges (test_code, sex, age_min_days, age_max_days, low, high, critical_low, critical_high, unit) VALUES
    ('HB','M',6570,54750,13.0,17.0,7,20,'g/dL'),
    ('HB','F',6570,54750,12.0,15.5,7,20,'g/dL'),
    ('WBC','ANY',6570,54750,4000,11000,2000,30000,'/cumm'),
    ('PLT','ANY',6570,54750,150000,450000,50000,1000000,'/cumm'),
    ('ESR','M',6570,54750,0,15,NULL,NULL,'mm/1st hr'),
    ('ESR','F',6570,54750,0,20,NULL,NULL,'mm/1st hr'),
    ('NEUT','ANY',6570,54750,40,75,NULL,NULL,'%'),
    ('LYMPH','ANY',6570,54750,20,45,NULL,NULL,'%'),
    ('MONO','ANY',6570,54750,2,10,NULL,NULL,'%'),
    ('EOS','ANY',6570,54750,1,6,NULL,NULL,'%'),
    ('PCV','M',6570,54750,40,50,NULL,NULL,'%'),
    ('PCV','F',6570,54750,36,46,NULL,NULL,'%'),
    ('MCV','ANY',6570,54750,80,100,NULL,NULL,'fL'),
    ('MCH','ANY',6570,54750,27,32,NULL,NULL,'pg'),
    ('MCHC','ANY',6570,54750,32,36,NULL,NULL,'g/dL'),
    ('RBS','ANY',6570,54750,3.9,7.8,2.8,22.2,'mmol/L'),
    ('FBS','ANY',6570,54750,3.9,5.5,2.8,22.2,'mmol/L'),
    ('PPBS','ANY',6570,54750,3.9,7.8,2.8,22.2,'mmol/L'),
    ('HBA1C','ANY',6570,54750,4.0,5.6,NULL,NULL,'%'),
    ('CREAT','M',6570,54750,0.7,1.3,NULL,NULL,'mg/dL'),
    ('CREAT','F',6570,54750,0.5,1.1,NULL,NULL,'mg/dL'),
    ('UREA','ANY',6570,54750,15,40,NULL,NULL,'mg/dL'),
    ('NA','ANY',6570,54750,135,145,120,160,'mmol/L'),
    ('K','ANY',6570,54750,3.5,5.1,2.5,6.0,'mmol/L'),
    ('CL','ANY',6570,54750,98,107,NULL,NULL,'mmol/L'),
    ('CA','ANY',6570,54750,8.5,10.5,6.5,13.0,'mg/dL'),
    ('URIC','M',6570,54750,3.4,7.0,NULL,NULL,'mg/dL'),
    ('URIC','F',6570,54750,2.4,6.0,NULL,NULL,'mg/dL'),
    ('ALT','ANY',6570,54750,NULL,40,NULL,NULL,'U/L'),
    ('AST','ANY',6570,54750,NULL,40,NULL,NULL,'U/L'),
    ('ALP','ANY',6570,54750,44,147,NULL,NULL,'U/L'),
    ('BILT','ANY',6570,54750,NULL,1.2,NULL,NULL,'mg/dL'),
    ('TSH','ANY',6570,54750,0.4,4.0,NULL,NULL,'mIU/L'),
    ('FT4','ANY',6570,54750,0.8,1.8,NULL,NULL,'ng/dL'),
    ('CHOL','ANY',6570,54750,NULL,200,NULL,NULL,'mg/dL'),
    ('TG','ANY',6570,54750,NULL,150,NULL,NULL,'mg/dL'),
    ('HDL','M',6570,54750,40,NULL,NULL,NULL,'mg/dL'),
    ('HDL','F',6570,54750,50,NULL,NULL,NULL,'mg/dL'),
    ('LDL','ANY',6570,54750,NULL,130,NULL,NULL,'mg/dL'),
    ('CRP','ANY',6570,54750,NULL,5,NULL,NULL,'mg/L'),
    ('PUS','ANY',6570,54750,NULL,5,NULL,NULL,'/HPF'),
    ('RBC_URINE','ANY',6570,54750,NULL,2,NULL,NULL,'/HPF'),
    ('HB','ANY',0,6569,NULL,NULL,7,NULL,'g/dL'),
    ('PLT','ANY',0,6569,NULL,NULL,50000,NULL,'/cumm'),
    ('K','ANY',0,6569,NULL,NULL,2.5,6.0,'mmol/L'),
    ('NA','ANY',0,6569,NULL,NULL,120,160,'mmol/L'),
    ('RBS','ANY',0,6569,NULL,NULL,2.8,22.2,'mmol/L'),
    ('FBS','ANY',0,6569,NULL,NULL,2.8,22.2,'mmol/L');

ALTER TABLE reports
    ADD COLUMN original_name VARCHAR(150) NULL AFTER mime_type,
    ADD COLUMN name_mismatch TINYINT(1) NOT NULL DEFAULT 0 AFTER impression,
    ADD COLUMN error VARCHAR(255) NULL AFTER extraction_raw;

ALTER TABLE report_results
    ADD COLUMN sort_order INT UNSIGNED NOT NULL DEFAULT 0 AFTER report_id,
    ADD COLUMN lab_flag VARCHAR(10) NULL AFTER ref_range_text;

-- Every Claude call is logged here for cost tracking, including the helper tasks.
ALTER TABLE ai_analyses
    MODIFY analysis_type ENUM('extraction','clinical','safety','summary','search','sms_draft') NOT NULL,
    ADD COLUMN patient_id BIGINT UNSIGNED NULL AFTER visit_id,
    ADD COLUMN report_id BIGINT UNSIGNED NULL AFTER patient_id;

-- Safety warnings are reviewed like suggestions: dismissed with a reason (FR-32).
ALTER TABLE ai_suggestions
    MODIFY section ENUM('diagnosis','investigation','advice','next_plan','red_flag','safety') NOT NULL;

ALTER TABLE ai_summaries
    ADD COLUMN clinic_id BIGINT UNSIGNED NULL AFTER id;
UPDATE ai_summaries s JOIN patients p ON p.id = s.patient_id SET s.clinic_id = p.clinic_id;
