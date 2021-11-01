#!/bin/sh

createdb -U "$POSTGRES_USER" app_test

psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" -d app_test <<-"EOSQL"
  CREATE OR REPLACE FUNCTION reset_db() RETURNS void AS $$
  DECLARE
      statements CURSOR FOR
          SELECT tablename FROM pg_tables
          WHERE tableowner = 'postgres' AND schemaname = 'public';
  BEGIN
      FOR stmt IN statements LOOP
          EXECUTE 'TRUNCATE TABLE ' || quote_ident(stmt.tablename) || ' CASCADE;';
      END LOOP;
  END;
  $$ LANGUAGE plpgsql;
EOSQL
