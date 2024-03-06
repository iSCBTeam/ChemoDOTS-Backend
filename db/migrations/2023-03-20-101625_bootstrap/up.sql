CREATE FUNCTION real_subtype_diff(a real, b real) RETURNS double precision
LANGUAGE SQL
IMMUTABLE
RETURNS NULL ON NULL INPUT
RETURN a - b;

CREATE TYPE realrange AS RANGE (
	SUBTYPE = real,
    SUBTYPE_DIFF = real_subtype_diff
);

CREATE TABLE moiety_group (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"name" varchar NOT NULL
);

CREATE TABLE moiety (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"id_moiety_group" bigint NOT NULL,
	"name" varchar NOT NULL,
	"rdpickle" bytea NOT NULL,
	"smarts" varchar NOT NULL,
	"priority" integer NOT NULL
);

CREATE TABLE substructure_filter_group (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"name" varchar NOT NULL
);

CREATE TABLE substructure_filter (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"id_substructure_filter_group" bigint NOT NULL,
	"name" varchar NOT NULL,
	"rdpickle" bytea NOT NULL,
	"smarts" varchar NOT NULL
);

CREATE TABLE reaction (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"name" varchar NOT NULL,
	"slug" varchar NOT NULL,
	"rdpickle" bytea NOT NULL,
	"smarts" varchar NOT NULL,
	"multistep" boolean NOT NULL,
	"reference" varchar
);

CREATE TABLE compound_provider (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"name" varchar NOT NULL,
	"ts_upd" timestamp,
	UNIQUE(name)
);

CREATE TABLE compound (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"id_compound_provider" bigint NOT NULL,
	"refid" varchar NOT NULL,
	"sdf" varchar,
	"smiles" varchar,
	"available" boolean NOT NULL,
	UNIQUE(id_compound_provider, refid)
);

CREATE TABLE building_block (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"rdpickle" bytea NOT NULL,
	"smiles" varchar NOT NULL,
	UNIQUE(smiles)
);

-- A given compound can refer to a set of different building blocks (e.g. after tautomer computation)
-- Several compounds can refer to the same set of building blocks (e.g. duplicate compounds with different counter-ions)
CREATE TABLE building_block_origin (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"id_building_block" bigint NOT NULL,
	"id_compound" bigint NOT NULL
);

CREATE TABLE building_block_reactant (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"id_building_block" bigint NOT NULL,
	"id_reaction" bigint NOT NULL,
	"reactant_idx" int NOT NULL
);

CREATE TABLE experiment (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"uuid" uuid NOT NULL DEFAULT gen_random_uuid(),
	"name" varchar NOT NULL,
	"status" varchar NOT NULL,
	"ts_start" timestamp NOT NULL,
	"ts_end" timestamp,
	UNIQUE(uuid)
);

CREATE TABLE experiment_selected_provider (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"id_experiment" bigint NOT NULL,
	"id_compound_provider" bigint NOT NULL
);

-- Store successive postprocessing filter settings
CREATE TABLE experiment_postproc_filter (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"id_experiment" bigint NOT NULL,
	"ts" timestamp NOT NULL,
	"desc_fsp3" realrange NOT NULL,
	"desc_hba" int4range NOT NULL,
	"desc_hbd" int4range NOT NULL,
	"desc_clogp" realrange NOT NULL,
	"desc_mw" realrange NOT NULL,
	"desc_tpsa" realrange NOT NULL
);

CREATE TABLE experiment_frag (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"id_experiment" bigint NOT NULL,
	"id_moiety" bigint NOT NULL,
	-- Growing mode: idx = 0, Linking mode (TODO): idx = 0 or 1
	"idx" int NOT NULL,
	"rdpickle" bytea NOT NULL,
	"smiles" varchar NOT NULL,
	"moiety_atoms" integer[] NOT NULL,
	UNIQUE(id_experiment, idx)
);

CREATE TABLE experiment_frag_reactant (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"id_experiment_frag" bigint NOT NULL,
	"id_reaction" bigint NOT NULL,
	"reactant_idx" int NOT NULL,
	"moiety_atoms" integer[] NOT NULL
);

CREATE TABLE experiment_substructure_filter (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"id_experiment" bigint NOT NULL,
	"id_substructure_filter" bigint NOT NULL,
	"reject" boolean NOT NULL
);

CREATE TABLE experiment_product (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"id_experiment_frag_reactant" bigint NOT NULL,
	"name" varchar NOT NULL,
	"fullname" varchar NOT NULL,
	"rdpickle" bytea NOT NULL,
	"smiles" varchar NOT NULL,
	"dup_count" int NOT NULL,
	"desc_fsp3" real NOT NULL,
	"desc_hba" integer NOT NULL,
	"desc_hbd" integer NOT NULL,
	"desc_clogp" real NOT NULL,
	"desc_mw" real NOT NULL,
	"desc_tpsa" real NOT NULL
);

CREATE INDEX index__experiment_product__descriptors ON experiment_product USING btree (id_experiment_frag_reactant, desc_fsp3, desc_hba, desc_hbd, desc_clogp, desc_mw, desc_tpsa);

-- Distinct building block reactants can lead to a partially overlapping set of products (duplicates), so we need to track in both directions
CREATE TABLE experiment_product_origin (
	"id" bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	"id_experiment_product" bigint NOT NULL,
	"id_building_block_reactant" bigint NOT NULL
);

ALTER TABLE moiety ADD CONSTRAINT fk__moiety__moiety_group
	FOREIGN KEY ("id_moiety_group")
	REFERENCES moiety_group("id");

CREATE INDEX index__moiety__id_moiety_group ON moiety USING btree (id_moiety_group);

