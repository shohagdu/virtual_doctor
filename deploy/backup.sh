#!/bin/sh
# Daily logical backup. Keeps 14 days locally; copy /backups off-server (encrypted) as well.
set -eu
while true; do
  ts=$(date -u +%Y%m%dT%H%M%SZ)
  mysqldump -h mysql -u root -p"$MYSQL_ROOT_PASSWORD" --single-transaction --routines \
    --source-data=2 "$MYSQL_DATABASE" | gzip > "/backups/${MYSQL_DATABASE}-${ts}.sql.gz"
  find /backups -name '*.sql.gz' -mtime +14 -delete
  sleep 86400
done
