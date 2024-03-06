use std::ops::Bound;

use diesel::pg::PgRowByRowLoadingMode;
use diesel::{prelude::*, sql_types::Bool, expression::AsExpression, connection::DefaultLoadingMode, helper_types::AsSelect, helper_types::Concat};
use chrono::NaiveDateTime;
use field_count::FieldCount;
use serde::Deserialize;
use uuid::Uuid;

use crate::custom::{RealrangeExpressionMethods, RealrangeType};
use crate::expression::dsl::{StringAgg, string_agg};
use crate::schema::*;

pub type DB = diesel::pg::Pg;
pub type DBConnection = diesel::pg::PgConnection;

allow_columns_to_appear_in_same_group_by_clause!(
	building_block::id,
	building_block::rdpickle,
	building_block::smiles,
	building_block_reactant::id,
	building_block_reactant::id_building_block,
	building_block_reactant::id_reaction,
	building_block_reactant::reactant_idx,
	experiment::id,
	experiment::name,
	experiment_product::id,
	experiment_product::fullname,
	experiment_product::name,
	experiment_product::rdpickle,
	experiment_product::smiles,
	reaction::id,
	reaction::slug);

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = building_block)]
#[diesel(belongs_to(BuildingBlockGroup, foreign_key = id_building_block_group))]
#[diesel(check_for_backend(DB))]
pub struct BuildingBlock {
	pub id: i64,
	pub rdpickle: Vec<u8>,
	pub smiles: String,
}

#[derive(AsChangeset, FieldCount, Insertable, Debug, PartialEq)]
#[diesel(table_name = building_block)]
#[diesel(check_for_backend(DB))]
pub struct NewBuildingBlock<'s> {
	pub rdpickle: &'s [u8],
	pub smiles: &'s str,
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug, PartialEq)]
#[diesel(table_name = building_block_origin)]
#[diesel(belongs_to(BuildingBlock, foreign_key = id_building_block))]
#[diesel(belongs_to(Compound, foreign_key = id_compound))]
#[diesel(check_for_backend(DB))]
pub struct BuildingBlockOrigin {
	pub id: i64,
	pub id_building_block: i64,
	pub id_compound: i64,
}

#[derive(AsChangeset, FieldCount, Insertable, Debug, PartialEq)]
#[diesel(table_name = building_block_origin)]
#[diesel(check_for_backend(DB))]
pub struct NewBuildingBlockOrigin {
	pub id_building_block: i64,
	pub id_compound: i64,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = building_block)]
#[diesel(check_for_backend(DB))]
pub struct ExportableBuildingBlock {
	pub id: i64,
	pub rdpickle: Vec<u8>,
	pub smiles: String,
	#[diesel(select_expression_type = StringAgg<Concat<Concat<compound_provider::columns::name, &'static str>, compound::columns::refid>, &'static str>)]
	#[diesel(select_expression = string_agg(compound_provider::columns::name.concat("-").concat(compound::columns::refid), ","))]
	pub name: String,
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug, PartialEq)]
#[diesel(table_name = building_block_reactant)]
#[diesel(belongs_to(BuildingBlock, foreign_key = id_building_block))]
#[diesel(belongs_to(Reaction, foreign_key = id_reaction))]
#[diesel(check_for_backend(DB))]
pub struct BuildingBlockReactant {
	pub id: i64,
	pub id_building_block: i64,
	pub id_reaction: i64,
	pub reactant_idx: i32,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = building_block_reactant)]
#[diesel(belongs_to(BuildingBlock, foreign_key = id_building_block))]
#[diesel(belongs_to(Reaction, foreign_key = id_reaction))]
#[diesel(check_for_backend(DB))]
pub struct MergedBuildingBlockReactant {
	pub id: i64,
	pub id_building_block: i64,
	pub id_reaction: i64,
	pub reactant_idx: i32,
	#[diesel(select_expression_type = building_block::rdpickle)]
	#[diesel(select_expression = building_block::rdpickle)]
	pub rdpickle: Vec<u8>,
	#[diesel(select_expression_type = building_block::smiles)]
	#[diesel(select_expression = building_block::smiles)]
	pub smiles: String,
	#[diesel(select_expression_type = StringAgg<Concat<Concat<compound_provider::columns::name, &'static str>, compound::columns::refid>, &'static str>)]
	#[diesel(select_expression = string_agg(compound_provider::columns::name.concat("-").concat(compound::columns::refid), ","))]
	pub name: String,
	#[diesel(select_expression_type = Concat<Concat<Concat<Concat<experiment::columns::name, &'static str>, reaction::columns::slug>, &'static str>, StringAgg<Concat<Concat<compound_provider::columns::name, &'static str>, compound::columns::refid>, &'static str>>)]
	#[diesel(select_expression = experiment::columns::name.concat("_").concat(reaction::columns::slug).concat("_").concat(string_agg(compound_provider::columns::name.concat("-").concat(compound::columns::refid), ",")))]
	pub fullname: String,
	
}

#[derive(AsChangeset, FieldCount, Insertable, Debug, PartialEq)]
#[diesel(table_name = building_block_reactant)]
#[diesel(check_for_backend(DB))]
pub struct NewBuildingBlockReactant {
	pub id_building_block: i64,
	pub id_reaction: i64,
	pub reactant_idx: i32,
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug, PartialEq)]
#[diesel(table_name = compound)]
#[diesel(belongs_to(CompoundProvider, foreign_key = id_compound_provider))]
#[diesel(check_for_backend(DB))]
pub struct Compound {
	pub id: i64,
	pub id_compound_provider: i64,
	pub refid: String,
	pub sdf: Option<String>,
	pub smiles: Option<String>,
	pub available: bool,
}

#[derive(AsChangeset, FieldCount, Insertable, Debug, PartialEq)]
#[diesel(table_name = compound)]
#[diesel(check_for_backend(DB))]
pub struct NewCompound<'s> {
	pub id_compound_provider: i64,
	pub refid: &'s str,
	pub sdf: Option<&'s str>,
	pub smiles: Option<&'s str>,
	pub available: bool,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = compound_provider)]
#[diesel(check_for_backend(DB))]
pub struct CompoundProvider {
	pub id: i64,
	pub name: String,
	pub ts_upd: Option<NaiveDateTime>,
}

#[derive(AsChangeset, Insertable, Debug, PartialEq)]
#[diesel(table_name = compound_provider)]
#[diesel(check_for_backend(DB))]
pub struct NewCompoundProvider<'s> {
	pub name: &'s str,
	pub ts_upd: Option<NaiveDateTime>,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = experiment)]
#[diesel(check_for_backend(DB))]
pub struct Experiment {
	pub id: i64,
	pub uuid: Uuid,
	pub name: String,
	pub status: String,
	pub ts_start: NaiveDateTime,
	pub ts_end: Option<NaiveDateTime>,
}

