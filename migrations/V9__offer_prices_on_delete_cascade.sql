ALTER TABLE
  offer_prices DROP CONSTRAINT offer_prices_offer_id_fkey,
ADD
  CONSTRAINT offer_prices_offer_id_fkey2 FOREIGN KEY (offer_id) REFERENCES offers(offer_id) ON DELETE CASCADE;