CREATE TABLE shop_customizations (
  shop_id UUID PRIMARY KEY REFERENCES market_booths(market_booth_id),
  user_id VARCHAR NOT NULL,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() ON UPDATE NOW(),
  logo_image_url_path VARCHAR,
  banner_image_url_path VARCHAR,
  header_background_color_light VARCHAR,
  header_background_color_dark VARCHAR,
  header_content_color_light VARCHAR,
  header_content_color_dark VARCHAR,
  secondary_background_color_light VARCHAR,
  secondary_background_color_dark VARCHAR,
  secondary_content_color_light VARCHAR,
  secondary_content_color_dark VARCHAR
)