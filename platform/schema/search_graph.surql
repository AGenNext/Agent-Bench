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

-- Everything is a searchable node in the site, so MAX context = all nodes,
-- all files, full logs; MIN context = the current topic node. The agent decides
-- how far to expand or reduce along the graph, within the site (= this database).
-- Contextual search scans METADATA, never executable content. Reading
-- descriptors (path, type, tags, description, digest) is safe; scanning or
-- running the executable bytes is not — so executables are indexed by metadata
-- only. Declarative text (docs/markdown) lives in `page` and is searchable there.
DEFINE TABLE file SCHEMAFULL;
DEFINE FIELD path         ON file TYPE string;
DEFINE FIELD content_type ON file TYPE string;                 -- e.g. text/markdown, application/wasm
DEFINE FIELD kind         ON file TYPE string DEFAULT "artifact"
    ASSERT $value IN ["artifact", "executable", "doc"];
DEFINE FIELD digest       ON file TYPE option<string>;         -- content address
DEFINE FIELD tags         ON file TYPE array<string> DEFAULT [];
DEFINE FIELD description   ON file TYPE string DEFAULT "";
-- Search over metadata only (path + description), incl. executables' metadata
-- (additional) — never file content/bytes.
DEFINE INDEX file_meta_search ON file FIELDS path, description SEARCH ANALYZER ab_text BM25;

DEFINE TABLE log SCHEMAFULL;
DEFINE FIELD at      ON log TYPE datetime DEFAULT time::now();
DEFINE FIELD message ON log TYPE string;
DEFINE INDEX log_search ON log FIELDS message SEARCH ANALYZER ab_text BM25;
