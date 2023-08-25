CREATE TABLE offers (
  offer_id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  market_booth_id UUID NOT NULL REFERENCES market_booths(market_booth_id),
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() ON UPDATE NOW(),
  name VARCHAR NOT NULL,
  description TEXT,
  CONSTRAINT uq_market_booth_id_user_id_offer_name UNIQUE (market_booth_id, user_id, name)
);