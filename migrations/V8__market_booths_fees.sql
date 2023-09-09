ALTER TABLE
  market_booths
ADD
  COLUMN platform_fee_percent INT NOT NULL DEFAULT(2),
ADD
  COLUMN minimum_platform_fee_cent INT NOT NULL DEFAULT(50);