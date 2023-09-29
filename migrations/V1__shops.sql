CREATE TABLE shops (
    shop_id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() ON UPDATE NOW(),
    slug VARCHAR NOT NULL UNIQUE,
    domain VARCHAR UNIQUE,
    name VARCHAR NOT NULL,
    name_ts tsvector GENERATED ALWAYS AS (to_tsvector('simple', name)) STORED,
    description TEXT,
    description_ts tsvector GENERATED ALWAYS AS (to_tsvector('simple', description)) STORED,
    platform_fee_percent INT NOT NULL DEFAULT(1),
    minimum_platform_fee_cent INT NOT NULL DEFAULT(50),
    CONSTRAINT uq_user_id_shop_name UNIQUE (user_id, name)
);