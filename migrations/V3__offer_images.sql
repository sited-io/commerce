CREATE TABLE offer_images (
  offer_image_id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  offer_id UUID NOT NULL REFERENCES offers(offer_id),
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  image_url_path VARCHAR NOT NULL,
  ordering INT NOT NULL
);

CREATE TRIGGER update_shops_updated_at BEFORE UPDATE
    ON offer_images FOR EACH ROW EXECUTE PROCEDURE 
    updated_at_now();