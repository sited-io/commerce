CREATE TABLE shop_domains (
  shop_id UUID PRIMARY KEY REFERENCES shops(shop_id),
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  domain VARCHAR NOT NULL,
  status VARCHAR NOT NULL,
  client_id VARCHAR
);

CREATE TRIGGER update_shops_updated_at BEFORE UPDATE
    ON shop_domains FOR EACH ROW EXECUTE PROCEDURE 
    updated_at_now();