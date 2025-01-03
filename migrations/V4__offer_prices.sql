CREATE TABLE offer_prices (
  offer_price_id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  offer_id UUID NOT NULL REFERENCES offers(offer_id) ON DELETE CASCADE,
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  currency VARCHAR NOT NULL,
  price_type VARCHAR NOT NULL,
  billing_scheme VARCHAR NOT NULL,
  unit_amount INT NOT NULL,
  recurring_interval VARCHAR,
  recurring_interval_count INT
);

CREATE TRIGGER update_shops_updated_at BEFORE UPDATE
    ON offer_prices FOR EACH ROW EXECUTE PROCEDURE 
    updated_at_now();