// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    building_block (id) {
        id -> Int8,
        rdpickle -> Bytea,
        smiles -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    building_block_origin (id) {
        id -> Int8,
        id_building_block -> Int8,
        id_compound -> Int8,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    building_block_reactant (id) {
        id -> Int8,
        id_building_block -> Int8,
        id_reaction -> Int8,
        reactant_idx -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    compound (id) {
        id -> Int8,
        id_compound_provider -> Int8,
        refid -> Varchar,
        sdf -> Nullable<Varchar>,
        smiles -> Nullable<Varchar>,
        available -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    compound_provider (id) {
        id -> Int8,
        name -> Varchar,
        ts_upd -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    experiment (id) {
        id -> Int8,
        uuid -> Uuid,
        name -> Varchar,
        status -> Varchar,
        ts_start -> Timestamp,
        ts_end -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    experiment_frag (id) {
        id -> Int8,
        id_experiment -> Int8,
        id_moiety -> Int8,
        idx -> Int4,
        rdpickle -> Bytea,
        smiles -> Varchar,
        moiety_atoms -> Array<Nullable<Int4>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    experiment_frag_reactant (id) {
        id -> Int8,
        id_experiment_frag -> Int8,
        id_reaction -> Int8,
        reactant_idx -> Int4,
        moiety_atoms -> Array<Nullable<Int4>>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    experiment_postproc_filter (id) {
        id -> Int8,
        id_experiment -> Int8,
        ts -> Timestamp,
        desc_fsp3 -> Realrange,
        desc_hba -> Int4range,
        desc_hbd -> Int4range,
        desc_clogp -> Realrange,
        desc_mw -> Realrange,
        desc_tpsa -> Realrange,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    experiment_product (id) {
        id -> Int8,
        id_experiment_frag_reactant -> Int8,
        name -> Varchar,
        fullname -> Varchar,
        rdpickle -> Bytea,
        smiles -> Varchar,
        dup_count -> Int4,
        desc_fsp3 -> Float4,
        desc_hba -> Int4,
        desc_hbd -> Int4,
        desc_clogp -> Float4,
        desc_mw -> Float4,
        desc_tpsa -> Float4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    experiment_product_origin (id) {
        id -> Int8,
        id_experiment_product -> Int8,
        id_building_block_reactant -> Int8,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    experiment_selected_provider (id) {
        id -> Int8,
        id_experiment -> Int8,
        id_compound_provider -> Int8,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    experiment_substructure_filter (id) {
        id -> Int8,
        id_experiment -> Int8,
        id_substructure_filter -> Int8,
        reject -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    moiety (id) {
        id -> Int8,
        id_moiety_group -> Int8,
        name -> Varchar,
        rdpickle -> Bytea,
        smarts -> Varchar,
        priority -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    moiety_group (id) {
        id -> Int8,
        name -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    reaction (id) {
        id -> Int8,
        name -> Varchar,
        slug -> Varchar,
        rdpickle -> Bytea,
        smarts -> Varchar,
        multistep -> Bool,
        reference -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    substructure_filter (id) {
        id -> Int8,
        id_substructure_filter_group -> Int8,
        name -> Varchar,
        rdpickle -> Bytea,
        smarts -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use crate::custom::sql_types::*;

    substructure_filter_group (id) {
        id -> Int8,
        name -> Varchar,
    }
}

diesel::joinable!(building_block_origin -> building_block (id_building_block));
diesel::joinable!(building_block_origin -> compound (id_compound));
diesel::joinable!(building_block_reactant -> building_block (id_building_block));
diesel::joinable!(building_block_reactant -> reaction (id_reaction));
diesel::joinable!(compound -> compound_provider (id_compound_provider));
diesel::joinable!(experiment_frag -> experiment (id_experiment));
diesel::joinable!(experiment_frag -> moiety (id_moiety));
diesel::joinable!(experiment_frag_reactant -> experiment_frag (id_experiment_frag));
diesel::joinable!(experiment_frag_reactant -> reaction (id_reaction));
diesel::joinable!(experiment_postproc_filter -> experiment (id_experiment));
diesel::joinable!(experiment_product -> experiment_frag_reactant (id_experiment_frag_reactant));
diesel::joinable!(experiment_product_origin -> building_block_reactant (id_building_block_reactant));
diesel::joinable!(experiment_product_origin -> experiment_product (id_experiment_product));
diesel::joinable!(experiment_selected_provider -> compound_provider (id_compound_provider));
diesel::joinable!(experiment_selected_provider -> experiment (id_experiment));
diesel::joinable!(experiment_substructure_filter -> experiment (id_experiment));
diesel::joinable!(experiment_substructure_filter -> substructure_filter (id_substructure_filter));
diesel::joinable!(moiety -> moiety_group (id_moiety_group));
diesel::joinable!(substructure_filter -> substructure_filter_group (id_substructure_filter_group));

diesel::allow_tables_to_appear_in_same_query!(
    building_block,
    building_block_origin,
    building_block_reactant,
    compound,
    compound_provider,
    experiment,
    experiment_frag,
    experiment_frag_reactant,
    experiment_postproc_filter,
    experiment_product,
    experiment_product_origin,
    experiment_selected_provider,
    experiment_substructure_filter,
    moiety,
    moiety_group,
    reaction,
    substructure_filter,
    substructure_filter_group,
);
