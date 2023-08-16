CREATE TABLE market_booths (
    market_booth_id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW() ON UPDATE NOW(),
    name VARCHAR NOT NULL,
    description TEXT,
    CONSTRAINT uq_user_id_name UNIQUE (user_id, name)
);