CREATE TABLE "public"."accounts" (
    "account_id" uuid NOT NULL,
    "name" varchar NOT NULL,
    "balance" bigint NOT NULL DEFAULT 0,
    PRIMARY KEY ("account_id")
);

CREATE TABLE "public"."gift_cards" (
    "gift_card_id" uuid NOT NULL,
    "name" varchar NOT NULL,
    "price" bigint NOT NULL DEFAULT 0,
    "count" integer NOT NULL,
    PRIMARY KEY ("gift_card_id")
);

CREATE TABLE "public"."orders" (
    "order_id" uuid NOT NULL,
    "account_id" uuid NOT NULL,
    "items" jsonb NOT NULL DEFAULT '{}',
    "total_price" int8 NOT NULL DEFAULT 0,
    PRIMARY KEY ("order_id")
);

ALTER TABLE "public"."orders" ADD FOREIGN KEY ("account_id") REFERENCES "public"."accounts" ("account_id");
