-- Make the evaluation/linked-data artifacts queryable: full-text search +
-- graph relations to crawl and construct. Depends on pages.surql +
-- memory_attribute.surql (page, card_eval tables).

-- --- Full-text search (BM25) ----------------------------------------------
DEFINE ANALYZER ab_text TOKENIZERS class FILTERS lowercase, ascii;
DEFINE INDEX page_search    ON page      FIELDS content   SEARCH ANALYZER ab_text BM25 HIGHLIGHTS;
DEFINE INDEX eval_attr_search ON card_eval FIELDS attribute SEARCH ANALYZER ab_text BM25;

-- --- Graph: subject ──evaluated──▶ card_eval ──measures──▶ attribute -------
DEFINE TABLE subject SCHEMAFULL;
DEFINE FIELD did  ON subject TYPE string;        -- DID
DEFINE INDEX subject_did ON subject FIELDS did UNIQUE;

DEFINE TABLE attribute SCHEMAFULL;
DEFINE FIELD name ON attribute TYPE string;
DEFINE INDEX attribute_name ON attribute FIELDS name UNIQUE;

-- Relation edges (graph crawl / construct).
DEFINE TABLE evaluated TYPE RELATION FROM subject  TO card_eval;
DEFINE TABLE measures  TYPE RELATION FROM card_eval TO attribute;
