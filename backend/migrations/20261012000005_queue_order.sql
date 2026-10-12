-- Queue order (Patient_Management_Spec §5.1 "drag to reorder queue"). The serial is the
-- token the patient holds and never changes; queue_position is the order they are called in.
ALTER TABLE visits ADD COLUMN queue_position INT UNSIGNED NULL AFTER queue_serial;
UPDATE visits SET queue_position = queue_serial;
