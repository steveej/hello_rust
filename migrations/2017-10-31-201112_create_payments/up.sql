-- Your SQL goes here
CREATE TABLE payments (
    id SERIAL PRIMARY KEY,
    date VARCHAR NOT NULL,
    number VARCHAR NOT NULL,
    unused VARCHAR NOT NULL,
    subject VARCHAR NOT NULL,
    invoice_number VARCHAR NOT NULL,
    amount VARCHAR NOT NULL
)