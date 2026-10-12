-- Optimistic concurrency for prescription drafts (Patient_Management_Spec §13:
-- "prevent overwrite of prescriptions" when two users edit the same visit).
ALTER TABLE prescriptions ADD COLUMN revision INT UNSIGNED NOT NULL DEFAULT 0 AFTER version;