#[derive(AsChangeset, Insertable, Debug, PartialEq)]
#[diesel(table_name = experiment)]
#[diesel(check_for_backend(DB))]
pub struct NewExperiment<'s> {
	pub name: &'s str,
	pub status: &'s str,
	pub ts_start: NaiveDateTime,
	pub ts_end: Option<NaiveDateTime>,
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug, PartialEq)]
#[diesel(table_name = experiment_selected_provider)]
#[diesel(belongs_to(Experiment, foreign_key = id_experiment))]
#[diesel(belongs_to(CompoundProvider, foreign_key = id_compound_provider))]
#[diesel(check_for_backend(DB))]
pub struct ExperimentSelectedProvider {
	pub id: i64,
	pub id_experiment: i64,
	pub id_compound_provider: i64,
}

#[derive(AsChangeset, FieldCount, Insertable, Debug, PartialEq)]
#[diesel(table_name = experiment_selected_provider)]
#[diesel(check_for_backend(DB))]
pub struct NewExperimentSelectedProvider {
	pub id_experiment: i64,
	pub id_compound_provider: i64,
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug, PartialEq)]
#[diesel(table_name = experiment_postproc_filter)]
#[diesel(belongs_to(Experiment, foreign_key = id_experiment))]
#[diesel(check_for_backend(DB))]
pub struct ExperimentPostprocFilter {
	pub id: i64,
	pub id_experiment: i64,
	pub ts: NaiveDateTime,
	pub desc_fsp3: RealrangeType,
	pub desc_hba: (Bound<i32>, Bound<i32>),
	pub desc_hbd: (Bound<i32>, Bound<i32>),
	pub desc_clogp: RealrangeType,
	pub desc_mw: RealrangeType,
	pub desc_tpsa: RealrangeType,
}

#[derive(AsChangeset, Insertable, Debug, PartialEq)]
#[diesel(table_name = experiment_postproc_filter)]
#[diesel(check_for_backend(DB))]
pub struct NewExperimentPostprocFilter {
	pub id_experiment: i64,
	pub ts: NaiveDateTime,
	pub desc_fsp3: RealrangeType,
	pub desc_hba: (Bound<i32>, Bound<i32>),
	pub desc_hbd: (Bound<i32>, Bound<i32>),
	pub desc_clogp: RealrangeType,
	pub desc_mw: RealrangeType,
	pub desc_tpsa: RealrangeType,
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug, PartialEq)]
#[diesel(table_name = experiment_frag)]
#[diesel(belongs_to(Experiment, foreign_key = id_experiment))]
#[diesel(belongs_to(Moiety, foreign_key = id_moiety))]
#[diesel(check_for_backend(DB))]
pub struct ExperimentFrag {
	pub id: i64,
	pub id_experiment: i64,
	pub id_moiety: i64,
	pub idx: i32,
	pub rdpickle: Vec<u8>,
	pub smiles: String,
	pub moiety_atoms: Vec<Option<i32>>,
}

#[derive(AsChangeset, Insertable, Debug, PartialEq)]
#[diesel(table_name = experiment_frag)]
#[diesel(check_for_backend(DB))]
pub struct NewExperimentFrag<'s> {
	pub id_experiment: i64,
	pub id_moiety: i64,
	pub idx: i32,
	pub rdpickle: &'s [u8],
	pub smiles: &'s str,
	pub moiety_atoms: &'s [i32],
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug, PartialEq)]
#[diesel(table_name = experiment_frag_reactant)]
#[diesel(belongs_to(ExperimentFrag, foreign_key = id_experiment_frag))]
#[diesel(belongs_to(Reaction, foreign_key = id_reaction))]
#[diesel(check_for_backend(DB))]
pub struct ExperimentFragReactant {
	pub id: i64,
	pub id_experiment_frag: i64,
	pub id_reaction: i64,
	pub reactant_idx: i32,
	pub moiety_atoms: Vec<Option<i32>>,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = experiment_frag_reactant)]
#[diesel(belongs_to(ExperimentFrag, foreign_key = id_experiment_frag))]
#[diesel(belongs_to(Reaction, foreign_key = id_reaction))]
#[diesel(belongs_to(Experiment, foreign_key = id_experiment))]
#[diesel(belongs_to(Moiety, foreign_key = id_moiety))]
#[diesel(check_for_backend(DB))]
pub struct MergedExperimentFragReactant {
	pub id: i64,
	pub id_experiment_frag: i64,
	pub id_reaction: i64,
	pub reactant_idx: i32,
	// We use the *real* moiety atoms from frag_reactant, not the raw ones from frag
	pub moiety_atoms: Vec<Option<i32>>,
	#[diesel(select_expression_type = experiment_frag::id_experiment)]
	#[diesel(select_expression = experiment_frag::id_experiment)]
	pub id_experiment: i64,
	#[diesel(select_expression_type = experiment_frag::idx)]
	#[diesel(select_expression = experiment_frag::idx)]
	pub idx: i32,
	#[diesel(select_expression_type = experiment_frag::rdpickle)]
	#[diesel(select_expression = experiment_frag::rdpickle)]
	pub rdpickle: Vec<u8>,
	#[diesel(select_expression_type = experiment_frag::smiles)]
	#[diesel(select_expression = experiment_frag::smiles)]
	pub smiles: String,
	#[diesel(select_expression_type = experiment_frag::id_moiety)]
	#[diesel(select_expression = experiment_frag::id_moiety)]
	pub id_moiety: i64,
}

#[derive(AsChangeset, Insertable, Debug, PartialEq)]
#[diesel(table_name = experiment_frag_reactant)]
#[diesel(check_for_backend(DB))]
pub struct NewExperimentFragReactant<'s> {
	pub id_experiment_frag: i64,
	pub id_reaction: i64,
	pub reactant_idx: i32,
	pub moiety_atoms: &'s [i32],
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug, PartialEq)]
#[diesel(table_name = experiment_product)]
#[diesel(belongs_to(ExperimentFragReactant, foreign_key = id_experiment_frag_reactant))]
#[diesel(check_for_backend(DB))]
pub struct ExperimentProduct {
	pub id: i64,
	pub id_experiment_frag_reactant: i64,
	pub name: String,
	pub fullname: String,
	pub rdpickle: Vec<u8>,
	pub smiles: String,
	pub dup_count: i32,
	pub desc_fsp3: f32,
	pub desc_hba: i32,
	pub desc_hbd: i32,
	pub desc_clogp: f32,
	pub desc_mw: f32,
	pub desc_tpsa: f32,
}

