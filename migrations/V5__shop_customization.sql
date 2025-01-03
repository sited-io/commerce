CREATE TABLE shop_customizations (
  shop_id UUID PRIMARY KEY REFERENCES shops(shop_id),
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  logo_image_light_url_path VARCHAR,
  logo_image_dark_url_path VARCHAR,
  banner_image_light_url_path VARCHAR,
  banner_image_dark_url_path VARCHAR,
  show_banner_in_listing BOOLEAN,
  show_banner_on_home BOOLEAN,
  header_background_color_light VARCHAR,
  header_background_color_dark VARCHAR,
  header_content_color_light VARCHAR,
  header_content_color_dark VARCHAR,
  secondary_background_color_light VARCHAR,
  secondary_background_color_dark VARCHAR,
  secondary_content_color_light VARCHAR,
  secondary_content_color_dark VARCHAR
);

CREATE TRIGGER update_shops_updated_at BEFORE UPDATE
    ON shop_customizations FOR EACH ROW EXECUTE PROCEDURE 
    updated_at_now();