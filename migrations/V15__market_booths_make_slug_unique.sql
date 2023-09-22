SET
  sql_safe_updates = false;

UPDATE
  market_booths
SET
  slug = regexp_replace(name, '[^a-z0-9\\-_]+', '-', 'gi');

ALTER TABLE
  market_booths
ALTER COLUMN
  slug
SET
  NOT NULL;

CREATE UNIQUE INDEX uq_market_booth_slug ON market_booths (slug);