ALTER TABLE
  offer_prices
ADD
  COLUMN recurring_interval VARCHAR,
ADD
  COLUMN recurring_interval_count INT;