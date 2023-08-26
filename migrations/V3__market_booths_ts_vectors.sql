ALTER TABLE
  market_booths
ADD
  COLUMN name_ts tsvector GENERATED ALWAYS AS (to_tsvector('simple', name)) STORED,
ADD
  COLUMN description_ts tsvector GENERATED ALWAYS AS (to_tsvector('simple', description)) STORED;