CREATE TABLE shipping_rates (
  shipping_rate_id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  offer_id UUID NOT NULL REFERENCES offers(offer_id),
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() ON UPDATE NOW(),
  country VARCHAR NOT NULL,
  amount INT NOT NULL,
  currency VARCHAR NOT NULL,
  CONSTRAINT uq_offer_id_country UNIQUE (offer_id, country)
)