ALTER TABLE substructure_filter ADD CONSTRAINT fk__substructure_filter__substructure_filter_group
	FOREIGN KEY ("id_substructure_filter_group")
	REFERENCES substructure_filter_group("id");

CREATE INDEX index__substructure_filter__id_substructure_filter_group ON substructure_filter USING btree (id_substructure_filter_group);

ALTER TABLE compound ADD CONSTRAINT fk__compound__compound_provider
	FOREIGN KEY ("id_compound_provider")
	REFERENCES compound_provider("id");

CREATE INDEX index__compound__id_compound_provider ON compound USING btree (id_compound_provider);

ALTER TABLE building_block_origin ADD CONSTRAINT fk__building_block_origin__building_block
	FOREIGN KEY ("id_building_block")
	REFERENCES building_block("id");
ALTER TABLE building_block_origin ADD CONSTRAINT fk__building_block_origin__compound
	FOREIGN KEY ("id_compound")
	REFERENCES compound("id");

CREATE INDEX index__building_block_origin__id_building_block ON building_block_origin USING btree (id_building_block);
CREATE INDEX index__building_block_origin__id_compound ON building_block_origin USING btree (id_compound);

ALTER TABLE building_block_reactant ADD CONSTRAINT fk__building_block_reactant__building_block
	FOREIGN KEY ("id_building_block")
	REFERENCES building_block("id");
ALTER TABLE building_block_reactant ADD CONSTRAINT fk__building_block_reactant__reaction
	FOREIGN KEY ("id_reaction")
	REFERENCES reaction("id");

CREATE INDEX index__building_block_reactant__id_building_block ON building_block_reactant USING btree (id_building_block);
CREATE INDEX index__building_block_reactant__id_reaction ON building_block_reactant USING btree (id_reaction);

ALTER TABLE experiment_selected_provider ADD CONSTRAINT fk__experiment_selected_provider__experiment
	FOREIGN KEY ("id_experiment")
	REFERENCES experiment("id");
ALTER TABLE experiment_selected_provider ADD CONSTRAINT fk__experiment_selected_provider__compound_provider
	FOREIGN KEY ("id_compound_provider")
	REFERENCES compound_provider("id");

CREATE INDEX index__experiment_selected_provider__id_experiment ON experiment_selected_provider USING btree (id_experiment);
CREATE INDEX index__experiment_selected_provider__id_compound_provider ON experiment_selected_provider USING btree (id_compound_provider);

ALTER TABLE experiment_postproc_filter ADD CONSTRAINT fk__experiment_postproc_filter__experiment
	FOREIGN KEY ("id_experiment")
	REFERENCES experiment("id");

CREATE INDEX index__experiment_postproc_filter__id_experiment ON experiment_postproc_filter USING btree (id_experiment);

ALTER TABLE experiment_frag ADD CONSTRAINT fk__experiment_frag__experiment
	FOREIGN KEY ("id_experiment")
	REFERENCES experiment("id");
ALTER TABLE experiment_frag ADD CONSTRAINT fk__experiment_frag__moiety
	FOREIGN KEY ("id_moiety")
	REFERENCES moiety("id");

CREATE INDEX index__experiment_frag__id_experiment ON experiment_frag USING btree (id_experiment);
CREATE INDEX index__experiment_frag__id_moiety ON experiment_frag USING btree (id_moiety);

ALTER TABLE experiment_frag_reactant ADD CONSTRAINT fk__experiment_frag_reactant__experiment_frag
	FOREIGN KEY ("id_experiment_frag")
	REFERENCES experiment_frag("id");
ALTER TABLE experiment_frag_reactant ADD CONSTRAINT fk__experiment_frag_reactant__reaction
	FOREIGN KEY ("id_reaction")
	REFERENCES reaction("id");

CREATE INDEX index__experiment_frag_reactant__id_experiment_frag ON experiment_frag_reactant USING btree (id_experiment_frag);
CREATE INDEX index__experiment_frag_reactant__id_reaction ON experiment_frag_reactant USING btree (id_reaction);

ALTER TABLE experiment_substructure_filter ADD CONSTRAINT fk__experiment_substructure_filter__experiment
	FOREIGN KEY ("id_experiment")
	REFERENCES experiment("id");
ALTER TABLE experiment_substructure_filter ADD CONSTRAINT fk__experiment_substructure_filter__substructure_filter
	FOREIGN KEY ("id_substructure_filter")
	REFERENCES substructure_filter("id");

CREATE INDEX index__experiment_substructure_filter__id_experiment ON experiment_substructure_filter USING btree (id_experiment);
CREATE INDEX index__experiment_substructure_filter__id_substructure_filter ON experiment_substructure_filter USING btree (id_substructure_filter);

ALTER TABLE experiment_product ADD CONSTRAINT fk__experiment_product__experiment_frag_reactant
	FOREIGN KEY ("id_experiment_frag_reactant")
	REFERENCES experiment_frag_reactant("id");

CREATE INDEX index__experiment_product__id_experiment_frag_reactant ON experiment_product USING btree (id_experiment_frag_reactant);

ALTER TABLE experiment_product_origin ADD CONSTRAINT fk__experiment_product_origin__experiment_product
	FOREIGN KEY ("id_experiment_product")
	REFERENCES experiment_product("id");
ALTER TABLE experiment_product_origin ADD CONSTRAINT fk__experiment_product_origin__building_block_reactant
	FOREIGN KEY ("id_building_block_reactant")
	REFERENCES building_block_reactant("id");

CREATE INDEX index__experiment_product_origin__id_experiment_product ON experiment_product_origin USING btree (id_experiment_product);
CREATE INDEX index__experiment_product_origin__id_building_block_reactant ON experiment_product_origin USING btree (id_building_block_reactant);