#[derive(AsChangeset, FieldCount, Insertable, Debug, PartialEq)]
#[diesel(table_name = experiment_product)]
#[diesel(check_for_backend(DB))]
pub struct NewExperimentProduct<'s> {
	pub id_experiment_frag_reactant: i64,
	pub name: &'s str,
	pub fullname: &'s str,
	pub rdpickle: &'s [u8],
	pub smiles: &'s str,
	pub dup_count: i32,
	pub desc_fsp3: f32,
	pub desc_hba: i32,
	pub desc_hbd: i32,
	pub desc_clogp: f32,
	pub desc_mw: f32,
	pub desc_tpsa: f32,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = experiment_product)]
#[diesel(check_for_backend(DB))]
pub struct ExportableExperimentProduct {
	pub id: i64,
	pub rdpickle: Vec<u8>,
	pub smiles: String,
	pub name: String,
	pub fullname: String,
	#[diesel(select_expression_type = experiment_frag_reactant::id_reaction)]
	#[diesel(select_expression = experiment_frag_reactant::id_reaction)]
	pub id_reaction: i64,
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug, PartialEq)]
#[diesel(table_name = experiment_product_origin)]
#[diesel(belongs_to(BuildingBlockReactant, foreign_key = id_building_block_reactant))]
#[diesel(belongs_to(ExperimentProduct, foreign_key = id_experiment_product))]
#[diesel(check_for_backend(DB))]
pub struct ExperimentProductOrigin {
	pub id: i64,
	pub id_building_block_reactant: i64,
	pub id_experiment_product: i64,
}

#[derive(AsChangeset, FieldCount, Insertable, Debug, PartialEq)]
#[diesel(table_name = experiment_product_origin)]
#[diesel(check_for_backend(DB))]
pub struct NewExperimentProductOrigin {
	pub id_building_block_reactant: i64,
	pub id_experiment_product: i64,
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug, PartialEq)]
#[diesel(table_name = experiment_substructure_filter)]
#[diesel(belongs_to(Experiment, foreign_key = id_experiment))]
#[diesel(belongs_to(SubstructureFilter, foreign_key = id_substructure_filter))]
#[diesel(check_for_backend(DB))]
pub struct ExperimentSubstructureFilter {
	pub id: i64,
	pub id_experiment: i64,
	pub id_substructure_filter: i64,
	pub reject: bool,
}

#[derive(AsChangeset, Insertable, Debug, PartialEq)]
#[diesel(table_name = experiment_substructure_filter)]
#[diesel(check_for_backend(DB))]
pub struct NewExperimentSubstructureFilter {
	pub id_experiment: i64,
	pub id_substructure_filter: i64,
	pub reject: bool,
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug, PartialEq)]
#[diesel(table_name = moiety)]
#[diesel(belongs_to(MoietyGroup, foreign_key = id_moiety_group))]
#[diesel(check_for_backend(DB))]
pub struct Moiety {
	pub id: i64,
	pub id_moiety_group: i64,
	pub name: String,
	pub rdpickle : Vec<u8>,
	pub smarts: String,
	pub priority: i32,
}

#[derive(AsChangeset, Insertable, Debug, PartialEq)]
#[diesel(table_name = moiety)]
#[diesel(check_for_backend(DB))]
pub struct NewMoiety<'s> {
	pub id_moiety_group: i64,
	pub name: &'s str,
	pub rdpickle : &'s [u8],
	pub smarts: &'s str,
	pub priority: i32,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = moiety_group)]
#[diesel(check_for_backend(DB))]
pub struct MoietyGroup {
	pub id: i64,
	pub name: String,
}

#[derive(AsChangeset, Insertable, Debug, PartialEq)]
#[diesel(table_name = moiety_group)]
#[diesel(check_for_backend(DB))]
pub struct NewMoietyGroup<'s> {
	pub name: &'s str,
}

#[derive(Queryable, Identifiable, Selectable, Associations, Debug, PartialEq)]
#[diesel(table_name = substructure_filter)]
#[diesel(belongs_to(SubstructureFilterGroup, foreign_key = id_substructure_filter_group))]
#[diesel(check_for_backend(DB))]
pub struct SubstructureFilter {
	pub id: i64,
	pub id_substructure_filter_group: i64,
	pub name: String,
	pub rdpickle: Vec<u8>,
	pub smarts: String,
}

#[derive(AsChangeset, Insertable, Debug, PartialEq)]
#[diesel(table_name = substructure_filter)]
#[diesel(check_for_backend(DB))]
pub struct NewSubstructureFilter<'s> {
	pub id_substructure_filter_group: i64,
	pub name: &'s str,
	pub rdpickle: &'s [u8],
	pub smarts: &'s str,
}


#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = substructure_filter_group)]
#[diesel(check_for_backend(DB))]
pub struct SubstructureFilterGroup {
	pub id: i64,
	pub name: String,
}

#[derive(AsChangeset, Insertable, Debug, PartialEq)]
#[diesel(table_name = substructure_filter_group)]
#[diesel(check_for_backend(DB))]
pub struct NewSubstructureFilterGroup<'s> {
	pub name: &'s str,
}

#[derive(Clone, Queryable, Identifiable, Selectable, Debug, PartialEq)]
#[diesel(table_name = reaction)]
#[diesel(check_for_backend(DB))]
pub	struct Reaction {
	pub id: i64,
	pub name: String,
	pub slug: String,
	pub rdpickle: Vec<u8>,
	pub smarts: String,
	pub multistep: bool,
	pub reference: Option<String>,
}

#[derive(AsChangeset, Insertable, Debug, PartialEq)]
#[diesel(table_name = reaction)]
#[diesel(check_for_backend(DB))]
pub	struct NewReaction<'s> {
	pub name: &'s str,
	pub slug: &'s str,
	pub rdpickle: &'s [u8],
	pub smarts: &'s str,
	pub multistep: bool,
	pub reference: Option<&'s str>,
}

fn boxed_bool<T>(val: bool) -> Box<dyn BoxableExpression<T, DB, SqlType = Bool>> {
	Box::new(AsExpression::<Bool>::as_expression(val))
}

