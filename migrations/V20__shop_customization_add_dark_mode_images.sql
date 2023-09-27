ALTER TABLE
  shop_customizations
ADD
  COLUMN logo_image_dark_url_path VARCHAR,
ADD
  COLUMN banner_image_dark_url_path VARCHAR,
ADD
  COLUMN show_banner_in_listing BOOLEAN,
ADD
  COLUMN show_banner_on_home BOOLEAN;