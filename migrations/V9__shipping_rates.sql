CREATE TABLE shipping_rates (
  shipping_rate_id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  offer_id UUID NOT NULL REFERENCES offers(offer_id),
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() ON UPDATE NOW(),
  amount INT NOT NULL,
  currency VARCHAR NOT NULL,
  all_countries BOOLEAN NOT NULL,
  specific_countries VARCHAR,
  CONSTRAINT uq_offer_id_user_id_currency UNIQUE (offer_id, user_id, currency)
)