pub fn create_building_block(conn: &mut DBConnection, elem: &NewBuildingBlock) -> QueryResult<BuildingBlock> {
	diesel::insert_into(building_block::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_building_block(conn: &mut DBConnection, id: i64) -> QueryResult<BuildingBlock> {
	building_block::table.find(id)
		.first(conn)
}

pub fn count_building_blocks(conn: &mut DBConnection) -> QueryResult<i64> {
	building_block::table
		.count()
		.get_result(conn)
}

pub fn count_building_blocks_with_experiment_providers(conn: &mut DBConnection, exp: &Experiment) -> QueryResult<i64> {
	experiment::table
		.inner_join(experiment_selected_provider::table
		.inner_join(compound_provider::table
		.inner_join(compound::table
		.inner_join(building_block_origin::table
		.inner_join(building_block::table)))))
		.filter(experiment::id.eq(exp.id))
		.select(diesel::dsl::count_distinct(building_block::id))
		.get_result(conn)
}

pub fn get_building_block_by_smiles(conn: &mut DBConnection, smiles: &str) -> QueryResult<BuildingBlock> {
	building_block::table.filter(building_block::smiles.eq(smiles))
		.first(conn)
}

pub fn get_or_create_building_block(conn: &mut DBConnection, elem: &NewBuildingBlock) -> QueryResult<BuildingBlock> {
	diesel::insert_into(building_block::table)
		.values(elem)
		.on_conflict(building_block::smiles)
		.do_update()
		.set(building_block::smiles.eq(building_block::smiles))
		.get_result(conn)
}

pub fn get_or_create_building_blocks(conn: &mut DBConnection, elems: &[NewBuildingBlock]) -> QueryResult<Vec<BuildingBlock>> {
	diesel::insert_into(building_block::table)
		.values(elems)
		.on_conflict(building_block::smiles)
		.do_update()
		// Dummy update
		.set(building_block::smiles.eq(diesel::upsert::excluded(building_block::smiles)))
		.get_results(conn)
}

pub fn update_building_block(conn: &mut DBConnection, id: i64, elem: &NewBuildingBlock) -> QueryResult<BuildingBlock> {
	diesel::update(building_block::table)
		.filter(building_block::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_building_block(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(building_block::table)
		.filter(building_block::id.eq(id))
		.execute(conn)
}

pub fn create_building_block_origin(conn: &mut DBConnection) -> QueryResult<BuildingBlockOrigin> {
	diesel::insert_into(building_block_origin::table)
		.default_values()
		.get_result(conn)
}

pub fn create_building_block_origins(conn: &mut DBConnection, elems: &[NewBuildingBlockOrigin]) -> QueryResult<Vec<BuildingBlockOrigin>> {
	diesel::insert_into(building_block_origin::table)
		.values(elems)
		.get_results(conn)
}

pub fn get_building_block_origin(conn: &mut DBConnection, id: i64) -> QueryResult<BuildingBlockOrigin> {
	building_block_origin::table.find(id)
		.first(conn)
}

pub fn delete_building_block_origin(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(building_block_origin::table)
		.filter(building_block_origin::id.eq(id))
		.execute(conn)
}

pub fn delete_all_building_block_origins(conn: &mut DBConnection) -> QueryResult<usize> {
	diesel::delete(building_block_origin::table)
		.execute(conn)
}

pub fn create_building_block_reactant(conn: &mut DBConnection, elem: &NewBuildingBlockReactant) -> QueryResult<BuildingBlockReactant> {
	diesel::insert_into(building_block_reactant::table)
		.values(elem)
		.get_result(conn)
}

pub fn create_building_block_reactants(conn: &mut DBConnection, elem: &[NewBuildingBlockReactant]) -> QueryResult<Vec<BuildingBlockReactant>> {
	diesel::insert_into(building_block_reactant::table)
		.values(elem)
		.get_results(conn)
}

pub fn get_building_block_reactant(conn: &mut DBConnection, id: i64) -> QueryResult<BuildingBlockReactant> {
	building_block_reactant::table.find(id)
		.first(conn)
}

pub fn update_building_block_reactant(conn: &mut DBConnection, id: i64, elem: &NewBuildingBlockReactant) -> QueryResult<BuildingBlockReactant> {
	diesel::update(building_block_reactant::table)
		.filter(building_block_reactant::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_building_block_reactant(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(building_block_reactant::table)
		.filter(building_block_reactant::id.eq(id))
		.execute(conn)
}

pub fn create_compound_return(conn: &mut DBConnection, elem: &NewCompound) -> QueryResult<Compound> {
	diesel::insert_into(compound::table)
		.values(elem)
		.get_result(conn)
}

pub fn create_compound(conn: &mut DBConnection, elem: &NewCompound) -> QueryResult<usize> {
	diesel::insert_into(compound::table)
		.values(elem)
		.execute(conn)
}

pub fn create_compounds(conn: &mut DBConnection, elem: &[NewCompound]) -> QueryResult<usize> {
	diesel::insert_into(compound::table)
		.values(elem)
		.execute(conn)
}

pub fn create_compounds_return(conn: &mut DBConnection, elem: &[NewCompound]) -> QueryResult<Vec<Compound>> {
	diesel::insert_into(compound::table)
		.values(elem)
		.get_results(conn)
}

pub fn get_compound(conn: &mut DBConnection, id: i64) -> QueryResult<Compound> {
	compound::table.find(id)
		.first(conn)
}

pub fn update_compound(conn: &mut DBConnection, id: i64, elem: &NewCompound) -> QueryResult<Compound> {
	diesel::update(compound::table)
		.filter(compound::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_compound(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(compound::table)
		.filter(compound::id.eq(id))
		.execute(conn)
}

pub fn delete_all_compounds(conn: &mut DBConnection) -> QueryResult<usize> {
	diesel::delete(compound::table)
		.execute(conn)
}

pub fn delete_all_compounds_with_provider(conn: &mut DBConnection, provider: &CompoundProvider) -> QueryResult<usize> {
	diesel::delete(compound::table)
		.filter(compound::id_compound_provider.eq(provider.id))
		.execute(conn)
}

pub fn create_compound_provider(conn: &mut DBConnection, elem: &NewCompoundProvider) -> QueryResult<CompoundProvider> {
	diesel::insert_into(compound_provider::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_compound_provider(conn: &mut DBConnection, id: i64) -> QueryResult<CompoundProvider> {
	compound_provider::table.find(id)
		.first(conn)
}

pub fn get_compound_provider_by_name(conn: &mut DBConnection, name: &str) -> QueryResult<CompoundProvider> {
	compound_provider::table.filter(compound_provider::name.eq(name))
		.first(conn)
}

pub fn get_compound_providers_by_name<'a>(conn: &'a mut DBConnection, name: &'a [&str]) -> QueryResult<impl Iterator<Item = QueryResult<CompoundProvider>> + 'a> {
	compound_provider::table.filter(compound_provider::name.eq_any(name))
		.select(CompoundProvider::as_select())
		.load_iter::<_, PgRowByRowLoadingMode>(conn)
}

pub fn update_compound_provider(conn: &mut DBConnection, id: i64, elem: &NewCompoundProvider) -> QueryResult<CompoundProvider> {
	diesel::update(compound_provider::table)
		.filter(compound_provider::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_compound_provider(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(compound_provider::table)
		.filter(compound_provider::id.eq(id))
		.execute(conn)
}

pub fn create_experiment(conn: &mut DBConnection, elem: &NewExperiment) -> QueryResult<Experiment> {
	diesel::insert_into(experiment::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_experiment(conn: &mut DBConnection, id: i64) -> QueryResult<Experiment> {
	experiment::table.find(id)
		.first(conn)
}

pub fn get_experiment_with_uuid(conn: &mut DBConnection, uuid: Uuid) -> QueryResult<Experiment> {
	experiment::table.filter(experiment::uuid.eq(uuid))
		.first(conn)
}

pub fn update_experiment(conn: &mut DBConnection, id: i64, elem: &NewExperiment) -> QueryResult<Experiment> {
	diesel::update(experiment::table)
		.filter(experiment::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_experiment(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(experiment::table)
		.filter(experiment::id.eq(id))
		.execute(conn)
}

pub fn create_experiment_selected_provider(conn: &mut DBConnection, elem: &NewExperimentSelectedProvider) -> QueryResult<ExperimentSelectedProvider> {
	diesel::insert_into(experiment_selected_provider::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_experiment_selected_provider(conn: &mut DBConnection, id: i64) -> QueryResult<ExperimentSelectedProvider> {
	experiment_selected_provider::table.find(id)
		.first(conn)
}

pub fn update_experiment_selected_provider(conn: &mut DBConnection, id: i64, elem: &NewExperimentSelectedProvider) -> QueryResult<ExperimentSelectedProvider> {
	diesel::update(experiment_selected_provider::table)
		.filter(experiment_selected_provider::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_experiment_selected_provider(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(experiment_selected_provider::table)
		.filter(experiment_selected_provider::id.eq(id))
		.execute(conn)
}

pub fn create_experiment_postproc_filter(conn: &mut DBConnection, elem: &NewExperimentPostprocFilter) -> QueryResult<ExperimentPostprocFilter> {
	diesel::insert_into(experiment_postproc_filter::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_experiment_postproc_filter(conn: &mut DBConnection, id: i64) -> QueryResult<ExperimentPostprocFilter> {
	experiment_postproc_filter::table.find(id)
		.first(conn)
}

pub fn get_experiment_postproc_filter_with_experiment(conn: &mut DBConnection, exp: &Experiment) -> QueryResult<ExperimentPostprocFilter> {
	experiment_postproc_filter::table
		.filter(experiment_postproc_filter::id_experiment.eq(exp.id))
		.first(conn)
}

pub fn get_last_experiment_postproc_filter_with_experiment(conn: &mut DBConnection, exp: &Experiment) -> QueryResult<ExperimentPostprocFilter> {
	experiment_postproc_filter::table
		.order_by(experiment_postproc_filter::ts.desc())
		.filter(experiment_postproc_filter::id_experiment.eq(exp.id))
		.first(conn)
}

pub fn update_experiment_postproc_filter(conn: &mut DBConnection, id: i64, elem: &NewExperimentPostprocFilter) -> QueryResult<ExperimentPostprocFilter> {
	diesel::update(experiment_postproc_filter::table)
		.filter(experiment_postproc_filter::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_experiment_postproc_filter(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(experiment_postproc_filter::table)
		.filter(experiment_postproc_filter::id.eq(id))
		.execute(conn)
}

pub fn create_experiment_substructure_filter(conn: &mut DBConnection, elem: &NewExperimentSubstructureFilter) -> QueryResult<ExperimentSubstructureFilter> {
	diesel::insert_into(experiment_substructure_filter::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_experiment_substructure_filter(conn: &mut DBConnection, id: i64) -> QueryResult<ExperimentSubstructureFilter> {
	experiment_substructure_filter::table.find(id)
		.first(conn)
}

pub fn update_experiment_substructure_filter(conn: &mut DBConnection, id: i64, elem: &NewExperimentSubstructureFilter) -> QueryResult<ExperimentSubstructureFilter> {
	diesel::update(experiment_substructure_filter::table)
		.filter(experiment_substructure_filter::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_experiment_substructure_filter(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(experiment_substructure_filter::table)
		.filter(experiment_substructure_filter::id.eq(id))
		.execute(conn)
}

pub fn create_experiment_frag(conn: &mut DBConnection, elem: &NewExperimentFrag) -> QueryResult<ExperimentFrag> {
	diesel::insert_into(experiment_frag::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_experiment_frag(conn: &mut DBConnection, id: i64) -> QueryResult<ExperimentFrag> {
	experiment_frag::table.find(id)
		.first(conn)
}

pub fn update_experiment_frag(conn: &mut DBConnection, id: i64, elem: &NewExperimentFrag) -> QueryResult<ExperimentFrag> {
	diesel::update(experiment_frag::table)
		.filter(experiment_frag::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_experiment_frag(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(experiment_frag::table)
		.filter(experiment_frag::id.eq(id))
		.execute(conn)
}

pub fn create_experiment_frag_reactant(conn: &mut DBConnection, elem: &NewExperimentFragReactant) -> QueryResult<ExperimentFragReactant> {
	diesel::insert_into(experiment_frag_reactant::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_experiment_frag_reactant(conn: &mut DBConnection, id: i64) -> QueryResult<ExperimentFragReactant> {
	experiment_frag_reactant::table.find(id)
		.first(conn)
}

pub fn update_experiment_frag_reactant(conn: &mut DBConnection, id: i64, elem: &NewExperimentFragReactant) -> QueryResult<ExperimentFragReactant> {
	diesel::update(experiment_frag_reactant::table)
		.filter(experiment_frag_reactant::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_experiment_frag_reactant(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(experiment_frag_reactant::table)
		.filter(experiment_frag_reactant::id.eq(id))
		.execute(conn)
}

pub fn create_experiment_product(conn: &mut DBConnection, elem: &NewExperimentProduct) -> QueryResult<ExperimentProduct> {
	diesel::insert_into(experiment_product::table)
		.values(elem)
		.get_result(conn)
}

pub fn create_experiment_products(conn: &mut DBConnection, elem: &[NewExperimentProduct]) -> QueryResult<Vec<ExperimentProduct>> {
	diesel::insert_into(experiment_product::table)
		.values(elem)
		.get_results(conn)
}

pub fn get_experiment_product(conn: &mut DBConnection, id: i64) -> QueryResult<ExperimentProduct> {
	experiment_product::table.find(id)
		.first(conn)
}

pub fn update_experiment_product(conn: &mut DBConnection, id: i64, elem: &NewExperimentProduct) -> QueryResult<ExperimentProduct> {
	diesel::update(experiment_product::table)
		.filter(experiment_product::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_experiment_product(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(experiment_product::table)
		.filter(experiment_product::id.eq(id))
		.execute(conn)
}

pub fn create_experiment_product_origin(conn: &mut DBConnection, elem: &NewExperimentProductOrigin) -> QueryResult<ExperimentProductOrigin> {
	diesel::insert_into(experiment_product_origin::table)
		.values(elem)
		.get_result(conn)
}

pub fn create_experiment_product_origins(conn: &mut DBConnection, elem: &[NewExperimentProductOrigin]) -> QueryResult<usize> {
	diesel::insert_into(experiment_product_origin::table)
		.values(elem)
		.execute(conn)
}

pub fn get_experiment_product_origin(conn: &mut DBConnection, id: i64) -> QueryResult<ExperimentProductOrigin> {
	experiment_product_origin::table.find(id)
		.first(conn)
}

pub fn delete_experiment_product_origin(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(experiment_product_origin::table)
		.filter(experiment_product_origin::id.eq(id))
		.execute(conn)
}

pub fn create_moiety(conn: &mut DBConnection, elem: &NewMoiety) -> QueryResult<Moiety> {
	diesel::insert_into(moiety::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_moiety(conn: &mut DBConnection, id: i64) -> QueryResult<Moiety> {
	moiety::table.find(id)
		.first(conn)
}

pub fn update_moiety(conn: &mut DBConnection, id: i64, elem: &NewMoiety) -> QueryResult<Moiety> {
	diesel::update(moiety::table)
		.filter(moiety::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_moiety(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(moiety::table)
		.filter(moiety::id.eq(id))
		.execute(conn)
}

pub fn create_moiety_group(conn: &mut DBConnection, elem: &NewMoietyGroup) -> QueryResult<MoietyGroup> {
	diesel::insert_into(moiety_group::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_moiety_group(conn: &mut DBConnection, id: i64) -> QueryResult<MoietyGroup> {
	moiety_group::table.find(id)
		.first(conn)
}

pub fn update_moiety_group(conn: &mut DBConnection, id: i64, elem: &NewMoietyGroup) -> QueryResult<MoietyGroup> {
	diesel::update(moiety_group::table)
		.filter(moiety_group::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_moiety_group(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(moiety_group::table)
		.filter(moiety_group::id.eq(id))
		.execute(conn)
}

pub fn create_substructure_filter(conn: &mut DBConnection, elem: &NewSubstructureFilter) -> QueryResult<SubstructureFilter> {
	diesel::insert_into(substructure_filter::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_substructure_filter(conn: &mut DBConnection, id: i64) -> QueryResult<SubstructureFilter> {
	substructure_filter::table.find(id)
		.first(conn)
}

pub fn update_substructure_filter(conn: &mut DBConnection, id: i64, elem: &NewSubstructureFilter) -> QueryResult<SubstructureFilter> {
	diesel::update(substructure_filter::table)
		.filter(substructure_filter::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_substructure_filter(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(substructure_filter::table)
		.filter(substructure_filter::id.eq(id))
		.execute(conn)
}

pub fn create_substructure_filter_group(conn: &mut DBConnection, elem: &NewSubstructureFilterGroup) -> QueryResult<SubstructureFilterGroup> {
	diesel::insert_into(substructure_filter_group::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_substructure_filter_group(conn: &mut DBConnection, id: i64) -> QueryResult<SubstructureFilterGroup> {
	substructure_filter_group::table.find(id)
		.first(conn)
}

pub fn update_substructure_filter_group(conn: &mut DBConnection, id: i64, elem: &NewSubstructureFilterGroup) -> QueryResult<SubstructureFilterGroup> {
	diesel::update(substructure_filter_group::table)
		.filter(substructure_filter_group::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_substructure_filter_group(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(substructure_filter_group::table)
		.filter(substructure_filter_group::id.eq(id))
		.execute(conn)
}

pub fn create_reaction(conn: &mut DBConnection, elem: &NewReaction) -> QueryResult<Reaction> {
	diesel::insert_into(reaction::table)
		.values(elem)
		.get_result(conn)
}

pub fn get_reaction(conn: &mut DBConnection, id: i64) -> QueryResult<Reaction> {
	reaction::table
		.find(id)
		.first(conn)
}

pub fn get_reactions(conn: &mut DBConnection) -> QueryResult<impl Iterator<Item = QueryResult<Reaction>>> {
	reaction::table
		.load_iter::<_, DefaultLoadingMode>(conn)
}

pub fn get_reactions_with_experiment(conn: &mut DBConnection, exp: &Experiment) -> QueryResult<impl Iterator<Item = QueryResult<Reaction>>> {
	experiment::table
		.inner_join(experiment_frag::table
		.inner_join(experiment_frag_reactant::table
		.inner_join(reaction::table)))
		.filter(experiment::id.eq(exp.id))
		.select(Reaction::as_select())
		.load_iter::<_, DefaultLoadingMode>(conn)
}

pub fn update_reaction(conn: &mut DBConnection, id: i64, elem: &NewReaction) -> QueryResult<Reaction> {
	diesel::update(reaction::table)
		.filter(reaction::id.eq(id))
		.set(elem)
		.get_result(conn)
}

pub fn delete_reaction(conn: &mut DBConnection, id: i64) -> QueryResult<usize> {
	diesel::delete(reaction::table)
		.filter(reaction::id.eq(id))
		.execute(conn)
}

pub fn get_building_blocks<'a>(conn: &'a mut DBConnection) -> QueryResult<impl Iterator<Item = QueryResult<BuildingBlock>> + 'a> {
	building_block::table
		.select(BuildingBlock::as_select())
		.load_iter::<_, PgRowByRowLoadingMode>(conn)
}

impl BuildingBlock {
	pub fn get_with_reaction<'a>(conn: &'a mut DBConnection, reaction: &Reaction, frag_reactant_idx: i32) -> QueryResult<impl Iterator<Item = QueryResult<Self>> + 'a> {
		building_block::table
			.inner_join(building_block_reactant::table)
			.filter(building_block_reactant::id_reaction.eq(reaction.id))
			.filter(building_block_reactant::reactant_idx.ne(frag_reactant_idx))
			.select(Self::as_select())
			.load_iter::<_, PgRowByRowLoadingMode>(conn)
	}
}

// Building blocks that lead to products for a given experiment
impl ExportableBuildingBlock {
	pub fn get_with_experiment<'a>(conn: &'a mut DBConnection, exp: &Experiment) -> QueryResult<impl Iterator<Item = QueryResult<Self>> + 'a> {
		experiment::table
			.inner_join(experiment_frag::table
			.inner_join(experiment_frag_reactant::table
			.inner_join(experiment_product::table
			.inner_join(experiment_product_origin::table
			.inner_join(building_block_reactant::table
			.inner_join(building_block::table
			.inner_join(building_block_origin::table
			.inner_join(compound::table
			.inner_join(compound_provider::table
			.inner_join(experiment_selected_provider::table))))))))))
			.filter(experiment_selected_provider::id_experiment.eq(experiment::id))
			.filter(experiment::id.eq(exp.id))
			.group_by((experiment::id, building_block::id))
			.select(Self::as_select())
			.load_iter::<_, PgRowByRowLoadingMode>(conn)
	}

	pub fn get_with_experiment_and_descs<'a>(conn: &'a mut DBConnection, exp: &Experiment, descs: &ExperimentProductDescFilter) -> QueryResult<impl Iterator<Item = QueryResult<Self>> + 'a> {
		experiment::table
			.inner_join(experiment_frag::table
			.inner_join(experiment_frag_reactant::table
			.inner_join(experiment_product::table
			.inner_join(experiment_product_origin::table
			.inner_join(building_block_reactant::table
			.inner_join(building_block::table
			.inner_join(building_block_origin::table
			.inner_join(compound::table
			.inner_join(compound_provider::table
			.inner_join(experiment_selected_provider::table))))))))))
			.filter(experiment_selected_provider::id_experiment.eq(experiment::id))
			.filter(experiment::id.eq(exp.id))
			.filter(ExperimentProduct::predicate_all_descs(descs))
			.group_by((experiment::id, building_block::id))
			.select(Self::as_select())
			.load_iter::<_, PgRowByRowLoadingMode>(conn)
	}

	pub fn get_with_experiment_postproc_filter<'a>(conn: &'a mut DBConnection, exp_postproc_filter: &ExperimentPostprocFilter) -> QueryResult<impl Iterator<Item = QueryResult<Self>> + 'a> {
		experiment::table
			.inner_join(experiment_postproc_filter::table)
			.inner_join(experiment_frag::table
				.inner_join(experiment_frag_reactant::table
				.inner_join(experiment_product::table
				.inner_join(experiment_product_origin::table
				.inner_join(building_block_reactant::table
				.inner_join(building_block::table
				.inner_join(building_block_origin::table
				.inner_join(compound::table
				.inner_join(compound_provider::table
				.inner_join(experiment_selected_provider::table))))))))))
			.filter(experiment_selected_provider::id_experiment.eq(experiment::id))
			.filter(experiment::id.eq(exp_postproc_filter.id_experiment))
			.filter(experiment_postproc_filter::id.eq(exp_postproc_filter.id))
			.filter(predicate_experiment_postproc_filter())
			.group_by((experiment::id, building_block::id))
			.select(Self::as_select())
			.load_iter::<_, PgRowByRowLoadingMode>(conn)
	}
}

fn predicate_experiment_postproc_filter<'a, QS>() -> Box<dyn BoxableExpression<QS, DB, SqlType = diesel::sql_types::Bool>>
where
	QS: 'static,
	experiment_postproc_filter::desc_fsp3: BoxableExpression<QS, DB>,
	experiment_postproc_filter::desc_hba: BoxableExpression<QS, DB>,
	experiment_postproc_filter::desc_hbd: BoxableExpression<QS, DB>,
	experiment_postproc_filter::desc_clogp: BoxableExpression<QS, DB>,
	experiment_postproc_filter::desc_mw: BoxableExpression<QS, DB>,
	experiment_postproc_filter::desc_tpsa: BoxableExpression<QS, DB>,
	experiment_product::desc_fsp3: BoxableExpression<QS, DB>,
	experiment_product::desc_hba: BoxableExpression<QS, DB>,
	experiment_product::desc_hbd: BoxableExpression<QS, DB>,
	experiment_product::desc_clogp: BoxableExpression<QS, DB>,
	experiment_product::desc_mw: BoxableExpression<QS, DB>,
	experiment_product::desc_tpsa: BoxableExpression<QS, DB>,
{
	Box::new(AsExpression::<Bool>::as_expression(true)
		.and(experiment_postproc_filter::desc_fsp3.contains(experiment_product::desc_fsp3))
		.and(experiment_postproc_filter::desc_hba.contains(experiment_product::desc_hba))
		.and(experiment_postproc_filter::desc_hbd.contains(experiment_product::desc_hbd))
		.and(experiment_postproc_filter::desc_clogp.contains(experiment_product::desc_clogp))
		.and(experiment_postproc_filter::desc_mw.contains(experiment_product::desc_mw))
		.and(experiment_postproc_filter::desc_tpsa.contains(experiment_product::desc_tpsa)))
}

#[derive(Default, Deserialize)]
pub struct ExperimentProductDescFilter {
	pub fsp3: Option<(f32, f32)>,
	pub hba: Option<(i32, i32)>,
	pub hbd: Option<(i32, i32)>,
	pub clogp: Option<(f32, f32)>,
	pub mw: Option<(f32, f32)>,
	pub tpsa: Option<(f32, f32)>,
}

impl ExperimentProduct {
	fn predicate_all_descs<'a, QS>(descs: &ExperimentProductDescFilter) -> Box<dyn BoxableExpression<QS, DB, SqlType = diesel::sql_types::Bool>>
	where
		QS: 'static,
		experiment_product::desc_fsp3: BoxableExpression<QS, DB>,
		experiment_product::desc_hba: BoxableExpression<QS, DB>,
		experiment_product::desc_hbd: BoxableExpression<QS, DB>,
		experiment_product::desc_clogp: BoxableExpression<QS, DB>,
		experiment_product::desc_mw: BoxableExpression<QS, DB>,
		experiment_product::desc_tpsa: BoxableExpression<QS, DB>,
	{
		let mut expr = boxed_bool(true);

		if let Some(desc) = descs.fsp3 {
			expr = Box::new(expr.and(experiment_product::desc_fsp3.between(desc.0, desc.1)));
		}
		if let Some(desc) = descs.hba {
			expr = Box::new(expr.and(experiment_product::desc_hba.between(desc.0, desc.1)));
		}
		if let Some(desc) = descs.hbd {
			expr = Box::new(expr.and(experiment_product::desc_hbd.between(desc.0, desc.1)));
		}
		if let Some(desc) = descs.clogp {
			expr = Box::new(expr.and(experiment_product::desc_clogp.between(desc.0, desc.1)));
		}
		if let Some(desc) = descs.mw {
			expr = Box::new(expr.and(experiment_product::desc_mw.between(desc.0, desc.1)));
		}
		if let Some(desc) = descs.tpsa {
			expr = Box::new(expr.and(experiment_product::desc_tpsa.between(desc.0, desc.1)));
		}
	
		expr
	}

	pub fn count_with_experiment(conn: &mut DBConnection, exp: &Experiment) -> QueryResult<i64> {
		experiment_product::table
			.inner_join(experiment_frag_reactant::table
			.inner_join(experiment_frag::table))
			.filter(experiment_frag::id_experiment.eq(exp.id))
			.select::<AsSelect<Self, DB>>(Self::as_select())
			.count()
			.get_result(conn)
	}

	pub fn get_with_experiment<'a>(conn: &'a mut DBConnection, exp: &Experiment) -> QueryResult<impl Iterator<Item = QueryResult<Self>> + 'a>
	{
		experiment::table
			.inner_join(experiment_frag::table
			.inner_join(experiment_frag_reactant::table
			.inner_join(experiment_product::table)))
			.filter(experiment::id.eq(exp.id))
			.select(Self::as_select())
			.load_iter::<_, PgRowByRowLoadingMode>(conn)
	}

	pub fn count_with_experiment_and_descs(conn: &mut DBConnection, exp: &Experiment, descs: &ExperimentProductDescFilter) -> QueryResult<i64> {
		experiment_product::table
			.inner_join(experiment_frag_reactant::table
			.inner_join(experiment_frag::table))
			.filter(experiment_frag::id_experiment.eq(exp.id))
			.filter(Self::predicate_all_descs(descs))
			.count()
			.get_result(conn)
	}

	fn predicate_any_reaction<T: 'static>(reactions: &[Reaction]) -> Box<dyn BoxableExpression<T, DB, SqlType = Bool>>
	where
		diesel::dsl::Eq<building_block_reactant::id_reaction, i64>: BoxableExpression<T, DB, SqlType = Bool>
	{
		reactions
			.into_iter()
			.fold(boxed_bool(false), |expr, reaction| {
				Box::new(expr.or(building_block_reactant::id_reaction.eq(reaction.id)))
			})
	}

	pub fn get_with_experiment_and_reactions_and_descs(conn: &mut DBConnection, exp: &Experiment, reactions: &[Reaction], descs: &ExperimentProductDescFilter) -> QueryResult<impl Iterator<Item = QueryResult<Self>>> {
		experiment_product::table
			.inner_join(experiment_frag_reactant::table
			.	inner_join(experiment_frag::table))
			.inner_join(experiment_product_origin::table
				.inner_join(building_block_reactant::table))
			.filter(experiment_frag::id_experiment.eq(exp.id))
			.filter(Self::predicate_any_reaction(reactions))
			.filter(Self::predicate_all_descs(descs))
			.select(Self::as_select())
			.load_iter::<_, DefaultLoadingMode>(conn)
	}

	pub fn count_with_experiment_and_reactions_and_descs(conn: &mut DBConnection, exp: &Experiment, reactions: &[Reaction], descs: &ExperimentProductDescFilter) -> QueryResult<i64> {
		experiment_product::table
			.inner_join(experiment_frag_reactant::table
				.inner_join(experiment_frag::table))
			.inner_join(experiment_product_origin::table
				.inner_join(building_block_reactant::table))
			.filter(experiment_frag::id_experiment.eq(exp.id))
			.filter(Self::predicate_any_reaction(reactions))
			.filter(Self::predicate_all_descs(descs))
			.select(experiment_product::id)
			.count()
			.get_result(conn)
	}
}

impl ExportableExperimentProduct {
	pub fn get_with_experiment<'a>(conn: &'a mut DBConnection, exp: &Experiment) -> QueryResult<impl Iterator<Item = QueryResult<Self>> + 'a> {
		experiment::table
			.inner_join(experiment_frag::table
			.inner_join(experiment_frag_reactant::table
			.inner_join(experiment_product::table)))
			.filter(experiment::id.eq(exp.id))
			.select(Self::as_select())
			.load_iter::<_, PgRowByRowLoadingMode>(conn)
	}

	pub fn get_with_experiment_postproc_filter<'a>(conn: &'a mut DBConnection, exp_postproc_filter: &ExperimentPostprocFilter) -> QueryResult<impl Iterator<Item = QueryResult<Self>> + 'a> {
		experiment_postproc_filter::table
			.inner_join(experiment::table
			.inner_join(experiment_frag::table
			.inner_join(experiment_frag_reactant::table
			.inner_join(experiment_product::table))))
			.filter(experiment_postproc_filter::id.eq(exp_postproc_filter.id))
			.filter(predicate_experiment_postproc_filter())
			.select(Self::as_select())
			.load_iter::<_, PgRowByRowLoadingMode>(conn)
	}
}

impl Reaction {
	pub fn get_all(conn: &mut DBConnection) -> QueryResult<impl Iterator<Item = QueryResult<Self>>> {
		reaction::table
			.load_iter::<_, DefaultLoadingMode>(conn)
	}

	pub fn get_with_experiment(conn: &mut DBConnection, exp: &Experiment) -> QueryResult<impl Iterator<Item = QueryResult<Self>>> {
		reaction::table
			.inner_join(experiment_frag_reactant::table
			.inner_join(experiment_frag::table))
			.filter(experiment_frag::id_experiment.eq(exp.id))
			.select(Self::as_select())
			.load_iter::<_, DefaultLoadingMode>(conn)
	}
}

impl MergedBuildingBlockReactant {
	pub fn get_with_experiment_and_reaction<'a>(conn: &'a mut DBConnection, exp: &Experiment, reaction: &Reaction) -> QueryResult<impl Iterator<Item = QueryResult<Self>> + 'a>
	{
		experiment::table
			.inner_join(experiment_selected_provider::table)
			.inner_join(experiment_frag::table
				.inner_join(experiment_frag_reactant::table
				.inner_join(reaction::table
				.inner_join(building_block_reactant::table
				.inner_join(building_block::table
				.inner_join(building_block_origin::table
				.inner_join(compound::table
				.inner_join(compound_provider::table))))))))
			.filter(experiment_selected_provider::id_compound_provider.eq(compound_provider::id))
			.filter(experiment::id.eq(exp.id))
			.filter(reaction::id.eq(reaction.id)) 
			.filter(building_block_reactant::reactant_idx.ne(experiment_frag_reactant::reactant_idx))
			.group_by((experiment::id, reaction::id, building_block::id, building_block_reactant::id))
			.select(Self::as_select())
			.load_iter::<_, PgRowByRowLoadingMode>(conn)
	}
}

impl MergedExperimentFragReactant {
	pub fn get_with_experiment_and_idx(conn: &mut DBConnection, exp: &Experiment, idx: usize) -> QueryResult<impl Iterator<Item = QueryResult<Self>>> {
		experiment_frag::table
			.inner_join(experiment_frag_reactant::table)
			.filter(experiment_frag::id_experiment.eq(exp.id))
			.filter(experiment_frag::idx.eq(i32::try_from(idx).unwrap_or(-1)))
			.select(Self::as_select())
			.load_iter::<_, DefaultLoadingMode>(conn)
	}
}
