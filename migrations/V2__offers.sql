CREATE TABLE offers (
  offer_id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  shop_id UUID NOT NULL REFERENCES shops(shop_id),
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() ON UPDATE NOW(),
  name VARCHAR NOT NULL,
  name_ts tsvector GENERATED ALWAYS AS (to_tsvector('simple', name)) STORED,
  description TEXT,
  description_ts tsvector GENERATED ALWAYS AS (to_tsvector('simple', description)) STORED,
  type_ VARCHAR,
  is_active BOOLEAN NOT NULL DEFAULT 'f',
  is_featured BOOLEAN NOT NULL DEFAULT 'f',
  CONSTRAINT uq_shop_id_user_id_offer_name UNIQUE (shop_id, user_id, name)
);