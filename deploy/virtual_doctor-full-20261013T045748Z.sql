-- MySQL dump 10.13  Distrib 9.7.2, for Linux (x86_64)
--
-- Host: localhost    Database: virtual_doctor
-- ------------------------------------------------------
-- Server version	9.7.2

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!50503 SET NAMES utf8mb4 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Current Database: `virtual_doctor`
--

CREATE DATABASE /*!32312 IF NOT EXISTS*/ `virtual_doctor` /*!40100 DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci */ /*!80016 DEFAULT ENCRYPTION='N' */;

USE `virtual_doctor`;

--
-- Table structure for table `_sqlx_migrations`
--

DROP TABLE IF EXISTS `_sqlx_migrations`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `_sqlx_migrations` (
  `version` bigint NOT NULL,
  `description` text NOT NULL,
  `installed_on` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `success` tinyint(1) NOT NULL,
  `checksum` blob NOT NULL,
  `execution_time` bigint NOT NULL,
  PRIMARY KEY (`version`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `_sqlx_migrations`
--

LOCK TABLES `_sqlx_migrations` WRITE;
/*!40000 ALTER TABLE `_sqlx_migrations` DISABLE KEYS */;
INSERT INTO `_sqlx_migrations` VALUES (20261012000001,'core','2026-10-12 05:58:34',1,0xFA1BD011B158F550FC856FF91DC7830C89377634F684BD3826F7FBB99E7F5604BC3DC7003B748B853EACB21698871DE0,122763982),(20261012000002,'patient management','2026-10-12 05:58:34',1,0x1E519343A066505717A55C4B56A0ADAAAEDA375345FA2C9E7D669CC68B6B997CA6AB9F258225427BE0846F875BF5AF97,638628261),(20261012000003,'ai reports','2026-10-12 05:58:34',1,0x112D70EE10985121BED7D74879BFCBF2E45A58168734707F3E8FDBD64D30BCE792F8470FE5F37508E32AF366FFFF18E9,205030798),(20261012000004,'prescription revision','2026-10-12 06:12:43',1,0x1A5FB0E37F5C93B1640507669C66B867002E024F38E66CE047FC78DCFA19E162E36A7772497EF74BB06203517E097E54,62424056),(20261012000005,'queue order','2026-10-12 07:01:52',1,0xA19DD97699026BD81AFD4509AAD9D638DC2620467E0EE12829BCEC82DF3BBCD54DFA66E9127DEC1D36DBB78375C9CC51,39028106),(20261012000006,'appointments billing sms','2026-10-12 09:06:01',1,0x6C96C9EB6F765A7088B3E1780DD068C7FB37C2A1F55D3CB681991E351A1E5A26059BC679DF46CB8040EF358278FAE050,238431063),(20261012000007,'ai module','2026-10-12 09:43:45',1,0x6FE553BB8D5B33744489AB69FE4CF065398E83F66CA0CF8AD1EB8F74C84C48DAC7CB4955FCF68A9AE7444B67F8454A9A,194495015),(20261012000008,'pending items','2026-10-12 09:56:58',1,0x587398FF8CB890CAF1A2012B0713C5FB82F3D31F8A5F2701658FA6B3FC228BE79B2BE7852DCA1CA59273BB4296BEDD78,556517957);
/*!40000 ALTER TABLE `_sqlx_migrations` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ai_analyses`
--

DROP TABLE IF EXISTS `ai_analyses`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ai_analyses` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned NOT NULL,
  `visit_id` bigint unsigned DEFAULT NULL,
  `patient_id` bigint unsigned DEFAULT NULL,
  `report_id` bigint unsigned DEFAULT NULL,
  `analysis_type` enum('extraction','clinical','safety','summary','search','sms_draft') NOT NULL,
  `content` json DEFAULT NULL,
  `claude_model` varchar(50) NOT NULL,
  `prompt_version` varchar(20) NOT NULL,
  `input_tokens` int unsigned NOT NULL DEFAULT '0',
  `output_tokens` int unsigned NOT NULL DEFAULT '0',
  `cache_read_tokens` int unsigned NOT NULL DEFAULT '0',
  `latency_ms` int unsigned DEFAULT NULL,
  `status` enum('running','done','failed') NOT NULL DEFAULT 'running',
  `error` text,
  `created_by` bigint unsigned DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `ix_analyses_visit` (`visit_id`),
  KEY `ix_analyses_usage` (`clinic_id`,`created_at`),
  CONSTRAINT `fk_analyses_visit` FOREIGN KEY (`visit_id`) REFERENCES `visits` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=13 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ai_analyses`
--

LOCK TABLES `ai_analyses` WRITE;
/*!40000 ALTER TABLE `ai_analyses` DISABLE KEYS */;
INSERT INTO `ai_analyses` VALUES (1,1,4,3,1,'extraction','{\"results\": [{\"test\": \"Haemoglobin\", \"unit\": \"g/dL\", \"range\": \"12-15.5\", \"value\": \"10.2\", \"lab_flag\": \"L\", \"confidence\": \"high\"}, {\"test\": \"Total Count (WBC)\", \"unit\": \"/cumm\", \"range\": \"4,000-11,000\", \"value\": \"13,800\", \"lab_flag\": \"H\", \"confidence\": \"high\"}, {\"test\": \"Platelet count\", \"unit\": \"/cumm\", \"range\": \"1,50,000-4,50,000\", \"value\": \"2,40,000\", \"confidence\": \"high\"}, {\"test\": \"ESR\", \"unit\": \"mm/1st hr\", \"range\": \"0-20\", \"value\": \"38\", \"confidence\": \"low\"}, {\"test\": \"Neutrophils\", \"unit\": \"%\", \"range\": \"40-75\", \"value\": \"82\", \"confidence\": \"medium\"}, {\"test\": \"S. Creatinine\", \"unit\": \"mg/dL\", \"value\": \"2.4\", \"confidence\": \"high\"}], \"lab_name\": \"ABC Diagnostic\", \"impression\": null, \"report_date\": \"2026-10-10\", \"report_type\": \"CBC\", \"unreadable_parts\": [\"stamp at bottom\"], \"patient_name_on_report\": \"Smoke Allergic\"}','claude-sonnet-5-5','v1',1200,300,900,1,'done',NULL,6,'2026-10-12 09:44:14'),(2,1,4,3,NULL,'clinical','{\"summary\": \"Leucocytosis with neutrophilia suggests an acute bacterial infection. Mild anaemia.\", \"findings\": [{\"text\": \"Raised WBC with neutrophilia suggests bacterial infection\", \"source\": [\"WBC 13,800 /cumm\", \"Neutrophils 82%\"]}], \"advice_bn\": [\"প্রচুর পানি পান করবেন\"], \"advice_en\": [\"Drink plenty of water\"], \"conflicts\": [\"Raised creatinine not addressed in the prescription\"], \"next_plan\": \"Review with CBC after 7 days\", \"red_flags\": [\"Persistent high fever\"], \"trend_comments\": [], \"possible_issues\": [{\"name\": \"Acute bacterial infection\", \"reason\": \"WBC and neutrophils raised\", \"likelihood\": \"High\"}, {\"name\": \"Mild anaemia\", \"reason\": \"Hb 10.2 g/dL\", \"likelihood\": \"Moderate\"}], \"report_summaries\": [{\"report\": \"CBC\", \"summary\": \"Raised WBC and neutrophils; low Hb.\"}], \"suggested_investigations\": [{\"name\": \"Urine R/E\", \"reason\": \"Look for a source of infection\"}]}','claude-sonnet-5-5','v1',1200,300,900,447,'done',NULL,6,'2026-10-12 09:44:14'),(3,1,4,3,NULL,'safety','{\"warnings\": [{\"kind\": \"lab\", \"source\": [], \"message\": \"Paracetamol is fine, but review the dose if liver tests are abnormal.\", \"severity\": \"low\", \"medicines\": [\"Napa\"]}]}','claude-sonnet-5-5','v1',1200,300,900,1,'done',NULL,6,'2026-10-12 09:44:14'),(4,1,NULL,3,NULL,'summary','{\"alerts\": [\"Allergy: Penicillin\"], \"one_line\": \"Adult with a recent bacterial infection and mild anaemia.\", \"lab_trends\": [\"Hb 10.2 g/dL (Oct 2026) — low\"], \"active_problems\": [\"Bacterial infection (Oct 2026)\"], \"points_to_check\": [\"Repeat CBC\", \"Kidney function\"], \"recurring_issues\": [], \"medicines_history\": [\"Paracetamol\"]}','claude-sonnet-5-5','v1',1200,300,900,183,'done',NULL,6,'2026-10-12 09:44:15'),(5,1,NULL,NULL,NULL,'search','{\"sex\": \"F\", \"visited_from\": \"2026-07-12\", \"diagnosis_terms\": [\"uti\", \"urinary tract infection\"]}','claude-haiku-5-5','v1',1200,300,900,1,'done',NULL,6,'2026-10-12 09:44:15'),(6,1,NULL,4,NULL,'sms_draft','{\"text\": \"প্রিয় {name}, {date} তারিখে আপনার ফলো-আপ। সিরিয়ালের জন্য কল করুন {phone}। {clinic}\"}','claude-haiku-5-5','v1',1200,300,900,1,'done',NULL,7,'2026-10-12 09:44:15'),(7,1,4,3,NULL,'safety','{\"warnings\": [{\"kind\": \"lab\", \"source\": [], \"message\": \"Paracetamol is fine, but review the dose if liver tests are abnormal.\", \"severity\": \"low\", \"medicines\": [\"Napa\"]}]}','claude-sonnet-5-5','v1',1200,300,900,1,'done',NULL,4,'2026-10-12 09:50:15'),(8,1,NULL,NULL,NULL,'search','{\"sex\": \"F\", \"visited_from\": \"2026-07-12\", \"diagnosis_terms\": [\"uti\", \"urinary tract infection\"]}','claude-haiku-5-5','v1',1200,300,900,2,'done',NULL,4,'2026-10-12 09:50:28'),(9,1,5,4,2,'extraction','{\"results\": [{\"test\": \"Haemoglobin\", \"unit\": \"g/dL\", \"range\": \"12-15.5\", \"value\": \"10.2\", \"lab_flag\": \"L\", \"confidence\": \"high\"}, {\"test\": \"Total Count (WBC)\", \"unit\": \"/cumm\", \"range\": \"4,000-11,000\", \"value\": \"13,800\", \"lab_flag\": \"H\", \"confidence\": \"high\"}, {\"test\": \"Platelet count\", \"unit\": \"/cumm\", \"range\": \"1,50,000-4,50,000\", \"value\": \"2,40,000\", \"confidence\": \"high\"}, {\"test\": \"ESR\", \"unit\": \"mm/1st hr\", \"range\": \"0-20\", \"value\": \"38\", \"confidence\": \"low\"}, {\"test\": \"Neutrophils\", \"unit\": \"%\", \"range\": \"40-75\", \"value\": \"82\", \"confidence\": \"medium\"}, {\"test\": \"S. Creatinine\", \"unit\": \"mg/dL\", \"value\": \"2.4\", \"confidence\": \"high\"}], \"lab_name\": \"ABC Diagnostic\", \"impression\": null, \"report_date\": \"2026-10-10\", \"report_type\": \"CBC\", \"unreadable_parts\": [\"stamp at bottom\"], \"patient_name_on_report\": \"Smoke Allergic\"}','claude-sonnet-5-5','v1',1200,300,900,75,'done',NULL,4,'2026-10-12 09:51:18'),(10,1,5,4,NULL,'clinical','{\"summary\": \"Leucocytosis with neutrophilia suggests an acute bacterial infection. Mild anaemia.\", \"findings\": [{\"text\": \"Raised WBC with neutrophilia suggests bacterial infection\", \"source\": [\"WBC 13,800 /cumm\", \"Neutrophils 82%\"]}], \"advice_bn\": [\"প্রচুর পানি পান করবেন\"], \"advice_en\": [\"Drink plenty of water\"], \"conflicts\": [\"Raised creatinine not addressed in the prescription\"], \"next_plan\": \"Review with CBC after 7 days\", \"red_flags\": [\"Persistent high fever\"], \"trend_comments\": [], \"possible_issues\": [{\"name\": \"Acute bacterial infection\", \"reason\": \"WBC and neutrophils raised\", \"likelihood\": \"High\"}, {\"name\": \"Mild anaemia\", \"reason\": \"Hb 10.2 g/dL\", \"likelihood\": \"Moderate\"}], \"report_summaries\": [{\"report\": \"CBC\", \"summary\": \"Raised WBC and neutrophils; low Hb.\"}], \"suggested_investigations\": [{\"name\": \"Urine R/E\", \"reason\": \"Look for a source of infection\"}]}','claude-sonnet-5-5','v1',1200,300,900,445,'done',NULL,4,'2026-10-12 09:51:26'),(11,1,11,5,NULL,'safety','{\"warnings\": []}','rules','rules-v1',0,0,0,NULL,'done',NULL,6,'2026-10-13 03:49:29'),(12,1,13,7,NULL,'safety','{\"warnings\": []}','rules','rules-v1',0,0,0,NULL,'done',NULL,6,'2026-10-13 04:24:12');
/*!40000 ALTER TABLE `ai_analyses` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ai_suggestions`
--

DROP TABLE IF EXISTS `ai_suggestions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ai_suggestions` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `analysis_id` bigint unsigned NOT NULL,
  `section` enum('diagnosis','investigation','advice','next_plan','red_flag','safety') NOT NULL,
  `text_en` text NOT NULL,
  `text_bn` text,
  `likelihood` varchar(20) DEFAULT NULL,
  `source_refs` json DEFAULT NULL,
  `action` enum('pending','accepted','edited','rejected') NOT NULL DEFAULT 'pending',
  `edited_text` text,
  `reason` varchar(255) DEFAULT NULL,
  `acted_by` bigint unsigned DEFAULT NULL,
  `acted_at` datetime DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `ix_suggestions_analysis` (`analysis_id`),
  CONSTRAINT `fk_suggestions_analysis` FOREIGN KEY (`analysis_id`) REFERENCES `ai_analyses` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=16 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ai_suggestions`
--

LOCK TABLES `ai_suggestions` WRITE;
/*!40000 ALTER TABLE `ai_suggestions` DISABLE KEYS */;
INSERT INTO `ai_suggestions` VALUES (1,2,'diagnosis','Acute bacterial infection',NULL,'High','[\"WBC and neutrophils raised\"]','accepted',NULL,NULL,6,'2026-10-12 09:44:14'),(2,2,'diagnosis','Mild anaemia',NULL,'Moderate','[\"Hb 10.2 g/dL\"]','rejected',NULL,'not relevant',6,'2026-10-12 09:44:14'),(3,2,'investigation','Urine R/E',NULL,NULL,'[\"Look for a source of infection\"]','pending',NULL,NULL,NULL,NULL),(4,2,'advice','Drink plenty of water','প্রচুর পানি পান করবেন',NULL,'[]','pending',NULL,NULL,NULL,NULL),(5,2,'next_plan','Review with CBC after 7 days',NULL,NULL,'[]','pending',NULL,NULL,NULL,NULL),(6,2,'red_flag','Persistent high fever',NULL,NULL,'[]','pending',NULL,NULL,NULL,NULL),(7,3,'safety','NSAIDs can worsen kidney function — creatinine is raised. Review the need for an NSAID.',NULL,'high','[\"Creatinine 2.4 mg/dL\"]','pending',NULL,NULL,NULL,NULL),(8,3,'safety','Paracetamol is fine, but review the dose if liver tests are abnormal.',NULL,'low','[]','pending',NULL,NULL,NULL,NULL),(9,7,'safety','Paracetamol is fine, but review the dose if liver tests are abnormal.',NULL,'low','[]','pending',NULL,NULL,NULL,NULL),(10,10,'diagnosis','Acute bacterial infection',NULL,'High','[\"WBC and neutrophils raised\"]','accepted',NULL,NULL,4,'2026-10-12 09:51:31'),(11,10,'diagnosis','Mild anaemia',NULL,'Moderate','[\"Hb 10.2 g/dL\"]','accepted',NULL,NULL,4,'2026-10-12 09:51:31'),(12,10,'investigation','Urine R/E',NULL,NULL,'[\"Look for a source of infection\"]','accepted',NULL,NULL,4,'2026-10-12 09:51:31'),(13,10,'advice','Drink plenty of water','প্রচুর পানি পান করবেন',NULL,'[]','accepted',NULL,NULL,4,'2026-10-12 09:51:31'),(14,10,'next_plan','Review with CBC after 7 days',NULL,NULL,'[]','accepted',NULL,NULL,4,'2026-10-12 09:51:31'),(15,10,'red_flag','Persistent high fever',NULL,NULL,'[]','pending',NULL,NULL,NULL,NULL);
/*!40000 ALTER TABLE `ai_suggestions` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ai_summaries`
--

DROP TABLE IF EXISTS `ai_summaries`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `ai_summaries` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned DEFAULT NULL,
  `patient_id` bigint unsigned NOT NULL,
  `content` json NOT NULL,
  `claude_model` varchar(50) NOT NULL,
  `prompt_version` varchar(20) NOT NULL,
  `input_tokens` int unsigned NOT NULL DEFAULT '0',
  `output_tokens` int unsigned NOT NULL DEFAULT '0',
  `created_by` bigint unsigned DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `ix_ai_summaries_patient` (`patient_id`,`created_at`),
  CONSTRAINT `fk_ai_summaries_patient` FOREIGN KEY (`patient_id`) REFERENCES `patients` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ai_summaries`
--

LOCK TABLES `ai_summaries` WRITE;
/*!40000 ALTER TABLE `ai_summaries` DISABLE KEYS */;
INSERT INTO `ai_summaries` VALUES (1,1,3,'{\"alerts\": [\"Allergy: Penicillin\"], \"one_line\": \"Adult with a recent bacterial infection and mild anaemia.\", \"lab_trends\": [\"Hb 10.2 g/dL (Oct 2026) — low\"], \"active_problems\": [\"Bacterial infection (Oct 2026)\"], \"points_to_check\": [\"Repeat CBC\", \"Kidney function\"], \"recurring_issues\": [], \"medicines_history\": [\"Paracetamol\"]}','claude-sonnet-5-5','v1',1200,300,6,'2026-10-12 09:44:15');
/*!40000 ALTER TABLE `ai_summaries` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `appointments`
--

DROP TABLE IF EXISTS `appointments`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `appointments` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned NOT NULL,
  `patient_id` bigint unsigned NOT NULL,
  `start_at` datetime NOT NULL,
  `visit_type` enum('new','followup','report') NOT NULL DEFAULT 'new',
  `status` enum('booked','arrived','seen','no_show','cancelled') NOT NULL DEFAULT 'booked',
  `source` varchar(30) DEFAULT NULL,
  `notes` varchar(255) DEFAULT NULL,
  `visit_id` bigint unsigned DEFAULT NULL,
  `sms_sent_at` datetime DEFAULT NULL,
  `created_by` bigint unsigned DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `ix_appointments_calendar` (`clinic_id`,`start_at`),
  KEY `ix_appointments_patient` (`patient_id`,`start_at`),
  KEY `fk_appointments_visit` (`visit_id`),
  CONSTRAINT `fk_appointments_patient` FOREIGN KEY (`patient_id`) REFERENCES `patients` (`id`),
  CONSTRAINT `fk_appointments_visit` FOREIGN KEY (`visit_id`) REFERENCES `visits` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `appointments`
--

LOCK TABLES `appointments` WRITE;
/*!40000 ALTER TABLE `appointments` DISABLE KEYS */;
INSERT INTO `appointments` VALUES (1,1,3,'2026-10-15 12:00:00','followup','booked','desk','BP check',NULL,'2026-10-12 09:06:47',7,'2026-10-12 09:06:17','2026-10-12 09:06:47'),(2,1,4,'2026-10-12 12:00:00','new','arrived','desk',NULL,7,'2026-10-12 09:06:32',7,'2026-10-12 09:06:17','2026-10-12 09:06:45');
/*!40000 ALTER TABLE `appointments` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `audit_logs`
--

DROP TABLE IF EXISTS `audit_logs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `audit_logs` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned NOT NULL,
  `user_id` bigint unsigned DEFAULT NULL,
  `entity` varchar(50) NOT NULL,
  `entity_id` bigint unsigned DEFAULT NULL,
  `action` varchar(30) NOT NULL,
  `ip` varchar(45) DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `ix_audit_entity` (`entity`,`entity_id`),
  KEY `ix_audit_clinic_time` (`clinic_id`,`created_at`)
) ENGINE=InnoDB AUTO_INCREMENT=680 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `audit_logs`
--

LOCK TABLES `audit_logs` WRITE;
/*!40000 ALTER TABLE `audit_logs` DISABLE KEYS */;
INSERT INTO `audit_logs` VALUES (1,1,1,'user',1,'login',NULL,'2026-10-12 05:58:47'),(2,1,1,'patient',1,'create',NULL,'2026-10-12 05:58:48'),(3,1,1,'patient',2,'create',NULL,'2026-10-12 05:58:48'),(4,1,1,'user',1,'login',NULL,'2026-10-12 05:58:55'),(5,1,1,'user',1,'login',NULL,'2026-10-12 06:13:01'),(6,1,2,'user',2,'login',NULL,'2026-10-12 06:13:01'),(7,1,3,'user',3,'login',NULL,'2026-10-12 06:13:02'),(8,1,1,'patient',1,'update',NULL,'2026-10-12 06:13:02'),(9,1,2,'medical_profile',1,'update',NULL,'2026-10-12 06:13:02'),(10,1,2,'visit',1,'create',NULL,'2026-10-12 06:13:02'),(11,1,1,'visit',2,'create',NULL,'2026-10-12 06:13:02'),(12,1,1,'visit',2,'view',NULL,'2026-10-12 06:13:02'),(13,1,1,'prescription',2,'view',NULL,'2026-10-12 06:13:02'),(14,1,1,'prescription',2,'save',NULL,'2026-10-12 06:13:02'),(15,1,1,'prescription',2,'save',NULL,'2026-10-12 06:13:02'),(16,1,1,'prescription',2,'lock',NULL,'2026-10-12 06:13:02'),(17,1,1,'prescription',2,'save',NULL,'2026-10-12 06:13:02'),(18,1,1,'patient',1,'view_history',NULL,'2026-10-12 06:13:02'),(19,1,2,'patient',1,'view_history',NULL,'2026-10-12 06:13:02'),(20,1,1,'user',1,'update',NULL,'2026-10-12 06:13:02'),(21,1,1,'clinic',1,'update',NULL,'2026-10-12 06:13:02'),(22,1,1,'user',1,'login',NULL,'2026-10-12 06:13:09'),(23,1,1,'prescription',2,'save',NULL,'2026-10-12 06:13:09'),(24,1,1,'user',1,'login',NULL,'2026-10-12 06:16:56'),(25,1,1,'patient',1,'view_history',NULL,'2026-10-12 06:16:56'),(26,1,1,'medical_profile',1,'view',NULL,'2026-10-12 06:16:56'),(27,1,1,'patient',1,'view',NULL,'2026-10-12 06:16:56'),(28,1,1,'visit',3,'create',NULL,'2026-10-12 06:16:56'),(29,1,1,'patient',1,'view_history',NULL,'2026-10-12 06:16:56'),(30,1,1,'prescription',3,'view',NULL,'2026-10-12 06:16:56'),(31,1,1,'visit',3,'view',NULL,'2026-10-12 06:16:56'),(32,1,1,'prescription',3,'save',NULL,'2026-10-12 06:16:58'),(33,1,1,'patient',1,'view',NULL,'2026-10-12 06:16:58'),(34,1,1,'patient',1,'view_history',NULL,'2026-10-12 06:16:58'),(35,1,1,'medical_profile',1,'view',NULL,'2026-10-12 06:16:58'),(36,1,1,'user',1,'login',NULL,'2026-10-12 06:17:34'),(37,1,1,'patient',1,'view',NULL,'2026-10-12 06:17:34'),(38,1,1,'medical_profile',1,'view',NULL,'2026-10-12 06:17:34'),(39,1,1,'patient',1,'view_history',NULL,'2026-10-12 06:17:34'),(40,1,1,'visit',3,'view',NULL,'2026-10-12 06:17:34'),(41,1,1,'prescription',3,'view',NULL,'2026-10-12 06:17:34'),(42,1,1,'prescription',3,'lock',NULL,'2026-10-12 06:17:34'),(43,1,1,'visit',3,'view',NULL,'2026-10-12 06:17:34'),(44,1,4,'user',4,'login',NULL,'2026-10-12 07:02:14'),(45,1,5,'user',5,'login',NULL,'2026-10-12 07:02:14'),(46,1,5,'patient',3,'create',NULL,'2026-10-12 07:02:14'),(47,1,5,'patient',4,'create',NULL,'2026-10-12 07:02:14'),(48,1,4,'medical_profile',3,'update',NULL,'2026-10-12 07:02:14'),(49,1,5,'visit',4,'create',NULL,'2026-10-12 07:02:15'),(50,1,5,'visit',5,'create',NULL,'2026-10-12 07:02:15'),(51,1,5,'visit',5,'reorder',NULL,'2026-10-12 07:02:15'),(52,1,5,'vitals',4,'save',NULL,'2026-10-12 07:02:15'),(53,1,5,'vitals',4,'save',NULL,'2026-10-12 07:02:15'),(54,1,4,'prescription',4,'save',NULL,'2026-10-12 07:02:15'),(55,1,4,'prescription',4,'allergy_override',NULL,'2026-10-12 07:02:15'),(56,1,4,'visit',4,'seen',NULL,'2026-10-12 07:02:15'),(57,1,4,'visit',5,'in_consultation',NULL,'2026-10-12 07:02:15'),(58,1,5,'patient',3,'view_vitals',NULL,'2026-10-12 07:02:15'),(59,1,4,'patient',3,'view_history',NULL,'2026-10-12 07:02:15'),(60,1,4,'user',4,'login',NULL,'2026-10-12 07:07:32'),(61,1,4,'user',4,'login',NULL,'2026-10-12 07:07:48'),(62,1,4,'visit',4,'view',NULL,'2026-10-12 07:07:48'),(63,1,4,'patient',3,'view_history',NULL,'2026-10-12 07:07:48'),(64,1,4,'prescription',4,'view',NULL,'2026-10-12 07:07:49'),(65,1,4,'patient',3,'view_history',NULL,'2026-10-12 07:07:53'),(66,1,4,'patient',3,'view',NULL,'2026-10-12 07:07:53'),(67,1,4,'medical_profile',3,'view',NULL,'2026-10-12 07:07:53'),(68,1,4,'patient',3,'view_vitals',NULL,'2026-10-12 07:07:55'),(69,1,4,'user',4,'login',NULL,'2026-10-12 07:08:11'),(70,1,4,'prescription',1,'view',NULL,'2026-10-12 07:08:11'),(71,1,4,'visit',1,'view',NULL,'2026-10-12 07:08:11'),(72,1,4,'patient',1,'view_history',NULL,'2026-10-12 07:08:11'),(73,1,4,'user',4,'login',NULL,'2026-10-12 07:08:27'),(74,1,4,'prescription',1,'view',NULL,'2026-10-12 07:08:27'),(75,1,4,'visit',1,'view',NULL,'2026-10-12 07:08:27'),(76,1,4,'patient',1,'view_history',NULL,'2026-10-12 07:08:27'),(77,1,4,'user',4,'login',NULL,'2026-10-12 08:51:31'),(78,1,6,'user',6,'login',NULL,'2026-10-12 08:52:18'),(79,1,7,'user',7,'login',NULL,'2026-10-12 08:52:19'),(80,1,8,'user',8,'login',NULL,'2026-10-12 08:52:19'),(81,1,8,'user',8,'login',NULL,'2026-10-12 08:52:38'),(82,1,6,'user',6,'login',NULL,'2026-10-12 08:53:14'),(83,1,6,'patient',4,'view',NULL,'2026-10-12 08:53:19'),(84,1,6,'medical_profile',4,'view',NULL,'2026-10-12 08:53:19'),(85,1,6,'patient',4,'view_history',NULL,'2026-10-12 08:53:19'),(86,1,6,'patient',4,'view_vitals',NULL,'2026-10-12 08:53:22'),(87,1,6,'prescription',5,'view',NULL,'2026-10-12 08:53:25'),(88,1,6,'visit',5,'view',NULL,'2026-10-12 08:53:25'),(89,1,6,'prescription',5,'save',NULL,'2026-10-12 08:54:08'),(90,1,6,'patient',4,'view_history',NULL,'2026-10-12 08:54:08'),(91,1,6,'visit',5,'seen',NULL,'2026-10-12 08:54:08'),(92,1,6,'visit',1,'in_consultation',NULL,'2026-10-12 08:54:08'),(93,1,6,'patient',4,'view_history',NULL,'2026-10-12 08:54:08'),(94,1,6,'visit',1,'view',NULL,'2026-10-12 08:54:08'),(95,1,6,'prescription',1,'view',NULL,'2026-10-12 08:54:08'),(96,1,6,'patient',1,'view_history',NULL,'2026-10-12 08:54:08'),(97,1,6,'visit',1,'seen',NULL,'2026-10-12 08:54:13'),(98,1,6,'patient',1,'view_history',NULL,'2026-10-12 08:54:13'),(99,1,6,'patient',4,'view_history',NULL,'2026-10-12 08:54:36'),(100,1,6,'visit',5,'view',NULL,'2026-10-12 08:54:36'),(101,1,6,'prescription',2,'view',NULL,'2026-10-12 08:54:41'),(102,1,6,'visit',2,'view',NULL,'2026-10-12 08:54:41'),(103,1,7,'user',7,'login',NULL,'2026-10-12 08:55:21'),(104,1,7,'patient',4,'view',NULL,'2026-10-12 08:55:37'),(105,1,7,'medical_profile',4,'view',NULL,'2026-10-12 08:55:37'),(106,1,7,'patient',4,'view_history',NULL,'2026-10-12 08:55:37'),(107,1,7,'patient',1,'view',NULL,'2026-10-12 09:01:33'),(108,1,7,'medical_profile',1,'view',NULL,'2026-10-12 09:01:33'),(109,1,7,'patient',1,'view_history',NULL,'2026-10-12 09:01:33'),(110,1,7,'visit',6,'create',NULL,'2026-10-12 09:01:42'),(111,1,7,'patient',1,'view_history',NULL,'2026-10-12 09:01:42'),(112,1,7,'patient',1,'view',NULL,'2026-10-12 09:05:07'),(113,1,7,'medical_profile',1,'view',NULL,'2026-10-12 09:05:07'),(114,1,7,'patient',1,'view_history',NULL,'2026-10-12 09:05:07'),(115,1,7,'patient',1,'view_vitals',NULL,'2026-10-12 09:05:10'),(116,1,6,'user',6,'login',NULL,'2026-10-12 09:06:16'),(117,1,7,'user',7,'login',NULL,'2026-10-12 09:06:17'),(118,1,6,'clinic_settings',1,'update',NULL,'2026-10-12 09:06:17'),(119,1,7,'appointment',1,'create',NULL,'2026-10-12 09:06:17'),(120,1,7,'appointment',2,'create',NULL,'2026-10-12 09:06:17'),(121,1,6,'user',6,'login',NULL,'2026-10-12 09:06:28'),(122,1,7,'user',7,'login',NULL,'2026-10-12 09:06:29'),(123,1,7,'appointment',1,'reschedule',NULL,'2026-10-12 09:06:45'),(124,1,7,'visit',7,'create',NULL,'2026-10-12 09:06:45'),(125,1,7,'appointment',2,'arrived',NULL,'2026-10-12 09:06:45'),(126,1,7,'appointment',1,'no_show',NULL,'2026-10-12 09:06:45'),(127,1,7,'appointment',1,'booked',NULL,'2026-10-12 09:06:45'),(128,1,6,'user',6,'login',NULL,'2026-10-12 09:07:12'),(129,1,7,'user',7,'login',NULL,'2026-10-12 09:07:13'),(130,1,7,'bill',1,'create',NULL,'2026-10-12 09:07:29'),(131,1,7,'bill',1,'update',NULL,'2026-10-12 09:07:29'),(132,1,6,'bill',2,'create',NULL,'2026-10-12 09:07:29'),(133,1,6,'user',6,'login',NULL,'2026-10-12 09:07:55'),(134,1,7,'user',7,'login',NULL,'2026-10-12 09:07:55'),(135,1,7,'sms',7,'queue',NULL,'2026-10-12 09:07:55'),(136,1,7,'sms',7,'retry',NULL,'2026-10-12 09:08:11'),(137,1,7,'followup',4,'came',NULL,'2026-10-12 09:08:11'),(138,1,7,'bill',1,'update',NULL,'2026-10-12 09:08:12'),(139,1,7,'bill',1,'update',NULL,'2026-10-12 09:08:12'),(140,1,6,'bill',2,'update',NULL,'2026-10-12 09:08:12'),(141,1,7,'patient',1,'view',NULL,'2026-10-12 09:11:33'),(142,1,7,'medical_profile',1,'view',NULL,'2026-10-12 09:11:33'),(143,1,7,'patient',1,'view_history',NULL,'2026-10-12 09:11:33'),(144,1,7,'medical_profile',1,'view',NULL,'2026-10-12 09:11:46'),(145,1,7,'patient',1,'view',NULL,'2026-10-12 09:11:46'),(146,1,7,'patient',1,'view_history',NULL,'2026-10-12 09:11:46'),(147,1,7,'patient',1,'view_vitals',NULL,'2026-10-12 09:11:49'),(148,1,7,'patient',1,'view',NULL,'2026-10-12 09:13:23'),(149,1,7,'medical_profile',1,'view',NULL,'2026-10-12 09:13:24'),(150,1,7,'patient',1,'view_history',NULL,'2026-10-12 09:13:24'),(151,1,7,'patient',1,'view',NULL,'2026-10-12 09:14:01'),(152,1,7,'medical_profile',1,'view',NULL,'2026-10-12 09:14:01'),(153,1,7,'patient',1,'view_history',NULL,'2026-10-12 09:14:01'),(154,1,4,'user',4,'login',NULL,'2026-10-12 09:14:46'),(155,1,4,'user',4,'login',NULL,'2026-10-12 09:15:27'),(156,1,4,'user',4,'login',NULL,'2026-10-12 09:16:00'),(157,1,4,'user',4,'login',NULL,'2026-10-12 09:16:20'),(158,1,7,'patient',1,'view',NULL,'2026-10-12 09:29:15'),(159,1,7,'medical_profile',1,'view',NULL,'2026-10-12 09:29:15'),(160,1,7,'patient',1,'view_history',NULL,'2026-10-12 09:29:15'),(161,1,7,'patient',1,'view',NULL,'2026-10-12 09:39:23'),(162,1,7,'medical_profile',1,'view',NULL,'2026-10-12 09:39:23'),(163,1,7,'patient',1,'view_history',NULL,'2026-10-12 09:39:23'),(164,1,7,'visit',7,'reorder',NULL,'2026-10-12 09:39:54'),(165,1,7,'visit',6,'reorder',NULL,'2026-10-12 09:39:55'),(166,1,7,'visit',7,'reorder',NULL,'2026-10-12 09:39:57'),(167,1,7,'patient',4,'view',NULL,'2026-10-12 09:40:37'),(168,1,7,'medical_profile',4,'view',NULL,'2026-10-12 09:40:37'),(169,1,7,'patient',4,'view_history',NULL,'2026-10-12 09:40:37'),(170,1,7,'visit',6,'reorder',NULL,'2026-10-12 09:41:09'),(171,1,7,'visit',6,'reorder',NULL,'2026-10-12 09:41:11'),(172,1,6,'user',6,'login',NULL,'2026-10-12 09:44:13'),(173,1,7,'user',7,'login',NULL,'2026-10-12 09:44:14'),(174,1,6,'report',1,'upload',NULL,'2026-10-12 09:44:14'),(175,1,6,'report',1,'view',NULL,'2026-10-12 09:44:14'),(176,1,6,'report',1,'verify',NULL,'2026-10-12 09:44:14'),(177,1,6,'ai_analysis',2,'create',NULL,'2026-10-12 09:44:14'),(178,1,6,'ai_suggestion',1,'accepted',NULL,'2026-10-12 09:44:14'),(179,1,6,'ai_suggestion',2,'rejected',NULL,'2026-10-12 09:44:14'),(180,1,6,'ai_summary',1,'create',NULL,'2026-10-12 09:44:15'),(181,1,7,'patient',4,'view',NULL,'2026-10-12 09:49:57'),(182,1,7,'patient',4,'view_history',NULL,'2026-10-12 09:49:57'),(183,1,7,'medical_profile',4,'view',NULL,'2026-10-12 09:49:57'),(184,1,4,'user',4,'login',NULL,'2026-10-12 09:50:06'),(185,1,4,'prescription',4,'view',NULL,'2026-10-12 09:50:06'),(186,1,4,'visit',4,'view',NULL,'2026-10-12 09:50:06'),(187,1,4,'patient',3,'view_history',NULL,'2026-10-12 09:50:06'),(188,1,4,'prescription',4,'view',NULL,'2026-10-12 09:50:13'),(189,1,4,'visit',4,'view',NULL,'2026-10-12 09:50:13'),(190,1,4,'patient',3,'view_history',NULL,'2026-10-12 09:50:13'),(191,1,4,'patient',3,'view',NULL,'2026-10-12 09:50:17'),(192,1,4,'medical_profile',3,'view',NULL,'2026-10-12 09:50:17'),(193,1,4,'patient',3,'view_history',NULL,'2026-10-12 09:50:17'),(194,1,4,'user',4,'login',NULL,'2026-10-12 09:51:14'),(195,1,4,'prescription',5,'view',NULL,'2026-10-12 09:51:14'),(196,1,4,'visit',5,'view',NULL,'2026-10-12 09:51:14'),(197,1,4,'patient',4,'view_history',NULL,'2026-10-12 09:51:14'),(198,1,4,'report',2,'upload',NULL,'2026-10-12 09:51:18'),(199,1,7,'patient',1,'view',NULL,'2026-10-12 09:51:24'),(200,1,7,'medical_profile',1,'view',NULL,'2026-10-12 09:51:24'),(201,1,7,'patient',1,'view_history',NULL,'2026-10-12 09:51:24'),(202,1,4,'ai_analysis',10,'create',NULL,'2026-10-12 09:51:26'),(203,1,4,'ai_suggestion',11,'accepted',NULL,'2026-10-12 09:51:31'),(204,1,4,'ai_suggestion',12,'accepted',NULL,'2026-10-12 09:51:31'),(205,1,4,'ai_suggestion',13,'accepted',NULL,'2026-10-12 09:51:31'),(206,1,4,'ai_suggestion',14,'accepted',NULL,'2026-10-12 09:51:31'),(207,1,4,'ai_suggestion',10,'accepted',NULL,'2026-10-12 09:51:31'),(208,1,7,'bill',3,'create',NULL,'2026-10-12 09:51:33'),(209,1,7,'bill',3,'update',NULL,'2026-10-12 09:51:43'),(210,1,7,'patient',3,'view',NULL,'2026-10-12 09:51:53'),(211,1,7,'medical_profile',3,'view',NULL,'2026-10-12 09:51:53'),(212,1,7,'patient',3,'view_history',NULL,'2026-10-12 09:51:53'),(213,1,4,'user',4,'login',NULL,'2026-10-12 09:51:55'),(214,1,4,'prescription',5,'view',NULL,'2026-10-12 09:51:56'),(215,1,4,'visit',5,'view',NULL,'2026-10-12 09:51:56'),(216,1,4,'patient',4,'view_history',NULL,'2026-10-12 09:51:56'),(217,1,4,'user',4,'login',NULL,'2026-10-12 09:52:11'),(218,1,4,'prescription',5,'view',NULL,'2026-10-12 09:52:12'),(219,1,4,'visit',5,'view',NULL,'2026-10-12 09:52:12'),(220,1,4,'patient',4,'view_history',NULL,'2026-10-12 09:52:12'),(221,1,7,'patient',4,'view',NULL,'2026-10-12 09:56:31'),(222,1,7,'medical_profile',4,'view',NULL,'2026-10-12 09:56:31'),(223,1,7,'patient',4,'view_history',NULL,'2026-10-12 09:56:31'),(224,1,7,'patient',4,'view_vitals',NULL,'2026-10-12 09:56:38'),(225,1,7,'report',2,'view',NULL,'2026-10-12 09:56:42'),(226,1,6,'user',6,'login',NULL,'2026-10-12 09:57:02'),(227,1,7,'user',7,'login',NULL,'2026-10-12 09:57:02'),(228,1,8,'user',8,'login',NULL,'2026-10-12 09:57:03'),(229,1,6,'rx_template',1,'save',NULL,'2026-10-12 09:57:03'),(230,1,6,'rx_template',1,'save',NULL,'2026-10-12 09:57:03'),(231,1,8,'medicine_catalogue',1,'import',NULL,'2026-10-12 09:57:03'),(232,1,8,'medicine_catalogue',1,'import',NULL,'2026-10-12 09:57:03'),(233,1,6,'document',1,'sick_leave',NULL,'2026-10-12 09:57:03'),(234,1,6,'document',2,'referral',NULL,'2026-10-12 09:57:03'),(235,1,6,'user',6,'login',NULL,'2026-10-12 09:57:16'),(236,1,8,'user',8,'login',NULL,'2026-10-12 09:57:16'),(237,1,6,'patient',5,'create',NULL,'2026-10-12 09:57:16'),(238,1,6,'patient',6,'create',NULL,'2026-10-12 09:57:16'),(239,1,6,'medical_profile',5,'update',NULL,'2026-10-12 09:57:16'),(240,1,6,'medical_profile',6,'update',NULL,'2026-10-12 09:57:16'),(241,1,6,'visit',8,'create',NULL,'2026-10-12 09:57:16'),(242,1,6,'visit',9,'create',NULL,'2026-10-12 09:57:16'),(243,1,8,'patient',6,'merged_into',NULL,'2026-10-12 09:57:16'),(244,1,8,'patient',5,'merged_from',NULL,'2026-10-12 09:57:16'),(245,1,6,'patient',5,'view_history',NULL,'2026-10-12 09:57:16'),(246,1,6,'medical_profile',5,'view',NULL,'2026-10-12 09:57:16'),(247,1,4,'user',4,'login',NULL,'2026-10-12 10:01:41'),(248,1,4,'prescription',5,'view',NULL,'2026-10-12 10:01:41'),(249,1,4,'visit',5,'view',NULL,'2026-10-12 10:01:41'),(250,1,4,'patient',4,'view_history',NULL,'2026-10-12 10:01:41'),(251,1,4,'prescription',5,'view',NULL,'2026-10-12 10:01:52'),(252,1,4,'visit',5,'view',NULL,'2026-10-12 10:01:52'),(253,1,4,'patient',4,'view_history',NULL,'2026-10-12 10:01:52'),(254,1,4,'patient',4,'view',NULL,'2026-10-12 10:01:58'),(255,1,4,'medical_profile',4,'view',NULL,'2026-10-12 10:01:58'),(256,1,4,'patient',4,'view_history',NULL,'2026-10-12 10:01:58'),(257,1,4,'user',4,'login',NULL,'2026-10-12 10:02:27'),(258,1,4,'prescription',5,'view',NULL,'2026-10-12 10:02:27'),(259,1,4,'visit',5,'view',NULL,'2026-10-12 10:02:27'),(260,1,4,'patient',4,'view_history',NULL,'2026-10-12 10:02:27'),(261,1,4,'medical_profile',1,'view',NULL,'2026-10-12 10:02:33'),(262,1,4,'patient',1,'view_history',NULL,'2026-10-12 10:02:33'),(263,1,4,'patient',1,'view',NULL,'2026-10-12 10:02:33'),(264,1,8,'user',8,'login',NULL,'2026-10-12 10:02:44'),(265,1,8,'patient',5,'view',NULL,'2026-10-12 10:02:44'),(266,1,4,'user',4,'login',NULL,'2026-10-12 10:03:14'),(267,1,4,'visit',5,'view',NULL,'2026-10-12 10:03:14'),(268,1,4,'prescription',5,'view',NULL,'2026-10-12 10:03:14'),(269,1,4,'patient',4,'view_history',NULL,'2026-10-12 10:03:14'),(270,1,4,'document',3,'sick_leave',NULL,'2026-10-12 10:03:20'),(271,1,8,'user',8,'login',NULL,'2026-10-12 10:03:26'),(272,1,8,'patient',1,'view',NULL,'2026-10-12 10:03:26'),(273,1,6,'user',6,'login',NULL,'2026-10-12 10:04:30'),(274,1,1,'user',1,'login',NULL,'2026-10-13 03:27:30'),(275,1,2,'user',2,'login',NULL,'2026-10-13 03:27:30'),(276,1,3,'user',3,'login',NULL,'2026-10-13 03:27:31'),(277,1,6,'user',6,'login',NULL,'2026-10-13 03:27:48'),(278,1,6,'patient',5,'view',NULL,'2026-10-13 03:28:48'),(279,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:28:48'),(280,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:28:48'),(281,1,6,'visit',10,'create',NULL,'2026-10-13 03:29:00'),(282,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:29:00'),(283,1,6,'prescription',10,'view',NULL,'2026-10-13 03:29:00'),(284,1,6,'visit',10,'view',NULL,'2026-10-13 03:29:00'),(285,1,6,'rx_template',3,'save',NULL,'2026-10-13 03:29:12'),(286,1,6,'rx_template',3,'save',NULL,'2026-10-13 03:29:14'),(287,1,6,'rx_template',3,'save',NULL,'2026-10-13 03:29:15'),(288,1,6,'rx_template',6,'save',NULL,'2026-10-13 03:29:18'),(289,1,6,'prescription',10,'save',NULL,'2026-10-13 03:29:31'),(290,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:29:31'),(291,1,6,'prescription',10,'lock',NULL,'2026-10-13 03:29:31'),(292,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:29:31'),(293,1,6,'visit',10,'view',NULL,'2026-10-13 03:29:31'),(294,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:31:49'),(295,1,6,'visit',10,'view',NULL,'2026-10-13 03:31:49'),(296,1,6,'report',3,'upload',NULL,'2026-10-13 03:35:32'),(297,1,6,'report',4,'upload',NULL,'2026-10-13 03:35:32'),(298,1,6,'report',5,'upload',NULL,'2026-10-13 03:35:32'),(299,1,6,'report',6,'upload',NULL,'2026-10-13 03:35:33'),(300,1,6,'patient',5,'view',NULL,'2026-10-13 03:36:57'),(301,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:36:57'),(302,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:36:57'),(303,1,6,'medical_profile',5,'update',NULL,'2026-10-13 03:37:08'),(304,1,6,'medical_profile',5,'update',NULL,'2026-10-13 03:37:13'),(305,1,6,'medical_profile',5,'update',NULL,'2026-10-13 03:37:30'),(306,1,6,'visit',8,'view',NULL,'2026-10-13 03:37:47'),(307,1,6,'prescription',8,'view',NULL,'2026-10-13 03:37:47'),(308,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:37:47'),(309,1,6,'patient',5,'view',NULL,'2026-10-13 03:37:49'),(310,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:38:16'),(311,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:38:55'),(312,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:38:55'),(313,1,6,'patient',5,'view',NULL,'2026-10-13 03:38:55'),(314,1,6,'patient',5,'view',NULL,'2026-10-13 03:39:04'),(315,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:39:04'),(316,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:39:04'),(317,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:39:35'),(318,1,6,'patient',5,'view',NULL,'2026-10-13 03:39:35'),(319,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:39:35'),(320,1,6,'patient',5,'view',NULL,'2026-10-13 03:39:35'),(321,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:39:35'),(322,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:39:35'),(323,1,6,'patient',5,'view_vitals',NULL,'2026-10-13 03:39:58'),(324,1,6,'patient',5,'view',NULL,'2026-10-13 03:40:49'),(325,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:40:49'),(326,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:40:49'),(327,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:41:24'),(328,1,6,'patient',5,'view',NULL,'2026-10-13 03:41:24'),(329,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:41:24'),(330,1,6,'user',6,'login',NULL,'2026-10-13 03:41:50'),(331,1,6,'prescription',4,'view',NULL,'2026-10-13 03:41:53'),(332,1,6,'visit',4,'view',NULL,'2026-10-13 03:41:53'),(333,1,6,'patient',3,'view_history',NULL,'2026-10-13 03:41:53'),(334,1,6,'patient',5,'view_vitals',NULL,'2026-10-13 03:42:12'),(335,1,6,'user',6,'login',NULL,'2026-10-13 03:42:20'),(336,1,6,'user',6,'login',NULL,'2026-10-13 03:42:50'),(337,1,6,'patient',1,'view',NULL,'2026-10-13 03:42:59'),(338,1,6,'medical_profile',1,'view',NULL,'2026-10-13 03:42:59'),(339,1,6,'patient',1,'view_history',NULL,'2026-10-13 03:42:59'),(340,1,6,'prescription',4,'view',NULL,'2026-10-13 03:43:03'),(341,1,6,'visit',4,'view',NULL,'2026-10-13 03:43:03'),(342,1,6,'patient',3,'view_history',NULL,'2026-10-13 03:43:03'),(343,1,6,'user',6,'login',NULL,'2026-10-13 03:43:54'),(344,1,6,'prescription',4,'view',NULL,'2026-10-13 03:43:58'),(345,1,6,'visit',4,'view',NULL,'2026-10-13 03:43:58'),(346,1,6,'patient',3,'view_history',NULL,'2026-10-13 03:43:58'),(347,1,6,'medical_profile',1,'view',NULL,'2026-10-13 03:44:05'),(348,1,6,'patient',1,'view',NULL,'2026-10-13 03:44:05'),(349,1,6,'patient',1,'view_history',NULL,'2026-10-13 03:44:05'),(350,1,6,'user',6,'login',NULL,'2026-10-13 03:45:11'),(351,1,6,'prescription',4,'view',NULL,'2026-10-13 03:45:16'),(352,1,6,'visit',4,'view',NULL,'2026-10-13 03:45:16'),(353,1,6,'patient',3,'view_history',NULL,'2026-10-13 03:45:16'),(354,1,6,'prescription',4,'view',NULL,'2026-10-13 03:45:24'),(355,1,6,'visit',4,'view',NULL,'2026-10-13 03:45:24'),(356,1,6,'patient',3,'view_history',NULL,'2026-10-13 03:45:24'),(357,1,6,'medical_profile',1,'view',NULL,'2026-10-13 03:45:29'),(358,1,6,'patient',1,'view',NULL,'2026-10-13 03:45:29'),(359,1,6,'patient',1,'view_history',NULL,'2026-10-13 03:45:29'),(360,1,6,'patient',5,'view',NULL,'2026-10-13 03:46:07'),(361,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:46:07'),(362,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:46:07'),(363,1,6,'patient',5,'view_vitals',NULL,'2026-10-13 03:46:10'),(364,1,6,'visit',11,'create',NULL,'2026-10-13 03:46:27'),(365,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:46:27'),(366,1,6,'prescription',11,'view',NULL,'2026-10-13 03:46:27'),(367,1,6,'visit',11,'view',NULL,'2026-10-13 03:46:27'),(368,1,6,'prescription',11,'save',NULL,'2026-10-13 03:46:58'),(369,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:46:58'),(370,1,6,'visit',11,'seen',NULL,'2026-10-13 03:46:58'),(371,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:46:58'),(372,1,6,'bill',4,'create',NULL,'2026-10-13 03:47:07'),(373,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:47:18'),(374,1,6,'patient',5,'view',NULL,'2026-10-13 03:47:18'),(375,1,6,'visit',11,'view',NULL,'2026-10-13 03:47:24'),(376,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:47:40'),(377,1,6,'report',6,'view',NULL,'2026-10-13 03:49:15'),(378,1,6,'patient',5,'view',NULL,'2026-10-13 03:49:19'),(379,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:49:19'),(380,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:49:19'),(381,1,6,'patient',5,'view_vitals',NULL,'2026-10-13 03:49:23'),(382,1,6,'visit',11,'view',NULL,'2026-10-13 03:49:28'),(383,1,6,'prescription',11,'view',NULL,'2026-10-13 03:50:36'),(384,1,6,'visit',11,'view',NULL,'2026-10-13 03:50:36'),(385,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:50:36'),(386,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:51:12'),(387,1,6,'visit',11,'view',NULL,'2026-10-13 03:51:12'),(388,1,6,'prescription',11,'view',NULL,'2026-10-13 03:51:12'),(389,1,6,'visit',11,'view',NULL,'2026-10-13 03:51:12'),(390,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:51:13'),(391,1,6,'user',6,'login',NULL,'2026-10-13 03:51:27'),(392,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:51:27'),(393,1,6,'patient',5,'view',NULL,'2026-10-13 03:51:27'),(394,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:51:27'),(395,1,6,'patient',1,'view',NULL,'2026-10-13 03:51:30'),(396,1,6,'medical_profile',1,'view',NULL,'2026-10-13 03:51:30'),(397,1,6,'patient',1,'view_history',NULL,'2026-10-13 03:51:30'),(398,1,6,'patient',1,'view',NULL,'2026-10-13 03:51:38'),(399,1,6,'medical_profile',1,'view',NULL,'2026-10-13 03:51:38'),(400,1,6,'patient',1,'view_history',NULL,'2026-10-13 03:51:38'),(401,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:51:59'),(402,1,6,'visit',11,'view',NULL,'2026-10-13 03:51:59'),(403,1,6,'patient',5,'view',NULL,'2026-10-13 03:52:12'),(404,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:52:12'),(405,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:52:12'),(406,1,6,'medical_profile',5,'update',NULL,'2026-10-13 03:52:22'),(407,1,6,'medical_profile',5,'update',NULL,'2026-10-13 03:52:28'),(408,1,6,'patient',5,'view_vitals',NULL,'2026-10-13 03:52:36'),(409,1,6,'patient',5,'view',NULL,'2026-10-13 03:54:41'),(410,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:54:41'),(411,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:54:41'),(412,1,6,'medical_profile',5,'update',NULL,'2026-10-13 03:55:51'),(413,1,6,'medical_profile',5,'update',NULL,'2026-10-13 03:55:53'),(414,1,6,'medical_profile',5,'update',NULL,'2026-10-13 03:55:55'),(415,1,6,'medical_profile',5,'update',NULL,'2026-10-13 03:56:04'),(416,1,6,'medical_profile',5,'update',NULL,'2026-10-13 03:56:06'),(417,1,6,'medical_profile',5,'update',NULL,'2026-10-13 03:56:12'),(418,1,6,'report',6,'view',NULL,'2026-10-13 03:56:16'),(419,1,6,'patient',5,'view',NULL,'2026-10-13 03:56:16'),(420,1,6,'report',5,'view',NULL,'2026-10-13 03:56:16'),(421,1,6,'report',4,'view',NULL,'2026-10-13 03:56:16'),(422,1,6,'report',3,'view',NULL,'2026-10-13 03:56:16'),(423,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:56:16'),(424,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:56:16'),(425,1,6,'patient',5,'view',NULL,'2026-10-13 03:56:16'),(426,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:56:16'),(427,1,6,'report',6,'view',NULL,'2026-10-13 03:56:16'),(428,1,6,'report',5,'view',NULL,'2026-10-13 03:56:16'),(429,1,6,'report',3,'view',NULL,'2026-10-13 03:56:16'),(430,1,6,'report',4,'view',NULL,'2026-10-13 03:56:16'),(431,1,6,'user',6,'login',NULL,'2026-10-13 03:56:34'),(432,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:56:34'),(433,1,6,'patient',5,'view',NULL,'2026-10-13 03:56:34'),(434,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:56:34'),(435,1,6,'report',6,'view',NULL,'2026-10-13 03:56:34'),(436,1,6,'report',5,'view',NULL,'2026-10-13 03:56:34'),(437,1,6,'report',4,'view',NULL,'2026-10-13 03:56:34'),(438,1,6,'report',3,'view',NULL,'2026-10-13 03:56:34'),(439,1,6,'patient',3,'view',NULL,'2026-10-13 03:56:45'),(440,1,6,'medical_profile',3,'view',NULL,'2026-10-13 03:56:45'),(441,1,6,'patient',3,'view_history',NULL,'2026-10-13 03:56:45'),(442,1,6,'report',1,'view',NULL,'2026-10-13 03:56:47'),(443,1,6,'patient',5,'view_vitals',NULL,'2026-10-13 03:57:06'),(444,1,6,'report',6,'view',NULL,'2026-10-13 03:57:13'),(445,1,6,'patient',5,'view',NULL,'2026-10-13 03:57:15'),(446,1,6,'medical_profile',5,'view',NULL,'2026-10-13 03:57:15'),(447,1,6,'patient',5,'view_history',NULL,'2026-10-13 03:57:15'),(448,1,6,'patient',7,'create',NULL,'2026-10-13 03:59:33'),(449,1,6,'medical_profile',7,'view',NULL,'2026-10-13 03:59:34'),(450,1,6,'patient',7,'view',NULL,'2026-10-13 03:59:34'),(451,1,6,'patient',7,'view_history',NULL,'2026-10-13 03:59:34'),(452,1,6,'patient',7,'view',NULL,'2026-10-13 03:59:35'),(453,1,6,'medical_profile',7,'view',NULL,'2026-10-13 03:59:35'),(454,1,6,'patient',7,'view_history',NULL,'2026-10-13 03:59:35'),(455,1,6,'patient',7,'view_vitals',NULL,'2026-10-13 03:59:42'),(456,1,6,'user',6,'login',NULL,'2026-10-13 04:00:07'),(457,1,6,'patient',5,'view',NULL,'2026-10-13 04:00:07'),(458,1,6,'medical_profile',5,'view',NULL,'2026-10-13 04:00:07'),(459,1,6,'patient',5,'view_history',NULL,'2026-10-13 04:00:07'),(460,1,6,'report',6,'view',NULL,'2026-10-13 04:00:07'),(461,1,6,'report',5,'view',NULL,'2026-10-13 04:00:07'),(462,1,6,'report',4,'view',NULL,'2026-10-13 04:00:07'),(463,1,6,'report',3,'view',NULL,'2026-10-13 04:00:07'),(464,1,6,'medical_profile',7,'view',NULL,'2026-10-13 04:00:26'),(465,1,6,'patient',7,'view',NULL,'2026-10-13 04:00:26'),(466,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:00:26'),(467,1,6,'visit',12,'create',NULL,'2026-10-13 04:00:41'),(468,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:00:41'),(469,1,6,'prescription',12,'view',NULL,'2026-10-13 04:00:41'),(470,1,6,'visit',12,'view',NULL,'2026-10-13 04:00:41'),(471,1,6,'prescription',12,'save',NULL,'2026-10-13 04:01:07'),(472,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:01:07'),(473,1,6,'visit',12,'seen',NULL,'2026-10-13 04:01:07'),(474,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:01:07'),(475,1,6,'visit',12,'view',NULL,'2026-10-13 04:01:12'),(476,1,6,'prescription',12,'save',NULL,'2026-10-13 04:02:09'),(477,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:02:09'),(478,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:02:47'),(479,1,6,'visit',12,'view',NULL,'2026-10-13 04:02:47'),(480,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:03:27'),(481,1,6,'visit',12,'view',NULL,'2026-10-13 04:03:27'),(482,1,6,'visit',12,'seen',NULL,'2026-10-13 04:03:40'),(483,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:03:40'),(484,1,6,'patient',7,'view',NULL,'2026-10-13 04:03:47'),(485,1,6,'medical_profile',7,'view',NULL,'2026-10-13 04:03:47'),(486,1,6,'visit',12,'view',NULL,'2026-10-13 04:04:04'),(487,1,6,'report',7,'upload',NULL,'2026-10-13 04:04:22'),(488,1,6,'report',8,'upload',NULL,'2026-10-13 04:04:22'),(489,1,6,'report',9,'upload',NULL,'2026-10-13 04:04:22'),(490,1,6,'report',10,'upload',NULL,'2026-10-13 04:04:22'),(491,1,6,'prescription',12,'view',NULL,'2026-10-13 04:04:29'),(492,1,6,'visit',12,'view',NULL,'2026-10-13 04:04:29'),(493,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:04:29'),(494,1,6,'medical_profile',7,'view',NULL,'2026-10-13 04:04:42'),(495,1,6,'patient',7,'view',NULL,'2026-10-13 04:04:42'),(496,1,6,'report',8,'view',NULL,'2026-10-13 04:04:42'),(497,1,6,'report',10,'view',NULL,'2026-10-13 04:04:42'),(498,1,6,'report',9,'view',NULL,'2026-10-13 04:04:42'),(499,1,6,'report',7,'view',NULL,'2026-10-13 04:04:42'),(500,1,6,'visit',13,'create',NULL,'2026-10-13 04:05:05'),(501,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:05:05'),(502,1,6,'prescription',13,'view',NULL,'2026-10-13 04:05:05'),(503,1,6,'visit',13,'view',NULL,'2026-10-13 04:05:05'),(504,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:07:08'),(505,1,6,'visit',13,'view',NULL,'2026-10-13 04:07:08'),(506,1,6,'prescription',13,'view',NULL,'2026-10-13 04:07:08'),(507,1,6,'visit',13,'view',NULL,'2026-10-13 04:07:08'),(508,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:07:08'),(509,1,6,'patient',7,'view',NULL,'2026-10-13 04:07:13'),(510,1,6,'medical_profile',7,'view',NULL,'2026-10-13 04:07:13'),(511,1,6,'report',8,'view',NULL,'2026-10-13 04:07:13'),(512,1,6,'report',9,'view',NULL,'2026-10-13 04:07:13'),(513,1,6,'report',10,'view',NULL,'2026-10-13 04:07:13'),(514,1,6,'report',7,'view',NULL,'2026-10-13 04:07:13'),(515,1,6,'user',6,'login',NULL,'2026-10-13 04:07:27'),(516,1,6,'prescription',13,'view',NULL,'2026-10-13 04:07:27'),(517,1,6,'visit',13,'view',NULL,'2026-10-13 04:07:27'),(518,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:07:27'),(519,1,6,'report',7,'view',NULL,'2026-10-13 04:07:29'),(520,1,6,'report',10,'view',NULL,'2026-10-13 04:07:29'),(521,1,6,'report',8,'view',NULL,'2026-10-13 04:07:29'),(522,1,6,'report',9,'view',NULL,'2026-10-13 04:07:29'),(523,1,6,'patient',7,'view',NULL,'2026-10-13 04:13:35'),(524,1,6,'medical_profile',7,'view',NULL,'2026-10-13 04:13:35'),(525,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:13:35'),(526,1,6,'prescription',13,'view',NULL,'2026-10-13 04:13:38'),(527,1,6,'visit',13,'view',NULL,'2026-10-13 04:13:38'),(528,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:14:12'),(529,1,6,'visit',13,'view',NULL,'2026-10-13 04:14:12'),(530,1,6,'user',6,'login',NULL,'2026-10-13 04:15:30'),(531,1,6,'prescription',10,'view',NULL,'2026-10-13 04:15:30'),(532,1,6,'visit',10,'view',NULL,'2026-10-13 04:15:30'),(533,1,6,'patient',5,'view_history',NULL,'2026-10-13 04:15:30'),(534,1,6,'report',6,'view',NULL,'2026-10-13 04:15:33'),(535,1,6,'report',3,'view',NULL,'2026-10-13 04:15:33'),(536,1,6,'report',5,'view',NULL,'2026-10-13 04:15:33'),(537,1,6,'report',4,'view',NULL,'2026-10-13 04:15:33'),(538,1,6,'prescription',13,'view',NULL,'2026-10-13 04:16:32'),(539,1,6,'visit',13,'view',NULL,'2026-10-13 04:16:32'),(540,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:16:32'),(541,1,6,'report',7,'view',NULL,'2026-10-13 04:16:35'),(542,1,6,'report',9,'view',NULL,'2026-10-13 04:16:35'),(543,1,6,'report',8,'view',NULL,'2026-10-13 04:16:35'),(544,1,6,'report',10,'view',NULL,'2026-10-13 04:16:35'),(545,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:18:33'),(546,1,6,'visit',13,'view',NULL,'2026-10-13 04:18:33'),(547,1,6,'prescription',13,'view',NULL,'2026-10-13 04:19:08'),(548,1,6,'visit',13,'view',NULL,'2026-10-13 04:19:08'),(549,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:19:08'),(550,1,6,'visit',13,'view',NULL,'2026-10-13 04:19:52'),(551,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:19:52'),(552,1,6,'prescription',13,'view',NULL,'2026-10-13 04:20:23'),(553,1,6,'visit',13,'view',NULL,'2026-10-13 04:20:23'),(554,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:20:23'),(555,1,6,'user',6,'login',NULL,'2026-10-13 04:21:07'),(556,1,6,'clinic_settings',1,'update',NULL,'2026-10-13 04:21:13'),(557,1,6,'prescription',13,'view',NULL,'2026-10-13 04:21:16'),(558,1,6,'visit',13,'view',NULL,'2026-10-13 04:21:16'),(559,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:21:16'),(560,1,6,'user',6,'login',NULL,'2026-10-13 04:21:25'),(561,1,6,'user',6,'login',NULL,'2026-10-13 04:21:50'),(562,1,6,'prescription',13,'view',NULL,'2026-10-13 04:21:50'),(563,1,6,'visit',13,'view',NULL,'2026-10-13 04:21:50'),(564,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:21:50'),(565,1,6,'user',6,'login',NULL,'2026-10-13 04:22:27'),(566,1,6,'clinic_settings',1,'update',NULL,'2026-10-13 04:22:35'),(567,1,6,'user',6,'login',NULL,'2026-10-13 04:22:39'),(568,1,6,'user',6,'login',NULL,'2026-10-13 04:22:52'),(569,1,6,'prescription',13,'view',NULL,'2026-10-13 04:22:52'),(570,1,6,'visit',13,'view',NULL,'2026-10-13 04:22:52'),(571,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:22:52'),(572,1,6,'visit',13,'view',NULL,'2026-10-13 04:23:23'),(573,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:23:23'),(574,1,6,'prescription',13,'view',NULL,'2026-10-13 04:23:25'),(575,1,6,'visit',13,'view',NULL,'2026-10-13 04:23:25'),(576,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:23:25'),(577,1,6,'prescription',13,'view',NULL,'2026-10-13 04:23:47'),(578,1,6,'visit',13,'view',NULL,'2026-10-13 04:23:47'),(579,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:23:47'),(580,1,6,'patient',7,'view',NULL,'2026-10-13 04:24:46'),(581,1,6,'medical_profile',7,'view',NULL,'2026-10-13 04:24:46'),(582,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:24:46'),(583,1,6,'report',10,'view',NULL,'2026-10-13 04:24:46'),(584,1,6,'report',9,'view',NULL,'2026-10-13 04:24:46'),(585,1,6,'report',8,'view',NULL,'2026-10-13 04:24:46'),(586,1,6,'report',7,'view',NULL,'2026-10-13 04:24:46'),(587,1,6,'visit',13,'view',NULL,'2026-10-13 04:25:02'),(588,1,6,'prescription',13,'save',NULL,'2026-10-13 04:25:09'),(589,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:25:09'),(590,1,6,'prescription',13,'save',NULL,'2026-10-13 04:25:32'),(591,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:25:32'),(592,1,6,'user',6,'login',NULL,'2026-10-13 04:27:08'),(593,1,6,'medical_profile',7,'view',NULL,'2026-10-13 04:27:14'),(594,1,6,'patient',7,'view',NULL,'2026-10-13 04:27:14'),(595,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:27:14'),(596,1,6,'report',9,'view',NULL,'2026-10-13 04:27:14'),(597,1,6,'report',8,'view',NULL,'2026-10-13 04:27:14'),(598,1,6,'report',10,'view',NULL,'2026-10-13 04:27:14'),(599,1,6,'report',7,'view',NULL,'2026-10-13 04:27:14'),(600,1,6,'medical_profile',4,'view',NULL,'2026-10-13 04:28:06'),(601,1,6,'patient',4,'view',NULL,'2026-10-13 04:28:06'),(602,1,6,'patient',4,'view_history',NULL,'2026-10-13 04:28:06'),(603,1,6,'report',2,'view',NULL,'2026-10-13 04:28:06'),(604,1,6,'medical_profile',7,'view',NULL,'2026-10-13 04:29:43'),(605,1,6,'patient',7,'view',NULL,'2026-10-13 04:29:43'),(606,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:29:43'),(607,1,6,'report',10,'view',NULL,'2026-10-13 04:29:43'),(608,1,6,'report',9,'view',NULL,'2026-10-13 04:29:43'),(609,1,6,'report',8,'view',NULL,'2026-10-13 04:29:43'),(610,1,6,'report',7,'view',NULL,'2026-10-13 04:29:43'),(611,1,6,'prescription',13,'view',NULL,'2026-10-13 04:29:46'),(612,1,6,'visit',13,'view',NULL,'2026-10-13 04:29:46'),(613,1,6,'prescription',13,'view',NULL,'2026-10-13 04:30:07'),(614,1,6,'visit',13,'view',NULL,'2026-10-13 04:30:07'),(615,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:30:07'),(616,1,6,'prescription',13,'lock',NULL,'2026-10-13 04:30:32'),(617,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:30:32'),(618,1,6,'visit',13,'view',NULL,'2026-10-13 04:30:32'),(619,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:31:11'),(620,1,6,'visit',13,'view',NULL,'2026-10-13 04:31:11'),(621,1,6,'user',6,'login',NULL,'2026-10-13 04:32:53'),(622,1,7,'user',7,'login',NULL,'2026-10-13 04:32:53'),(623,1,6,'medicine',2,'create',NULL,'2026-10-13 04:32:53'),(624,1,6,'medicine',2,'update',NULL,'2026-10-13 04:32:53'),(625,1,6,'medicine',2,'deactivate',NULL,'2026-10-13 04:32:53'),(626,1,6,'medicine',2,'delete',NULL,'2026-10-13 04:32:54'),(627,1,6,'prescription',13,'view',NULL,'2026-10-13 04:33:24'),(628,1,6,'visit',13,'view',NULL,'2026-10-13 04:33:24'),(629,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:33:24'),(630,1,6,'report',9,'view',NULL,'2026-10-13 04:33:48'),(631,1,6,'report',8,'view',NULL,'2026-10-13 04:33:48'),(632,1,6,'report',7,'view',NULL,'2026-10-13 04:33:48'),(633,1,6,'report',10,'view',NULL,'2026-10-13 04:33:48'),(634,1,6,'prescription',13,'view',NULL,'2026-10-13 04:34:11'),(635,1,6,'visit',13,'view',NULL,'2026-10-13 04:34:11'),(636,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:34:11'),(637,1,6,'user',6,'login',NULL,'2026-10-13 04:34:34'),(638,1,6,'medicine',3,'create',NULL,'2026-10-13 04:34:44'),(639,1,6,'user',6,'login',NULL,'2026-10-13 04:35:13'),(640,1,6,'medicine_catalogue',1,'import',NULL,'2026-10-13 04:35:16'),(641,1,6,'user',6,'login',NULL,'2026-10-13 04:35:55'),(642,1,6,'user',6,'login',NULL,'2026-10-13 04:36:14'),(643,1,6,'medicine',5,'deactivate',NULL,'2026-10-13 04:36:17'),(644,1,6,'medicine',7,'delete',NULL,'2026-10-13 04:36:22'),(645,1,6,'user',6,'login',NULL,'2026-10-13 04:36:56'),(646,1,6,'user',6,'login',NULL,'2026-10-13 04:38:55'),(647,1,6,'medical_profile',7,'view',NULL,'2026-10-13 04:39:22'),(648,1,6,'patient',7,'view',NULL,'2026-10-13 04:39:22'),(649,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:39:22'),(650,1,6,'report',10,'view',NULL,'2026-10-13 04:39:22'),(651,1,6,'report',9,'view',NULL,'2026-10-13 04:39:22'),(652,1,6,'report',8,'view',NULL,'2026-10-13 04:39:22'),(653,1,6,'report',7,'view',NULL,'2026-10-13 04:39:22'),(654,1,6,'user',6,'login',NULL,'2026-10-13 04:39:46'),(655,1,6,'medicine',8,'create',NULL,'2026-10-13 04:39:51'),(656,1,6,'user',6,'login',NULL,'2026-10-13 04:42:44'),(657,1,6,'user',6,'login',NULL,'2026-10-13 04:44:16'),(658,1,6,'patient',7,'view',NULL,'2026-10-13 04:44:50'),(659,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:44:50'),(660,1,6,'medical_profile',7,'view',NULL,'2026-10-13 04:44:50'),(661,1,6,'report',10,'view',NULL,'2026-10-13 04:44:50'),(662,1,6,'report',9,'view',NULL,'2026-10-13 04:44:50'),(663,1,6,'report',7,'view',NULL,'2026-10-13 04:44:50'),(664,1,6,'report',8,'view',NULL,'2026-10-13 04:44:50'),(665,1,6,'user',6,'login',NULL,'2026-10-13 04:45:01'),(666,1,6,'patient',7,'view',NULL,'2026-10-13 04:46:14'),(667,1,6,'medical_profile',7,'view',NULL,'2026-10-13 04:46:14'),(668,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:46:14'),(669,1,6,'report',9,'view',NULL,'2026-10-13 04:46:14'),(670,1,6,'report',8,'view',NULL,'2026-10-13 04:46:14'),(671,1,6,'report',10,'view',NULL,'2026-10-13 04:46:14'),(672,1,6,'report',7,'view',NULL,'2026-10-13 04:46:14'),(673,1,6,'patient',7,'view',NULL,'2026-10-13 04:52:09'),(674,1,6,'medical_profile',7,'view',NULL,'2026-10-13 04:52:09'),(675,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:52:09'),(676,1,6,'visit',14,'create',NULL,'2026-10-13 04:52:13'),(677,1,6,'patient',7,'view_history',NULL,'2026-10-13 04:52:13'),(678,1,6,'prescription',14,'view',NULL,'2026-10-13 04:52:13'),(679,1,6,'visit',14,'view',NULL,'2026-10-13 04:52:13');
/*!40000 ALTER TABLE `audit_logs` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `bills`
--

DROP TABLE IF EXISTS `bills`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `bills` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned NOT NULL,
  `visit_id` bigint unsigned NOT NULL,
  `receipt_no` varchar(20) DEFAULT NULL,
  `fee` decimal(10,2) NOT NULL DEFAULT '0.00',
  `discount` decimal(10,2) NOT NULL DEFAULT '0.00',
  `discount_reason` varchar(150) DEFAULT NULL,
  `paid` decimal(10,2) NOT NULL DEFAULT '0.00',
  `method` enum('cash','bkash','nagad','card') NOT NULL DEFAULT 'cash',
  `status` enum('paid','due') NOT NULL DEFAULT 'due',
  `created_by` bigint unsigned DEFAULT NULL,
  `updated_by` bigint unsigned DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_bills_visit` (`visit_id`),
  UNIQUE KEY `uq_bills_receipt` (`clinic_id`,`receipt_no`),
  CONSTRAINT `fk_bills_visit` FOREIGN KEY (`visit_id`) REFERENCES `visits` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=5 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `bills`
--

LOCK TABLES `bills` WRITE;
/*!40000 ALTER TABLE `bills` DISABLE KEYS */;
INSERT INTO `bills` VALUES (1,1,7,'R-2026-000001',800.00,100.00,'relative',700.00,'bkash','paid',7,7,'2026-10-12 09:07:29','2026-10-12 09:08:12'),(2,1,6,'R-2026-000002',500.00,0.00,NULL,500.00,'cash','paid',6,6,'2026-10-12 09:07:29','2026-10-12 09:08:12'),(3,1,5,'R-2026-000003',500.00,0.00,NULL,500.00,'cash','paid',7,7,'2026-10-12 09:51:33','2026-10-12 09:51:43'),(4,1,10,'R-2026-000004',500.00,0.00,NULL,500.00,'cash','paid',6,NULL,'2026-10-13 03:47:07','2026-10-13 03:47:07');
/*!40000 ALTER TABLE `bills` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `clinic_counters`
--

DROP TABLE IF EXISTS `clinic_counters`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `clinic_counters` (
  `clinic_id` bigint unsigned NOT NULL,
  `name` varchar(30) NOT NULL,
  `year` smallint unsigned NOT NULL,
  `value` bigint unsigned NOT NULL DEFAULT '0',
  PRIMARY KEY (`clinic_id`,`name`,`year`),
  CONSTRAINT `fk_counters_clinic` FOREIGN KEY (`clinic_id`) REFERENCES `clinics` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `clinic_counters`
--

LOCK TABLES `clinic_counters` WRITE;
/*!40000 ALTER TABLE `clinic_counters` DISABLE KEYS */;
INSERT INTO `clinic_counters` VALUES (1,'patient',2026,7),(1,'receipt',2026,4);
/*!40000 ALTER TABLE `clinic_counters` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `clinics`
--

DROP TABLE IF EXISTS `clinics`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `clinics` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(150) NOT NULL,
  `address` varchar(255) DEFAULT NULL,
  `phone` varchar(15) DEFAULT NULL,
  `logo_path` varchar(255) DEFAULT NULL,
  `settings` json DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `clinics`
--

LOCK TABLES `clinics` WRITE;
/*!40000 ALTER TABLE `clinics` DISABLE KEYS */;
INSERT INTO `clinics` VALUES (1,'Demo Chamber','Dhanmondi, Dhaka','01700000000',NULL,'{\"ai\": {\"enabled\": true, \"auto_extract\": true, \"advice_language\": \"both\"}, \"sms\": {\"enabled\": true, \"language\": \"bn\", \"templates\": {\"followup_reminder\": {\"bn\": \"প্রিয় {name}, {date} তারিখে {doctor}-এর কাছে আপনার ফলো-আপ ভিজিট। সিরিয়ালের জন্য কল করুন {phone}। {clinic}\", \"en\": \"Dear {name}, your follow-up visit with {doctor} is due on {date}. For a serial call {phone}. {clinic}\"}, \"appointment_confirm\": {\"bn\": \"প্রিয় {name}, {clinic}-এ {doctor}-এর সাথে আপনার অ্যাপয়েন্টমেন্ট {date}, {time}-এ নিশ্চিত হয়েছে।\", \"en\": \"Dear {name}, your appointment with {doctor} at {clinic} is booked for {date}, {time}.\"}, \"appointment_reminder\": {\"bn\": \"স্মরণিকা: {name}, আগামীকাল ({date}) {time}-এ {doctor}-এর সাথে আপনার অ্যাপয়েন্টমেন্ট। {clinic}\", \"en\": \"Reminder: {name}, you have an appointment with {doctor} tomorrow ({date}) at {time}. {clinic}\"}}, \"confirm_on_book\": true, \"remind_day_before\": true}, \"fees\": {\"new\": 800, \"report\": 300, \"followup\": 500}, \"rx_lists\": {\"advice\": [\"প্রচুর পানি পান করবেন\", \"বিশ্রাম নিবেন\", \"তৈলাক্ত ও ভাজা খাবার এড়িয়ে চলবেন\", \"সময়মতো খাবার খাবেন\", \"লবণ কম খাবেন\", \"মিষ্টি খাবার এড়িয়ে চলবেন\", \"ধূমপান করবেন না\", \"প্রতিদিন ৩০ মিনিট হাঁটবেন\", \"ওষুধের কোর্স সম্পূর্ণ করবেন\", \"জ্বর বেশি হলে মাথায় পানি দিবেন\"], \"diagnosis\": [\"Viral fever\", \"URTI\", \"LRTI\", \"Acute pharyngitis\", \"Allergic rhinitis\", \"Enteric fever\", \"Dengue fever\", \"Acute gastroenteritis\", \"Peptic ulcer disease\", \"GERD\", \"IBS\", \"UTI\", \"Hypertension\", \"Type 2 diabetes mellitus\", \"Bronchial asthma\", \"Iron deficiency anaemia\", \"Migraine\", \"Tension-type headache\", \"Osteoarthritis\", \"Low back pain\"], \"next_plan\": [\"Review after 7 days\", \"Review after 14 days\", \"Review after 1 month\", \"Review with reports\", \"Come back if symptoms get worse\"], \"complaints\": [\"Fever\", \"Cough\", \"Headache\", \"Sore throat\", \"Runny nose\", \"Body ache\", \"Weakness\", \"Dizziness\", \"Abdominal pain\", \"Vomiting\", \"Loose motion\", \"Constipation\", \"Loss of appetite\", \"Burning micturition\", \"Chest pain\", \"Shortness of breath\", \"Palpitation\", \"Joint pain\", \"Back pain\", \"Itching\", \"Skin rash\", \"Itchy eyes\", \"Ear pain\"], \"examination\": [\"Anaemia: absent\", \"Jaundice: absent\", \"Oedema: absent\", \"Cyanosis: absent\", \"Dehydration: absent\", \"Throat: congested\", \"Tonsils: enlarged\", \"Heart: S1 + S2 + 0\", \"Lungs: clear\", \"Lungs: crepitations present\", \"Lungs: rhonchi present\", \"Abdomen: soft, non-tender\", \"Abdomen: epigastric tenderness\", \"Liver: not palpable\", \"Spleen: not palpable\", \"Lymph nodes: not palpable\"], \"investigations\": [\"CBC\", \"ESR\", \"CRP\", \"Urine R/E\", \"Urine C/S\", \"RBS\", \"FBS\", \"2HABF\", \"HbA1c\", \"S. Creatinine\", \"S. Electrolytes\", \"SGPT\", \"Lipid profile\", \"TSH\", \"Widal test\", \"Dengue NS1 antigen\", \"Blood culture\", \"Chest X-ray P/A view\", \"ECG\", \"USG of whole abdomen\"]}, \"schedule\": [{\"end\": \"21:00\", \"start\": \"17:00\", \"weekday\": 0}, {\"end\": \"21:00\", \"start\": \"17:00\", \"weekday\": 1}, {\"end\": \"21:00\", \"start\": \"17:00\", \"weekday\": 2}, {\"end\": \"21:00\", \"start\": \"17:00\", \"weekday\": 3}, {\"end\": \"21:00\", \"start\": \"17:00\", \"weekday\": 4}, {\"end\": \"21:00\", \"start\": \"17:00\", \"weekday\": 6}], \"daily_limit\": 20, \"slot_minutes\": 10}','2026-10-12 05:58:35','2026-10-13 04:22:35');
/*!40000 ALTER TABLE `clinics` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `documents`
--

DROP TABLE IF EXISTS `documents`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `documents` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned NOT NULL,
  `patient_id` bigint unsigned NOT NULL,
  `visit_id` bigint unsigned NOT NULL,
  `kind` enum('sick_leave','fitness','referral') NOT NULL,
  `body` json NOT NULL,
  `created_by` bigint unsigned DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `ix_documents_visit` (`visit_id`),
  KEY `ix_documents_patient` (`patient_id`,`created_at`),
  CONSTRAINT `fk_documents_patient` FOREIGN KEY (`patient_id`) REFERENCES `patients` (`id`),
  CONSTRAINT `fk_documents_visit` FOREIGN KEY (`visit_id`) REFERENCES `visits` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `documents`
--

LOCK TABLES `documents` WRITE;
/*!40000 ALTER TABLE `documents` DISABLE KEYS */;
INSERT INTO `documents` VALUES (1,1,4,5,'sick_leave','{\"to\": \"2026-10-18\", \"from\": \"2026-10-12\", \"reason\": null, \"remarks\": \"Avoid strenuous activity\", \"summary\": null, \"diagnosis\": \"Enteric fever\", \"referred_to\": null}',6,'2026-10-12 09:57:03'),(2,1,4,5,'referral','{\"to\": null, \"from\": null, \"reason\": \"Raised creatinine 2.4 mg/dL\", \"remarks\": null, \"summary\": \"Fever 5 days, CBC leucocytosis\", \"diagnosis\": null, \"referred_to\": \"Nephrology, DMCH\"}',6,'2026-10-12 09:57:03'),(3,1,4,5,'sick_leave','{\"to\": \"2026-10-18\", \"from\": \"2026-10-12\", \"reason\": null, \"remarks\": null, \"summary\": null, \"diagnosis\": \"Enteric fever\", \"referred_to\": null}',4,'2026-10-12 10:03:20');
/*!40000 ALTER TABLE `documents` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `followups`
--

DROP TABLE IF EXISTS `followups`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `followups` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned NOT NULL,
  `patient_id` bigint unsigned NOT NULL,
  `prescription_id` bigint unsigned DEFAULT NULL,
  `due_date` date NOT NULL,
  `status` enum('pending','came','missed') NOT NULL DEFAULT 'pending',
  `reminder_sent_at` datetime DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `ix_followups_due` (`status`,`due_date`),
  KEY `fk_followups_patient` (`patient_id`),
  KEY `fk_followups_prescription` (`prescription_id`),
  KEY `ix_followups_clinic_due` (`clinic_id`,`status`,`due_date`),
  CONSTRAINT `fk_followups_patient` FOREIGN KEY (`patient_id`) REFERENCES `patients` (`id`),
  CONSTRAINT `fk_followups_prescription` FOREIGN KEY (`prescription_id`) REFERENCES `prescriptions` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=9 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `followups`
--

LOCK TABLES `followups` WRITE;
/*!40000 ALTER TABLE `followups` DISABLE KEYS */;
INSERT INTO `followups` VALUES (3,1,1,2,'2026-10-26','pending',NULL,'2026-10-12 06:13:09','2026-10-12 09:06:01'),(4,1,3,NULL,'2026-10-05','came','2026-10-12 09:08:22','2026-10-12 09:07:13','2026-10-12 09:08:22'),(5,1,4,NULL,'2026-10-12','pending',NULL,'2026-10-12 09:07:13','2026-10-12 09:07:13'),(6,1,2,NULL,'2026-10-15','pending',NULL,'2026-10-12 09:07:13','2026-10-12 09:07:13'),(7,1,5,7,'2026-10-14','pending',NULL,'2026-10-13 03:46:58','2026-10-13 03:46:58'),(8,1,7,9,'2026-11-05','pending',NULL,'2026-10-13 04:25:32','2026-10-13 04:25:32');
/*!40000 ALTER TABLE `followups` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `jobs`
--

DROP TABLE IF EXISTS `jobs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `jobs` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `type` varchar(50) NOT NULL,
  `payload` json NOT NULL,
  `status` enum('pending','running','done','failed') NOT NULL DEFAULT 'pending',
  `attempts` int unsigned NOT NULL DEFAULT '0',
  `run_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `last_error` text,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `ix_jobs_status_run` (`status`,`run_at`)
) ENGINE=InnoDB AUTO_INCREMENT=9 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `jobs`
--

LOCK TABLES `jobs` WRITE;
/*!40000 ALTER TABLE `jobs` DISABLE KEYS */;
INSERT INTO `jobs` VALUES (1,'sms_send','{\"sms_id\": 1}','done',1,'2026-10-12 09:08:32',NULL,'2026-10-12 09:06:17','2026-10-12 09:06:32'),(2,'sms_send','{\"sms_id\": 2}','done',1,'2026-10-12 09:08:32',NULL,'2026-10-12 09:06:17','2026-10-12 09:06:32'),(3,'sms_send','{\"sms_id\": 3}','done',1,'2026-10-12 09:08:47',NULL,'2026-10-12 09:06:45','2026-10-12 09:06:47'),(4,'sms_send','{\"sms_id\": 4}','pending',0,'2026-10-14 04:00:00',NULL,'2026-10-12 09:06:45','2026-10-12 09:06:45'),(5,'sms_send','{\"sms_id\": 5}','done',1,'2026-10-12 09:08:47',NULL,'2026-10-12 09:06:45','2026-10-12 09:06:47'),(6,'sms_send','{\"sms_id\": 6}','pending',0,'2026-10-14 04:00:00',NULL,'2026-10-12 09:06:45','2026-10-12 09:06:45'),(7,'sms_send','{\"sms_id\": 7}','done',1,'2026-10-12 09:10:07',NULL,'2026-10-12 09:07:55','2026-10-12 09:08:07'),(8,'sms_send','{\"sms_id\": 7}','done',1,'2026-10-12 09:10:22',NULL,'2026-10-12 09:08:11','2026-10-12 09:08:22');
/*!40000 ALTER TABLE `jobs` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `medicine_favorites`
--

DROP TABLE IF EXISTS `medicine_favorites`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `medicine_favorites` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` bigint unsigned NOT NULL,
  `line` json NOT NULL,
  `brand` varchar(150) NOT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `ix_favorites_user` (`user_id`,`brand`),
  CONSTRAINT `fk_favorites_user` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=5 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `medicine_favorites`
--

LOCK TABLES `medicine_favorites` WRITE;
/*!40000 ALTER TABLE `medicine_favorites` DISABLE KEYS */;
INSERT INTO `medicine_favorites` VALUES (4,6,'{\"form\": null, \"brand\": \"Napa\", \"notes\": null, \"timing\": \"খাবারের আগে\", \"generic\": null, \"duration\": \"৭ দিন\", \"strength\": \"500 mg\", \"medicine_id\": null, \"dose_pattern\": \"1+1+1\"}','Napa','2026-10-13 04:29:55');
/*!40000 ALTER TABLE `medicine_favorites` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `medicines`
--

DROP TABLE IF EXISTS `medicines`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `medicines` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned DEFAULT NULL,
  `brand` varchar(150) NOT NULL,
  `generic` varchar(255) NOT NULL,
  `strength` varchar(100) DEFAULT NULL,
  `form` varchar(50) DEFAULT NULL,
  `company` varchar(150) DEFAULT NULL,
  `price` decimal(10,2) DEFAULT NULL,
  `is_active` tinyint(1) NOT NULL DEFAULT '1',
  PRIMARY KEY (`id`),
  KEY `ix_medicines_brand` (`brand`),
  KEY `ix_medicines_clinic_brand` (`clinic_id`,`brand`),
  KEY `ix_medicines_generic` (`generic`),
  FULLTEXT KEY `ft_medicines` (`brand`,`generic`)
) ENGINE=InnoDB AUTO_INCREMENT=9 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `medicines`
--

LOCK TABLES `medicines` WRITE;
/*!40000 ALTER TABLE `medicines` DISABLE KEYS */;
INSERT INTO `medicines` VALUES (1,1,'Testbrand-A','Testgeneric Alpha2','10 mg',NULL,NULL,NULL,1),(3,1,'Demo-Cef','Cefixime','200 mg','Capsule',NULL,35.00,1),(4,1,'Demo-Para','Paracetamol','500 mg','Tablet','Demo Pharma',1.20,1),(5,1,'Demo-Ome','Omeprazole','20 mg','Capsule','Demo Pharma',6.00,0),(6,1,'Demo-Amox','Amoxicillin','500 mg','Capsule','Demo Pharma',10.00,1),(8,1,'Demo-Monas','Montelukast','10 mg','Tablet','Square Pharmaceuticals',NULL,1);
/*!40000 ALTER TABLE `medicines` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `patient_medical_profile`
--

DROP TABLE IF EXISTS `patient_medical_profile`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `patient_medical_profile` (
  `patient_id` bigint unsigned NOT NULL,
  `allergies` json DEFAULT NULL,
  `chronic_conditions` json DEFAULT NULL,
  `long_term_medicines` json DEFAULT NULL,
  `past_history` text,
  `family_history` text,
  `social_history` text,
  `gynae_obs_history` text,
  `pregnancy_status` varchar(30) DEFAULT NULL,
  `vaccination` json DEFAULT NULL,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`patient_id`),
  CONSTRAINT `fk_pmp_patient` FOREIGN KEY (`patient_id`) REFERENCES `patients` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `patient_medical_profile`
--

LOCK TABLES `patient_medical_profile` WRITE;
/*!40000 ALTER TABLE `patient_medical_profile` DISABLE KEYS */;
INSERT INTO `patient_medical_profile` VALUES (1,'[\"Penicillin\"]','[]','[]','Appendectomy 2019',NULL,NULL,NULL,NULL,'[]','2026-10-12 06:13:02'),(2,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,'2026-10-12 05:58:48'),(3,'[\"Penicillin\"]','[\"Hypertension\"]','[]',NULL,NULL,NULL,NULL,NULL,'[]','2026-10-12 09:07:13'),(4,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,'2026-10-12 07:02:14'),(5,'[\"Penicillin\", \"Sulfa\", \"DTM\"]','[\"Hypertension\", \"IHD\", \"Asthma\"]','[\"ss\", \"Napa\"]','Typhoid 2019\nAppendectomy','ss\nss\ns','s',NULL,NULL,'[\"ss\"]','2026-10-13 03:56:11'),(7,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,'2026-10-13 03:59:33');
/*!40000 ALTER TABLE `patient_medical_profile` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `patients`
--

DROP TABLE IF EXISTS `patients`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `patients` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned NOT NULL,
  `patient_code` varchar(20) NOT NULL,
  `title` varchar(10) DEFAULT NULL,
  `name_en` varchar(150) NOT NULL,
  `name_bn` varchar(150) DEFAULT NULL,
  `phone` varchar(15) NOT NULL,
  `dob` date NOT NULL,
  `dob_estimated` tinyint(1) NOT NULL DEFAULT '0',
  `sex` enum('M','F','O') NOT NULL,
  `blood_group` varchar(5) DEFAULT NULL,
  `marital_status` varchar(20) DEFAULT NULL,
  `address` varchar(255) DEFAULT NULL,
  `district` varchar(100) DEFAULT NULL,
  `nid` varbinary(255) DEFAULT NULL COMMENT 'Encrypted at application level',
  `guardian_name` varchar(150) DEFAULT NULL,
  `guardian_phone` varchar(15) DEFAULT NULL,
  `referred_by` varchar(150) DEFAULT NULL,
  `photo_path` varchar(255) DEFAULT NULL,
  `created_by` bigint unsigned DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `deleted_at` datetime DEFAULT NULL,
  `merged_into` bigint unsigned DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_patients_code` (`patient_code`),
  KEY `ix_patients_phone` (`clinic_id`,`phone`),
  KEY `ix_patients_name` (`clinic_id`,`name_en`),
  FULLTEXT KEY `ft_patients_name` (`name_en`,`name_bn`) /*!50100 WITH PARSER `ngram` */ ,
  CONSTRAINT `fk_patients_clinic` FOREIGN KEY (`clinic_id`) REFERENCES `clinics` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=8 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `patients`
--

LOCK TABLES `patients` WRITE;
/*!40000 ALTER TABLE `patients` DISABLE KEYS */;
INSERT INTO `patients` VALUES (1,1,'P-2026-000001','Ms','Maisha Rahman',NULL,'01712345678','2008-03-01',0,'F','B+',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,1,'2026-10-12 05:58:48','2026-10-12 06:13:02',NULL,NULL),(2,1,'P-2026-000002',NULL,'Rahim Uddin',NULL,'01712345678','1981-05-02',0,'M',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,1,'2026-10-12 05:58:48','2026-10-12 05:58:48',NULL,NULL),(3,1,'P-2026-000003',NULL,'Smoke Allergic',NULL,'01711000001','1996-10-12',1,'F',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,5,'2026-10-12 07:02:14','2026-10-12 07:02:14',NULL,NULL),(4,1,'P-2026-000004',NULL,'Smoke Second',NULL,'01711000002','1986-10-12',1,'M',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,5,'2026-10-12 07:02:14','2026-10-12 07:02:14',NULL,NULL),(5,1,'P-2026-000005','Mr','Merge Test Keep',NULL,'01799000001','1976-10-12',1,'M',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,6,'2026-10-12 09:57:16','2026-10-12 09:57:16',NULL,NULL),(6,1,'P-2026-000006','Mr','Merge Test Dup',NULL,'01799000001','1976-10-12',1,'M',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,6,'2026-10-12 09:57:16','2026-10-12 09:57:16','2026-10-12 09:57:16',5),(7,1,'P-2026-000007','Mr','Md. Omar Faruk',NULL,'01839707645','1994-10-13',1,'M','B+',NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,6,'2026-10-13 03:59:33','2026-10-13 03:59:33',NULL,NULL);
/*!40000 ALTER TABLE `patients` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `prescription_medicines`
--

DROP TABLE IF EXISTS `prescription_medicines`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `prescription_medicines` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `prescription_id` bigint unsigned NOT NULL,
  `sort_order` smallint unsigned NOT NULL DEFAULT '0',
  `medicine_id` bigint unsigned DEFAULT NULL,
  `brand` varchar(150) NOT NULL,
  `generic` varchar(255) DEFAULT NULL,
  `strength` varchar(100) DEFAULT NULL,
  `form` varchar(50) DEFAULT NULL,
  `dose_pattern` varchar(30) DEFAULT NULL,
  `timing` varchar(100) DEFAULT NULL,
  `duration` varchar(50) DEFAULT NULL,
  `notes` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `ix_pm_prescription` (`prescription_id`,`sort_order`),
  KEY `fk_pm_medicine` (`medicine_id`),
  CONSTRAINT `fk_pm_medicine` FOREIGN KEY (`medicine_id`) REFERENCES `medicines` (`id`),
  CONSTRAINT `fk_pm_prescription` FOREIGN KEY (`prescription_id`) REFERENCES `prescriptions` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=14 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `prescription_medicines`
--

LOCK TABLES `prescription_medicines` WRITE;
/*!40000 ALTER TABLE `prescription_medicines` DISABLE KEYS */;
INSERT INTO `prescription_medicines` VALUES (2,1,0,NULL,'Napa',NULL,'500 mg',NULL,NULL,NULL,NULL,NULL),(3,1,1,NULL,'Furotil Plus',NULL,NULL,NULL,NULL,NULL,NULL,NULL),(5,2,0,NULL,'Napa',NULL,'500 mg',NULL,NULL,NULL,NULL,NULL),(6,3,0,NULL,'Napa',NULL,'500 mg',NULL,'1+1+1',NULL,NULL,NULL),(7,3,1,NULL,'Penicillin V',NULL,NULL,NULL,NULL,NULL,NULL,NULL),(8,4,0,NULL,'Flubex','Flucloxacillin',NULL,NULL,'1+1+1',NULL,NULL,NULL),(9,5,0,NULL,'Napa',NULL,'500 mg',NULL,'1+0+0','খাবারের সাথে','৫ দিন',NULL),(10,7,0,NULL,'Napa',NULL,'500 mg',NULL,'1+0+0','খাবারের সাথে','৭ দিন',NULL),(11,8,0,NULL,'Napa',NULL,'500 mg',NULL,'1+1+1','খাবারের আগে','৭ দিন',NULL),(13,9,0,NULL,'Napa',NULL,'500 mg',NULL,'1+1+1','খাবারের আগে','৭ দিন',NULL);
/*!40000 ALTER TABLE `prescription_medicines` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `prescriptions`
--

DROP TABLE IF EXISTS `prescriptions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `prescriptions` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `visit_id` bigint unsigned NOT NULL,
  `version` int unsigned NOT NULL DEFAULT '1',
  `revision` int unsigned NOT NULL DEFAULT '0',
  `complaints` json DEFAULT NULL,
  `examination` json DEFAULT NULL,
  `investigations` json DEFAULT NULL,
  `diagnosis` json DEFAULT NULL,
  `treatment_plan` json DEFAULT NULL,
  `advice` json DEFAULT NULL,
  `next_plan` json DEFAULT NULL,
  `operation_notes` text,
  `next_visit_date` date DEFAULT NULL,
  `locked` tinyint(1) NOT NULL DEFAULT '0',
  `pdf_path` varchar(255) DEFAULT NULL,
  `created_by` bigint unsigned DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_prescriptions_version` (`visit_id`,`version`),
  CONSTRAINT `fk_prescriptions_visit` FOREIGN KEY (`visit_id`) REFERENCES `visits` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=10 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `prescriptions`
--

LOCK TABLES `prescriptions` WRITE;
/*!40000 ALTER TABLE `prescriptions` DISABLE KEYS */;
INSERT INTO `prescriptions` VALUES (1,2,1,1,'[]','[]','[]','[\"UTI\"]','[]','[]','[]',NULL,'2026-10-20',1,NULL,1,'2026-10-12 06:13:02','2026-10-12 06:13:02'),(2,2,2,1,'[]','[]','[]','[\"UTI\"]','[]','[]','[]',NULL,'2026-10-26',0,NULL,1,'2026-10-12 06:13:02','2026-10-12 06:13:09'),(3,3,1,0,'[\"Fever 2 days\"]','[]','[]','[\"Pharyngitis\"]','[]','[]','[]',NULL,NULL,1,NULL,1,'2026-10-12 06:16:58','2026-10-12 06:17:34'),(4,4,1,0,'[]','[]','[]','[]','[]','[]','[]',NULL,NULL,0,NULL,4,'2026-10-12 07:02:15','2026-10-12 07:02:15'),(5,5,1,0,'[]','[\"asdf\", \"saasdf\"]','[]','[]','[]','[]','[]',NULL,NULL,0,NULL,6,'2026-10-12 08:54:08','2026-10-12 08:54:08'),(6,10,1,0,'[]','[]','[]','[\"Acute cystitis\"]','[]','[]','[]',NULL,NULL,1,NULL,6,'2026-10-13 03:29:31','2026-10-13 03:29:31'),(7,11,1,0,'[]','[]','[]','[]','[]','[]','[]',NULL,'2026-10-14',0,NULL,6,'2026-10-13 03:46:58','2026-10-13 03:46:58'),(8,12,1,1,'[\"test\"]','[\"test2\"]','[\"22\"]','[\"22\"]','[]','[]','[]',NULL,NULL,0,NULL,6,'2026-10-13 04:01:07','2026-10-13 04:02:09'),(9,13,1,1,'[]','[]','[]','[]','[]','[\"সময়মতো খাবার খাবেন\", \"ধূমপান করবেন না\"]','[\"Review after 14 days\", \"Review after 1 month\"]',NULL,'2026-11-05',1,NULL,6,'2026-10-13 04:25:09','2026-10-13 04:30:32');
/*!40000 ALTER TABLE `prescriptions` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `prompt_versions`
--

DROP TABLE IF EXISTS `prompt_versions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `prompt_versions` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(50) NOT NULL,
  `version` varchar(20) NOT NULL,
  `system_prompt` mediumtext NOT NULL,
  `schema` json DEFAULT NULL,
  `is_active` tinyint(1) NOT NULL DEFAULT '0',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_prompt_versions` (`name`,`version`)
) ENGINE=InnoDB AUTO_INCREMENT=61 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `prompt_versions`
--

LOCK TABLES `prompt_versions` WRITE;
/*!40000 ALTER TABLE `prompt_versions` DISABLE KEYS */;
INSERT INTO `prompt_versions` VALUES (1,'extraction','v1','You read laboratory and imaging reports (printed, scanned or photographed; English or Bangla) and record what is written.\nRules:\n1. Record every test result exactly as printed: test name, value, unit, reference range, and the lab\'s own H/L flag if printed.\n2. Never calculate, correct or guess a value. If a value or range is not legible, leave it out and list the part in unreadable_parts.\n3. Give a confidence for each result: high (clearly printed), medium (slightly unclear), low (hard to read).\n4. Identify the report type (e.g. CBC, Urine R/E, LFT, RFT, Lipid profile, Thyroid, HbA1c, Blood sugar, Electrolytes, USG, X-ray, ECG, Culture & Sensitivity).\n5. For narrative reports (USG, X-ray, CT, ECG) put the impression / conclusion text in impression, and key findings as results with the finding as the value.\n6. Dates as YYYY-MM-DD. patient_name_on_report is the name exactly as printed, or null.\n7. Answer only through the provided tool.','{\"type\": \"object\", \"required\": [\"report_type\", \"results\"], \"properties\": {\"results\": {\"type\": \"array\", \"items\": {\"type\": \"object\", \"required\": [\"test\", \"value\", \"confidence\"], \"properties\": {\"test\": {\"type\": \"string\"}, \"unit\": {\"type\": [\"string\", \"null\"]}, \"range\": {\"type\": [\"string\", \"null\"]}, \"value\": {\"type\": \"string\"}, \"lab_flag\": {\"type\": [\"string\", \"null\"], \"description\": \"H, L or empty as printed\"}, \"confidence\": {\"enum\": [\"high\", \"medium\", \"low\"], \"type\": \"string\"}}}}, \"lab_name\": {\"type\": [\"string\", \"null\"]}, \"impression\": {\"type\": [\"string\", \"null\"]}, \"report_date\": {\"type\": [\"string\", \"null\"], \"description\": \"YYYY-MM-DD\"}, \"report_type\": {\"type\": \"string\"}, \"unreadable_parts\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"patient_name_on_report\": {\"type\": [\"string\", \"null\"]}}}',1,'2026-10-12 09:43:46'),(2,'clinical','v1','You are a clinical decision-support assistant for a registered doctor in Bangladesh.\nRules:\n1. Use only the data provided. Never invent, assume or fill in missing results.\n2. For every finding, cite the source test and value it is based on.\n3. Use cautious wording: \"suggests\", \"consistent with\", \"consider\". Never write \"the patient has\".\n4. Give likelihood as High, Moderate or Low — never percentages.\n5. Point out conflicts between the entered diagnosis and the reports, and abnormal findings the prescription does not address.\n6. Never prescribe new medicines or doses; only flag safety concerns.\n7. If data is unclear or insufficient, say so instead of guessing.\n8. Patient advice must be short and simple, in English and in Bangla (natural Bangla a patient understands).\n9. Answer only through the provided tool.','{\"type\": \"object\", \"required\": [\"summary\", \"findings\", \"possible_issues\", \"suggested_investigations\", \"advice_en\", \"advice_bn\", \"red_flags\", \"conflicts\"], \"properties\": {\"summary\": {\"type\": \"string\", \"description\": \"3-5 lines\"}, \"findings\": {\"type\": \"array\", \"items\": {\"type\": \"object\", \"required\": [\"text\", \"source\"], \"properties\": {\"text\": {\"type\": \"string\"}, \"source\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}}}}, \"advice_bn\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}, \"description\": \"Same items as advice_en, in Bangla, same order\"}, \"advice_en\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"conflicts\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"next_plan\": {\"type\": [\"string\", \"null\"]}, \"red_flags\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"trend_comments\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}, \"description\": \"Change since earlier results, if earlier results were given\"}, \"possible_issues\": {\"type\": \"array\", \"items\": {\"type\": \"object\", \"required\": [\"name\", \"reason\"], \"properties\": {\"name\": {\"type\": \"string\"}, \"reason\": {\"type\": \"string\"}, \"likelihood\": {\"enum\": [\"High\", \"Moderate\", \"Low\"], \"type\": \"string\"}}}}, \"report_summaries\": {\"type\": \"array\", \"items\": {\"type\": \"object\", \"required\": [\"report\", \"summary\"], \"properties\": {\"report\": {\"type\": \"string\"}, \"summary\": {\"type\": \"string\"}}}}, \"suggested_investigations\": {\"type\": \"array\", \"items\": {\"type\": \"object\", \"required\": [\"name\", \"reason\"], \"properties\": {\"name\": {\"type\": \"string\"}, \"reason\": {\"type\": \"string\"}}}}}}',1,'2026-10-12 09:43:46'),(3,'safety','v1','You check a doctor\'s prescription for safety problems, for a registered doctor in Bangladesh.\nCheck: drug-drug interactions among the medicines; doses or drugs needing caution given the lab values (kidney, liver, platelets, potassium, glucose); drug-condition contraindications (pregnancy, chronic conditions, age).\nRules:\n1. Use only the data given. Do not invent lab values or conditions.\n2. Report only clinically meaningful problems; say nothing about safe combinations.\n3. Each warning names the medicine(s) and the reason, cites the lab value or condition, and has a severity: high (avoid / change), moderate (monitor or adjust dose), low (be aware).\n4. Never suggest a new medicine or a specific dose; say what to review.\n5. Answer only through the provided tool.','{\"type\": \"object\", \"required\": [\"warnings\"], \"properties\": {\"warnings\": {\"type\": \"array\", \"items\": {\"type\": \"object\", \"required\": [\"medicines\", \"kind\", \"severity\", \"message\"], \"properties\": {\"kind\": {\"enum\": [\"interaction\", \"renal\", \"hepatic\", \"lab\", \"pregnancy\", \"condition\", \"age\"], \"type\": \"string\"}, \"source\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"message\": {\"type\": \"string\"}, \"severity\": {\"enum\": [\"high\", \"moderate\", \"low\"], \"type\": \"string\"}, \"medicines\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}}}}}}',1,'2026-10-12 09:43:46'),(4,'patient_summary','v1','You summarise a patient\'s medical history for their doctor in Bangladesh, before a consultation.\nRules:\n1. Use only the data given (visits, diagnoses, medicines, labs, vitals). Never invent anything.\n2. Be brief and clinical. Dates as Mon YYYY.\n3. Use cautious wording; never state a new diagnosis.\n4. Alerts must include every allergy given.\n5. points_to_check are concrete things the doctor may want to review today, based on the data.\n6. If an earlier summary is given, update it with the newer visits rather than repeating it.\n7. Answer only through the provided tool.','{\"type\": \"object\", \"required\": [\"one_line\", \"active_problems\", \"recurring_issues\", \"lab_trends\", \"medicines_history\", \"alerts\", \"points_to_check\"], \"properties\": {\"alerts\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"one_line\": {\"type\": \"string\"}, \"lab_trends\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"active_problems\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"points_to_check\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"recurring_issues\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"medicines_history\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}}}',1,'2026-10-12 09:43:46'),(5,'smart_search','v1','You turn a doctor\'s search request about their own patients into search filters.\nRules:\n1. Only fill filters the request asks for; leave the rest out.\n2. Dates as YYYY-MM-DD, worked out from today\'s date given in the message.\n3. Terms are short medical words as a doctor would write them in a prescription (e.g. \"UTI\", \"diabetes\", \"metformin\"); include common synonyms or abbreviations as separate terms.\n4. sex is M or F.\n5. Answer only through the provided tool.','{\"type\": \"object\", \"properties\": {\"sex\": {\"enum\": [\"M\", \"F\"], \"type\": \"string\"}, \"age_max\": {\"type\": \"integer\"}, \"age_min\": {\"type\": \"integer\"}, \"visited_to\": {\"type\": \"string\"}, \"visited_from\": {\"type\": \"string\"}, \"chronic_terms\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"medicine_terms\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"complaint_terms\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}, \"diagnosis_terms\": {\"type\": \"array\", \"items\": {\"type\": \"string\"}}}}',1,'2026-10-12 09:43:46'),(6,'sms_draft','v1','You write short, polite SMS reminders from a clinic in Bangladesh to a patient.\nRules:\n1. Keep every placeholder in curly braces, e.g. {name}, exactly as given; do not fill them in.\n2. Never mention a diagnosis, disease, test or medicine.\n3. One or two short sentences; Bangla SMS under 140 characters, English under 300.\n4. Write in the requested language only.\n5. Answer only through the provided tool.','{\"type\": \"object\", \"required\": [\"text\"], \"properties\": {\"text\": {\"type\": \"string\"}}}',1,'2026-10-12 09:43:46');
/*!40000 ALTER TABLE `prompt_versions` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `reference_ranges`
--

DROP TABLE IF EXISTS `reference_ranges`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `reference_ranges` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned DEFAULT NULL COMMENT 'NULL = system default',
  `test_code` varchar(50) NOT NULL,
  `sex` enum('M','F','ANY') NOT NULL DEFAULT 'ANY',
  `age_min_days` int unsigned NOT NULL DEFAULT '0',
  `age_max_days` int unsigned NOT NULL DEFAULT '54750',
  `low` decimal(12,4) DEFAULT NULL,
  `high` decimal(12,4) DEFAULT NULL,
  `critical_low` decimal(12,4) DEFAULT NULL,
  `critical_high` decimal(12,4) DEFAULT NULL,
  `unit` varchar(30) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `ix_ranges_lookup` (`test_code`,`sex`,`clinic_id`)
) ENGINE=InnoDB AUTO_INCREMENT=49 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `reference_ranges`
--

LOCK TABLES `reference_ranges` WRITE;
/*!40000 ALTER TABLE `reference_ranges` DISABLE KEYS */;
INSERT INTO `reference_ranges` VALUES (1,NULL,'HB','M',6570,54750,13.0000,17.0000,7.0000,20.0000,'g/dL'),(2,NULL,'HB','F',6570,54750,12.0000,15.5000,7.0000,20.0000,'g/dL'),(3,NULL,'WBC','ANY',6570,54750,4000.0000,11000.0000,2000.0000,30000.0000,'/cumm'),(4,NULL,'PLT','ANY',6570,54750,150000.0000,450000.0000,50000.0000,1000000.0000,'/cumm'),(5,NULL,'ESR','M',6570,54750,0.0000,15.0000,NULL,NULL,'mm/1st hr'),(6,NULL,'ESR','F',6570,54750,0.0000,20.0000,NULL,NULL,'mm/1st hr'),(7,NULL,'NEUT','ANY',6570,54750,40.0000,75.0000,NULL,NULL,'%'),(8,NULL,'LYMPH','ANY',6570,54750,20.0000,45.0000,NULL,NULL,'%'),(9,NULL,'MONO','ANY',6570,54750,2.0000,10.0000,NULL,NULL,'%'),(10,NULL,'EOS','ANY',6570,54750,1.0000,6.0000,NULL,NULL,'%'),(11,NULL,'PCV','M',6570,54750,40.0000,50.0000,NULL,NULL,'%'),(12,NULL,'PCV','F',6570,54750,36.0000,46.0000,NULL,NULL,'%'),(13,NULL,'MCV','ANY',6570,54750,80.0000,100.0000,NULL,NULL,'fL'),(14,NULL,'MCH','ANY',6570,54750,27.0000,32.0000,NULL,NULL,'pg'),(15,NULL,'MCHC','ANY',6570,54750,32.0000,36.0000,NULL,NULL,'g/dL'),(16,NULL,'RBS','ANY',6570,54750,3.9000,7.8000,2.8000,22.2000,'mmol/L'),(17,NULL,'FBS','ANY',6570,54750,3.9000,5.5000,2.8000,22.2000,'mmol/L'),(18,NULL,'PPBS','ANY',6570,54750,3.9000,7.8000,2.8000,22.2000,'mmol/L'),(19,NULL,'HBA1C','ANY',6570,54750,4.0000,5.6000,NULL,NULL,'%'),(20,NULL,'CREAT','M',6570,54750,0.7000,1.3000,NULL,NULL,'mg/dL'),(21,NULL,'CREAT','F',6570,54750,0.5000,1.1000,NULL,NULL,'mg/dL'),(22,NULL,'UREA','ANY',6570,54750,15.0000,40.0000,NULL,NULL,'mg/dL'),(23,NULL,'NA','ANY',6570,54750,135.0000,145.0000,120.0000,160.0000,'mmol/L'),(24,NULL,'K','ANY',6570,54750,3.5000,5.1000,2.5000,6.0000,'mmol/L'),(25,NULL,'CL','ANY',6570,54750,98.0000,107.0000,NULL,NULL,'mmol/L'),(26,NULL,'CA','ANY',6570,54750,8.5000,10.5000,6.5000,13.0000,'mg/dL'),(27,NULL,'URIC','M',6570,54750,3.4000,7.0000,NULL,NULL,'mg/dL'),(28,NULL,'URIC','F',6570,54750,2.4000,6.0000,NULL,NULL,'mg/dL'),(29,NULL,'ALT','ANY',6570,54750,NULL,40.0000,NULL,NULL,'U/L'),(30,NULL,'AST','ANY',6570,54750,NULL,40.0000,NULL,NULL,'U/L'),(31,NULL,'ALP','ANY',6570,54750,44.0000,147.0000,NULL,NULL,'U/L'),(32,NULL,'BILT','ANY',6570,54750,NULL,1.2000,NULL,NULL,'mg/dL'),(33,NULL,'TSH','ANY',6570,54750,0.4000,4.0000,NULL,NULL,'mIU/L'),(34,NULL,'FT4','ANY',6570,54750,0.8000,1.8000,NULL,NULL,'ng/dL'),(35,NULL,'CHOL','ANY',6570,54750,NULL,200.0000,NULL,NULL,'mg/dL'),(36,NULL,'TG','ANY',6570,54750,NULL,150.0000,NULL,NULL,'mg/dL'),(37,NULL,'HDL','M',6570,54750,40.0000,NULL,NULL,NULL,'mg/dL'),(38,NULL,'HDL','F',6570,54750,50.0000,NULL,NULL,NULL,'mg/dL'),(39,NULL,'LDL','ANY',6570,54750,NULL,130.0000,NULL,NULL,'mg/dL'),(40,NULL,'CRP','ANY',6570,54750,NULL,5.0000,NULL,NULL,'mg/L'),(41,NULL,'PUS','ANY',6570,54750,NULL,5.0000,NULL,NULL,'/HPF'),(42,NULL,'RBC_URINE','ANY',6570,54750,NULL,2.0000,NULL,NULL,'/HPF'),(43,NULL,'HB','ANY',0,6569,NULL,NULL,7.0000,NULL,'g/dL'),(44,NULL,'PLT','ANY',0,6569,NULL,NULL,50000.0000,NULL,'/cumm'),(45,NULL,'K','ANY',0,6569,NULL,NULL,2.5000,6.0000,'mmol/L'),(46,NULL,'NA','ANY',0,6569,NULL,NULL,120.0000,160.0000,'mmol/L'),(47,NULL,'RBS','ANY',0,6569,NULL,NULL,2.8000,22.2000,'mmol/L'),(48,NULL,'FBS','ANY',0,6569,NULL,NULL,2.8000,22.2000,'mmol/L');
/*!40000 ALTER TABLE `reference_ranges` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `report_results`
--

DROP TABLE IF EXISTS `report_results`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `report_results` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `report_id` bigint unsigned NOT NULL,
  `sort_order` int unsigned NOT NULL DEFAULT '0',
  `test_code` varchar(50) NOT NULL,
  `test_name` varchar(100) NOT NULL,
  `value_text` varchar(50) DEFAULT NULL,
  `value_num` decimal(12,4) DEFAULT NULL,
  `unit` varchar(30) DEFAULT NULL,
  `ref_range_text` varchar(50) DEFAULT NULL,
  `lab_flag` varchar(10) DEFAULT NULL,
  `range_source` enum('lab','system') DEFAULT NULL,
  `status` enum('normal','borderline','high','low','critical','abnormal') DEFAULT NULL,
  `confidence` enum('high','medium','low') DEFAULT NULL,
  `edited_by_doctor` tinyint(1) NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`),
  KEY `ix_results_report` (`report_id`),
  KEY `ix_results_test` (`test_code`),
  CONSTRAINT `fk_results_report` FOREIGN KEY (`report_id`) REFERENCES `reports` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=20 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `report_results`
--

LOCK TABLES `report_results` WRITE;
/*!40000 ALTER TABLE `report_results` DISABLE KEYS */;
INSERT INTO `report_results` VALUES (7,1,0,'HB','Haemoglobin','10.2',10.2000,'g/dL','12-15.5','L','lab','low','high',0),(8,1,1,'WBC','Total Count (WBC)','13,800',13800.0000,'/cumm','4,000-11,000','H','lab','high','high',0),(9,1,2,'PLT','Platelet count','2,40,000',240000.0000,'/cumm','1,50,000-4,50,000',NULL,'lab','normal','high',0),(10,1,3,'ESR','ESR','18',18.0000,'mm/1st hr','0-20',NULL,'lab','normal','low',1),(11,1,4,'NEUT','Neutrophils','82',82.0000,'%','40-75',NULL,'lab','high','medium',0),(12,1,5,'CREAT','S. Creatinine','2.4',2.4000,'mg/dL',NULL,NULL,'system','high','high',0),(13,1,6,'K','Potassium','6.4',6.4000,'mmol/L',NULL,NULL,'system','critical',NULL,1),(14,2,0,'HB','Haemoglobin','10.2',10.2000,'g/dL','12-15.5','L','lab','low','high',0),(15,2,1,'WBC','Total Count (WBC)','13,800',13800.0000,'/cumm','4,000-11,000','H','lab','high','high',0),(16,2,2,'PLT','Platelet count','2,40,000',240000.0000,'/cumm','1,50,000-4,50,000',NULL,'lab','normal','high',0),(17,2,3,'ESR','ESR','38',38.0000,'mm/1st hr','0-20',NULL,'lab','high','low',0),(18,2,4,'NEUT','Neutrophils','82',82.0000,'%','40-75',NULL,'lab','high','medium',0),(19,2,5,'CREAT','S. Creatinine','2.4',2.4000,'mg/dL',NULL,NULL,'system','high','high',0);
/*!40000 ALTER TABLE `report_results` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `reports`
--

DROP TABLE IF EXISTS `reports`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `reports` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned NOT NULL,
  `patient_id` bigint unsigned NOT NULL,
  `visit_id` bigint unsigned DEFAULT NULL,
  `report_type` varchar(50) DEFAULT NULL,
  `lab_name` varchar(150) DEFAULT NULL,
  `report_date` date DEFAULT NULL,
  `file_path` varchar(255) NOT NULL,
  `file_hash` char(64) NOT NULL,
  `mime_type` varchar(50) NOT NULL,
  `original_name` varchar(150) DEFAULT NULL,
  `status` enum('uploaded','extracting','extracted','verified','failed') NOT NULL DEFAULT 'uploaded',
  `impression` text,
  `name_mismatch` tinyint(1) NOT NULL DEFAULT '0',
  `extraction_raw` json DEFAULT NULL,
  `error` varchar(255) DEFAULT NULL,
  `uploaded_by` bigint unsigned DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `deleted_at` datetime DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `ix_reports_hash` (`clinic_id`,`file_hash`),
  KEY `ix_reports_patient` (`patient_id`,`report_date`),
  KEY `fk_reports_visit` (`visit_id`),
  CONSTRAINT `fk_reports_patient` FOREIGN KEY (`patient_id`) REFERENCES `patients` (`id`),
  CONSTRAINT `fk_reports_visit` FOREIGN KEY (`visit_id`) REFERENCES `visits` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=11 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `reports`
--

LOCK TABLES `reports` WRITE;
/*!40000 ALTER TABLE `reports` DISABLE KEYS */;
INSERT INTO `reports` VALUES (1,1,3,4,'CBC','ABC Diagnostic','2026-10-10','./storage/reports/1/d0baf4c4c71e23ed6ee53ace7d60a4bdbc14d499cd368a5330492fa34a230c48.pdf','d0baf4c4c71e23ed6ee53ace7d60a4bdbc14d499cd368a5330492fa34a230c48','application/pdf','CBC রিপোর্ট.pdf','verified',NULL,0,'{\"results\": [{\"test\": \"Haemoglobin\", \"unit\": \"g/dL\", \"range\": \"12-15.5\", \"value\": \"10.2\", \"lab_flag\": \"L\", \"confidence\": \"high\"}, {\"test\": \"Total Count (WBC)\", \"unit\": \"/cumm\", \"range\": \"4,000-11,000\", \"value\": \"13,800\", \"lab_flag\": \"H\", \"confidence\": \"high\"}, {\"test\": \"Platelet count\", \"unit\": \"/cumm\", \"range\": \"1,50,000-4,50,000\", \"value\": \"2,40,000\", \"confidence\": \"high\"}, {\"test\": \"ESR\", \"unit\": \"mm/1st hr\", \"range\": \"0-20\", \"value\": \"38\", \"confidence\": \"low\"}, {\"test\": \"Neutrophils\", \"unit\": \"%\", \"range\": \"40-75\", \"value\": \"82\", \"confidence\": \"medium\"}, {\"test\": \"S. Creatinine\", \"unit\": \"mg/dL\", \"value\": \"2.4\", \"confidence\": \"high\"}], \"lab_name\": \"ABC Diagnostic\", \"impression\": null, \"report_date\": \"2026-10-10\", \"report_type\": \"CBC\", \"unreadable_parts\": [\"stamp at bottom\"], \"patient_name_on_report\": \"Smoke Allergic\"}',NULL,6,'2026-10-12 09:44:14','2026-10-12 09:44:14',NULL),(2,1,4,5,'CBC','ABC Diagnostic','2026-10-10','./storage/reports/1/6c7c1a9d068309f7828bd3e73ea387c65a0b3f95b861104f6a2a7d7d368cd8c7.jpg','6c7c1a9d068309f7828bd3e73ea387c65a0b3f95b861104f6a2a7d7d368cd8c7','image/jpeg','photo.png','extracted',NULL,0,'{\"results\": [{\"test\": \"Haemoglobin\", \"unit\": \"g/dL\", \"range\": \"12-15.5\", \"value\": \"10.2\", \"lab_flag\": \"L\", \"confidence\": \"high\"}, {\"test\": \"Total Count (WBC)\", \"unit\": \"/cumm\", \"range\": \"4,000-11,000\", \"value\": \"13,800\", \"lab_flag\": \"H\", \"confidence\": \"high\"}, {\"test\": \"Platelet count\", \"unit\": \"/cumm\", \"range\": \"1,50,000-4,50,000\", \"value\": \"2,40,000\", \"confidence\": \"high\"}, {\"test\": \"ESR\", \"unit\": \"mm/1st hr\", \"range\": \"0-20\", \"value\": \"38\", \"confidence\": \"low\"}, {\"test\": \"Neutrophils\", \"unit\": \"%\", \"range\": \"40-75\", \"value\": \"82\", \"confidence\": \"medium\"}, {\"test\": \"S. Creatinine\", \"unit\": \"mg/dL\", \"value\": \"2.4\", \"confidence\": \"high\"}], \"lab_name\": \"ABC Diagnostic\", \"impression\": null, \"report_date\": \"2026-10-10\", \"report_type\": \"CBC\", \"unreadable_parts\": [\"stamp at bottom\"], \"patient_name_on_report\": \"Smoke Allergic\"}',NULL,4,'2026-10-12 09:51:18','2026-10-12 09:51:18',NULL),(3,1,5,10,NULL,NULL,NULL,'./storage/reports/1/4f64934ae4277f8763c74a79fee4128898b1aad6b434a062e736858ec612bf95.jpg','4f64934ae4277f8763c74a79fee4128898b1aad6b434a062e736858ec612bf95','image/jpeg','4.jpeg','uploaded',NULL,0,NULL,NULL,6,'2026-10-13 03:35:32','2026-10-13 03:35:32',NULL),(4,1,5,10,NULL,NULL,NULL,'./storage/reports/1/8a28532e2890806bfeb6c2fab528379495225815ec9125a8594d3f5acd8bcda7.jpg','8a28532e2890806bfeb6c2fab528379495225815ec9125a8594d3f5acd8bcda7','image/jpeg','3.jpeg','uploaded',NULL,0,NULL,NULL,6,'2026-10-13 03:35:32','2026-10-13 03:35:32',NULL),(5,1,5,10,NULL,NULL,NULL,'./storage/reports/1/1d64e9d8082506460b1e79193bb4065c291a80ed98ee15b895f44223dcaa0cbf.jpg','1d64e9d8082506460b1e79193bb4065c291a80ed98ee15b895f44223dcaa0cbf','image/jpeg','2.jpeg','uploaded',NULL,0,NULL,NULL,6,'2026-10-13 03:35:32','2026-10-13 03:35:32',NULL),(6,1,5,10,NULL,NULL,NULL,'./storage/reports/1/f3f2a0e98887f8b04958a0d4ff938c2daaeac49c38903d5034023ff476a93172.jpg','f3f2a0e98887f8b04958a0d4ff938c2daaeac49c38903d5034023ff476a93172','image/jpeg','1.jpeg','uploaded',NULL,0,NULL,NULL,6,'2026-10-13 03:35:33','2026-10-13 03:35:33',NULL),(7,1,7,12,NULL,NULL,NULL,'./storage/reports/1/4f64934ae4277f8763c74a79fee4128898b1aad6b434a062e736858ec612bf95.jpg','4f64934ae4277f8763c74a79fee4128898b1aad6b434a062e736858ec612bf95','image/jpeg','4.jpeg','uploaded',NULL,0,NULL,NULL,6,'2026-10-13 04:04:22','2026-10-13 04:04:22',NULL),(8,1,7,12,NULL,NULL,NULL,'./storage/reports/1/8a28532e2890806bfeb6c2fab528379495225815ec9125a8594d3f5acd8bcda7.jpg','8a28532e2890806bfeb6c2fab528379495225815ec9125a8594d3f5acd8bcda7','image/jpeg','3.jpeg','uploaded',NULL,0,NULL,NULL,6,'2026-10-13 04:04:22','2026-10-13 04:04:22',NULL),(9,1,7,12,NULL,NULL,NULL,'./storage/reports/1/1d64e9d8082506460b1e79193bb4065c291a80ed98ee15b895f44223dcaa0cbf.jpg','1d64e9d8082506460b1e79193bb4065c291a80ed98ee15b895f44223dcaa0cbf','image/jpeg','2.jpeg','uploaded',NULL,0,NULL,NULL,6,'2026-10-13 04:04:22','2026-10-13 04:04:22',NULL),(10,1,7,12,NULL,NULL,NULL,'./storage/reports/1/f3f2a0e98887f8b04958a0d4ff938c2daaeac49c38903d5034023ff476a93172.jpg','f3f2a0e98887f8b04958a0d4ff938c2daaeac49c38903d5034023ff476a93172','image/jpeg','1.jpeg','uploaded',NULL,0,NULL,NULL,6,'2026-10-13 04:04:22','2026-10-13 04:04:22',NULL);
/*!40000 ALTER TABLE `reports` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `rx_templates`
--

DROP TABLE IF EXISTS `rx_templates`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `rx_templates` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned NOT NULL,
  `name` varchar(100) NOT NULL,
  `body` json NOT NULL,
  `created_by` bigint unsigned DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_rx_templates_name` (`clinic_id`,`name`),
  CONSTRAINT `fk_rx_templates_clinic` FOREIGN KEY (`clinic_id`) REFERENCES `clinics` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=7 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `rx_templates`
--

LOCK TABLES `rx_templates` WRITE;
/*!40000 ALTER TABLE `rx_templates` DISABLE KEYS */;
INSERT INTO `rx_templates` VALUES (1,1,'UTI (adult)','{\"advice\": [], \"diagnosis\": [\"Acute cystitis\"], \"medicines\": [], \"next_plan\": [], \"complaints\": [], \"examination\": [], \"investigations\": [], \"treatment_plan\": [], \"next_visit_date\": null, \"operation_notes\": null}',6,'2026-10-12 09:57:03','2026-10-12 09:57:03'),(3,1,'Acute cystitis','{\"advice\": [], \"diagnosis\": [\"Acute cystitis\"], \"medicines\": [], \"next_plan\": [], \"complaints\": [], \"examination\": [], \"investigations\": [], \"treatment_plan\": [], \"next_visit_date\": null, \"operation_notes\": null}',6,'2026-10-13 03:29:12','2026-10-13 03:29:12'),(6,1,'Acute cystitis3','{\"advice\": [], \"diagnosis\": [\"Acute cystitis\"], \"medicines\": [], \"next_plan\": [], \"complaints\": [], \"examination\": [], \"investigations\": [], \"treatment_plan\": [], \"next_visit_date\": null, \"operation_notes\": null}',6,'2026-10-13 03:29:18','2026-10-13 03:29:18');
/*!40000 ALTER TABLE `rx_templates` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `sms_messages`
--

DROP TABLE IF EXISTS `sms_messages`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `sms_messages` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned NOT NULL,
  `patient_id` bigint unsigned DEFAULT NULL,
  `phone` varchar(15) NOT NULL,
  `body` text NOT NULL,
  `kind` varchar(30) NOT NULL,
  `ref_id` bigint unsigned DEFAULT NULL,
  `status` enum('queued','sent','failed','cancelled') NOT NULL DEFAULT 'queued',
  `attempts` int unsigned NOT NULL DEFAULT '0',
  `error` varchar(255) DEFAULT NULL,
  `provider_ref` varchar(100) DEFAULT NULL,
  `send_after` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `created_by` bigint unsigned DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `sent_at` datetime DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `ix_sms_ref` (`kind`,`ref_id`),
  KEY `ix_sms_clinic` (`clinic_id`,`created_at`),
  KEY `fk_sms_patient` (`patient_id`),
  CONSTRAINT `fk_sms_clinic` FOREIGN KEY (`clinic_id`) REFERENCES `clinics` (`id`),
  CONSTRAINT `fk_sms_patient` FOREIGN KEY (`patient_id`) REFERENCES `patients` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=8 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `sms_messages`
--

LOCK TABLES `sms_messages` WRITE;
/*!40000 ALTER TABLE `sms_messages` DISABLE KEYS */;
INSERT INTO `sms_messages` VALUES (1,1,3,'8801711000001','প্রিয় Smoke Allergic, Demo Chamber-এ Dr Demo-এর সাথে আপনার অ্যাপয়েন্টমেন্ট ১৩/১০/২০২৬, ১৭:২০-এ নিশ্চিত হয়েছে।','appointment_confirm',1,'sent',1,NULL,'log','2026-10-12 09:06:17',7,'2026-10-12 09:06:17','2026-10-12 09:06:32'),(2,1,4,'8801711000002','প্রিয় Smoke Second, Demo Chamber-এ Dr Demo-এর সাথে আপনার অ্যাপয়েন্টমেন্ট ১২/১০/২০২৬, ১৮:০০-এ নিশ্চিত হয়েছে।','appointment_confirm',2,'sent',1,NULL,'log','2026-10-12 09:06:17',7,'2026-10-12 09:06:17','2026-10-12 09:06:32'),(3,1,3,'8801711000001','প্রিয় Smoke Allergic, Demo Chamber-এ Dr Demo-এর সাথে আপনার অ্যাপয়েন্টমেন্ট ১৫/১০/২০২৬, ১৮:০০-এ নিশ্চিত হয়েছে।','appointment_confirm',1,'cancelled',0,NULL,NULL,'2026-10-12 09:06:45',7,'2026-10-12 09:06:45',NULL),(4,1,3,'8801711000001','স্মরণিকা: Smoke Allergic, আগামীকাল (১৫/১০/২০২৬) ১৮:০০-এ Dr Demo-এর সাথে আপনার অ্যাপয়েন্টমেন্ট। Demo Chamber','appointment_reminder',1,'cancelled',0,NULL,NULL,'2026-10-14 04:00:00',7,'2026-10-12 09:06:45',NULL),(5,1,3,'8801711000001','প্রিয় Smoke Allergic, Demo Chamber-এ Dr Demo-এর সাথে আপনার অ্যাপয়েন্টমেন্ট ১৫/১০/২০২৬, ১৮:০০-এ নিশ্চিত হয়েছে।','appointment_confirm',1,'sent',1,NULL,'log','2026-10-12 09:06:46',7,'2026-10-12 09:06:45','2026-10-12 09:06:47'),(6,1,3,'8801711000001','স্মরণিকা: Smoke Allergic, আগামীকাল (১৫/১০/২০২৬) ১৮:০০-এ Dr Demo-এর সাথে আপনার অ্যাপয়েন্টমেন্ট। Demo Chamber','appointment_reminder',1,'queued',0,NULL,NULL,'2026-10-14 04:00:00',7,'2026-10-12 09:06:45',NULL),(7,1,3,'8801711000001','প্রিয় Smoke Allergic, ০৫/১০/২০২৬ তারিখে Dr Demo-এর কাছে আপনার ফলো-আপ ভিজিট। সিরিয়ালের জন্য কল করুন ০১৭০০০০০০০০। Demo Chamber','followup_reminder',4,'sent',1,NULL,'log','2026-10-12 09:07:56',7,'2026-10-12 09:07:55','2026-10-12 09:08:22');
/*!40000 ALTER TABLE `sms_messages` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `users`
--

DROP TABLE IF EXISTS `users`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `users` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned NOT NULL,
  `name` varchar(150) NOT NULL,
  `email` varchar(190) NOT NULL,
  `phone` varchar(15) DEFAULT NULL,
  `password_hash` varchar(255) NOT NULL,
  `role` enum('doctor','assistant','admin') NOT NULL,
  `degrees` varchar(255) DEFAULT NULL,
  `bmdc_reg_no` varchar(30) DEFAULT NULL,
  `signature_path` varchar(255) DEFAULT NULL,
  `is_active` tinyint(1) NOT NULL DEFAULT '1',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_users_email` (`email`),
  KEY `fk_users_clinic` (`clinic_id`),
  CONSTRAINT `fk_users_clinic` FOREIGN KEY (`clinic_id`) REFERENCES `clinics` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=9 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `users`
--

LOCK TABLES `users` WRITE;
/*!40000 ALTER TABLE `users` DISABLE KEYS */;
INSERT INTO `users` VALUES (1,1,'Dr Demo','doctor@example.com',NULL,'$argon2id$v=19$m=19456,t=2,p=1$7vqs3KrgNCxJgHC1gq7nxg$j8IqKozE0Z48xIjxanWRDpQii37vpSkFGyibFtryqPM','doctor','MBBS, FCPS (Medicine)','A-12345',NULL,1,'2026-10-12 05:58:35','2026-10-13 03:27:30'),(2,1,'Asst Demo','assistant@example.com',NULL,'$argon2id$v=19$m=19456,t=2,p=1$7vqs3KrgNCxJgHC1gq7nxg$j8IqKozE0Z48xIjxanWRDpQii37vpSkFGyibFtryqPM','assistant',NULL,NULL,NULL,1,'2026-10-12 06:12:43','2026-10-13 03:27:30'),(3,1,'Admin Demo','admin@example.com',NULL,'$argon2id$v=19$m=19456,t=2,p=1$7vqs3KrgNCxJgHC1gq7nxg$j8IqKozE0Z48xIjxanWRDpQii37vpSkFGyibFtryqPM','admin',NULL,NULL,NULL,1,'2026-10-12 06:12:44','2026-10-13 03:27:30'),(4,1,'P2 Doctor','p2doc@example.com',NULL,'$argon2id$v=19$m=19456,t=2,p=1$W8/ZYKCS2qBS+Axnuyaoww$7DUJFIifuNfUJWV6cGkuTRlSMbweUI2Djc8JzBtEphA','doctor',NULL,NULL,NULL,1,'2026-10-12 07:01:52','2026-10-12 07:01:52'),(5,1,'P2 Assistant','p2asst@example.com',NULL,'$argon2id$v=19$m=19456,t=2,p=1$7IuzvY6EPeNndXsq1Xmo2w$ufmJxwq9LBSpUa2w6PPO40VrrwJkFZa8MpDWbesh6nc','assistant',NULL,NULL,NULL,1,'2026-10-12 07:01:53','2026-10-12 07:01:53'),(6,1,'Demo Doctor','demo.doctor@example.com',NULL,'$argon2id$v=19$m=19456,t=2,p=1$7vqs3KrgNCxJgHC1gq7nxg$j8IqKozE0Z48xIjxanWRDpQii37vpSkFGyibFtryqPM','doctor',NULL,NULL,NULL,1,'2026-10-12 08:52:17','2026-10-12 08:52:17'),(7,1,'Demo Assistant','demo.assistant@example.com',NULL,'$argon2id$v=19$m=19456,t=2,p=1$Ns9a9bsqoUhv4XnuQzHWIQ$pD+ot2WEyHxsay2beUeMJ1IRO1yMFFaW8B3HYBzRAso','assistant',NULL,NULL,NULL,1,'2026-10-12 08:52:18','2026-10-12 08:52:18'),(8,1,'Demo Admin','demo.admin@example.com',NULL,'$argon2id$v=19$m=19456,t=2,p=1$erPgzk/va691Rhdn1K8fJg$+BJ31vecQEVonI1cK6AlhPjKtqYDFEXl2R9K8gFJ90k','admin',NULL,NULL,NULL,1,'2026-10-12 08:52:18','2026-10-12 08:52:18');
/*!40000 ALTER TABLE `users` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `visits`
--

DROP TABLE IF EXISTS `visits`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `visits` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `clinic_id` bigint unsigned NOT NULL,
  `patient_id` bigint unsigned NOT NULL,
  `visit_no` int unsigned NOT NULL,
  `visit_date` date NOT NULL,
  `visit_type` enum('new','followup','report') NOT NULL DEFAULT 'new',
  `queue_serial` int unsigned DEFAULT NULL,
  `queue_position` int unsigned DEFAULT NULL,
  `status` enum('waiting','in_consultation','seen','cancelled') NOT NULL DEFAULT 'waiting',
  `doctor_id` bigint unsigned DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `deleted_at` datetime DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `ix_visits_patient_date` (`patient_id`,`visit_date`),
  KEY `ix_visits_queue` (`clinic_id`,`visit_date`,`status`),
  KEY `fk_visits_doctor` (`doctor_id`),
  CONSTRAINT `fk_visits_doctor` FOREIGN KEY (`doctor_id`) REFERENCES `users` (`id`),
  CONSTRAINT `fk_visits_patient` FOREIGN KEY (`patient_id`) REFERENCES `patients` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=15 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `visits`
--

LOCK TABLES `visits` WRITE;
/*!40000 ALTER TABLE `visits` DISABLE KEYS */;
INSERT INTO `visits` VALUES (1,1,1,1,'2026-10-12','followup',1,2,'seen',6,'2026-10-12 06:13:02','2026-10-12 08:54:13',NULL),(2,1,1,2,'2026-10-12','new',2,2,'seen',1,'2026-10-12 06:13:02','2026-10-12 07:01:52',NULL),(3,1,1,3,'2026-10-12','followup',3,3,'seen',1,'2026-10-12 06:16:56','2026-10-12 07:01:52',NULL),(4,1,3,1,'2026-10-12','new',4,3,'seen',NULL,'2026-10-12 07:02:15','2026-10-12 07:02:15',NULL),(5,1,4,1,'2026-10-12','followup',5,1,'seen',4,'2026-10-12 07:02:15','2026-10-12 08:54:08',NULL),(6,1,1,4,'2026-10-12','followup',6,5,'waiting',NULL,'2026-10-12 09:01:42','2026-10-12 09:41:11',NULL),(7,1,4,2,'2026-10-12','new',7,4,'waiting',NULL,'2026-10-12 09:06:45','2026-10-12 09:41:11',NULL),(8,1,5,1,'2026-10-12','new',8,6,'in_consultation',6,'2026-10-12 09:57:16','2026-10-12 09:57:16',NULL),(9,1,5,2,'2026-10-12','new',9,7,'in_consultation',6,'2026-10-12 09:57:16','2026-10-12 09:57:16',NULL),(10,1,5,3,'2026-10-13','followup',1,1,'seen',6,'2026-10-13 03:29:00','2026-10-13 03:29:31',NULL),(11,1,5,4,'2026-10-13','followup',2,2,'seen',6,'2026-10-13 03:46:27','2026-10-13 03:46:58',NULL),(12,1,7,1,'2026-10-13','new',3,3,'seen',6,'2026-10-13 04:00:41','2026-10-13 04:01:07',NULL),(13,1,7,2,'2026-10-13','followup',4,4,'seen',6,'2026-10-13 04:05:05','2026-10-13 04:30:32',NULL),(14,1,7,3,'2026-10-13','followup',5,5,'in_consultation',6,'2026-10-13 04:52:13','2026-10-13 04:52:13',NULL);
/*!40000 ALTER TABLE `visits` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `vitals`
--

DROP TABLE IF EXISTS `vitals`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `vitals` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `visit_id` bigint unsigned NOT NULL,
  `bp_sys` smallint unsigned DEFAULT NULL,
  `bp_dia` smallint unsigned DEFAULT NULL,
  `pulse` smallint unsigned DEFAULT NULL,
  `temp` decimal(4,1) DEFAULT NULL,
  `weight` decimal(5,2) DEFAULT NULL,
  `height` decimal(5,2) DEFAULT NULL,
  `bmi` decimal(4,1) DEFAULT NULL,
  `spo2` tinyint unsigned DEFAULT NULL,
  `rbs` decimal(5,2) DEFAULT NULL,
  `recorded_by` bigint unsigned DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `ix_vitals_visit` (`visit_id`),
  CONSTRAINT `fk_vitals_visit` FOREIGN KEY (`visit_id`) REFERENCES `visits` (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `vitals`
--

LOCK TABLES `vitals` WRITE;
/*!40000 ALTER TABLE `vitals` DISABLE KEYS */;
INSERT INTO `vitals` VALUES (1,4,130,85,NULL,NULL,61.00,160.00,23.8,NULL,NULL,5,'2026-10-12 07:02:15');
/*!40000 ALTER TABLE `vitals` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Dumping events for database 'virtual_doctor'
--

--
-- Dumping routines for database 'virtual_doctor'
--
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2026-10-13  4:57:49
