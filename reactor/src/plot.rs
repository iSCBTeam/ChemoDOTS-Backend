use std::{fs::{self, File}, io::{self, Read, Write}, fmt::Display};

use db::model::Experiment;
use itertools::Itertools;
use plotters::{prelude::*, element::PointCollection};
use chemodots_db as db;
use rayon::prelude::*;
use rust_decimal::prelude::*;
use serde_json::{Value, json};
use serde::{ser::Serialize, Deserialize};
use uuid::Uuid;

fn fmtdown(n: f64, ndigits: u32) -> String {
    let d = Decimal::from_f64_retain(n).unwrap();
    d.round_sf_with_strategy(ndigits, RoundingStrategy::ToNegativeInfinity)
        .unwrap()
        .normalize()
        .to_string()
}

fn fmtup(n: f64, ndigits: u32) -> String {
    let d = Decimal::from_f64_retain(n).unwrap();
    d.round_sf_with_strategy(ndigits, RoundingStrategy::ToPositiveInfinity)
        .unwrap()
        .normalize()
        .to_string()
}

fn gen_json<Var: Serialize>(graph_filename: &str, min: Var, max: Var) -> Result<(), String> {
	let json_filename = format!("{graph_filename}.json");
	let mut file = File::create(json_filename)
		.map_err(|_| format!("Failed to create json file for plot {graph_filename}"))?;
	let res_json = json!({
		"min": min,
		"max": max,
	});
    file.write_all(res_json.to_string().as_bytes())
		.map_err(|_| format!("Failed to write json data for plot {graph_filename}"))?;
	Ok(())
}

pub trait GenPlot {
	fn gen_plot();
}

impl GenPlot for &[i64] {
	fn gen_plot() {
		
	}
}

impl GenPlot for &[f64] {
	fn gen_plot() {
		
	}
}

fn draw_plot_i64(out_filename: &str, caption: &str, data: &[i64], font_family: &str) {
	let (x_min, x_max) = match data.iter().minmax() {
		itertools::MinMaxResult::NoElements => (0, 0),
		itertools::MinMaxResult::OneElement(a) => (*a, *a),
		itertools::MinMaxResult::MinMax(a, b) => (*a, *b),
	};

	let x_spec = (x_min..x_max)
		.into_segmented();

	let data_count = data.len();
	let data = data
		.into_iter()
		.filter_map(|x| x_spec.index_of(&SegmentValue::Exact(*x)))
		.counts()
		.into_iter()
		.map(|(index, count)| (
			x_spec.from_index(index).unwrap(),
			count as f64 * 100.0 / data_count as f64))
		.collect::<Vec<_>>();

	let y_max = data
		.iter()
		.map(|(_, count)| *count)
		.max_by(f64::total_cmp)
		.unwrap_or(0.0);
	let y_max = (y_max.ceil() as u64)
		.checked_next_multiple_of(2)
		.unwrap_or(0) as f64;
	let y_step = if y_max != 0.0 {
		y_max / 2.0
	} else {
		1.0
	};
	let y_spec = (0.0..(y_max + 1.0)).step(y_step);

	let mut raw_svg_content = String::new();
	let area = SVGBackend::with_string(&mut raw_svg_content, (300, 300)).into_drawing_area();

	area.fill(&WHITE).unwrap();

	let mut chart = ChartBuilder::on(&area)
		.margin(5)
		.caption(caption, (font_family, 30))
		.set_label_area_size(LabelAreaPosition::Left, 75)
		.set_label_area_size(LabelAreaPosition::Bottom, 25)
		.build_cartesian_2d(x_spec, y_spec)
		.unwrap();

	chart
		.configure_mesh()
		.disable_mesh()
		.disable_x_axis()
		.y_labels(3)
		.y_label_style((font_family, 30))
		.y_label_formatter(&|val| format!("{:.0}%", val))
		.draw()
		.unwrap();

	let actual = Histogram::vertical(&chart)
		.style(BLUE.filled())
		.margin(1)
		.data(data);

	chart.draw_series(actual).unwrap();

	chart.configure_series_labels().draw().unwrap();

	area
		.present()
		.expect("Unable to present the plot");

	drop(chart);
	drop(area);

	let mut file = File::create(out_filename)
		.expect("Unable to create the plot file");

	file.write_all(raw_svg_content.as_bytes())
		.expect("Unable to write the plot file contents");

	gen_json(out_filename, x_min, x_max)
		.expect("Unable to generate the plot info file");

	eprintln!("{caption}: {x_min} - {x_max}");
}

fn draw_plot(out_filename: &str, caption: &str, data: &[f64], font_family: &str) {
	let (x_min, x_max) = match data.iter().minmax() {
		itertools::MinMaxResult::NoElements => (0.0, 0.0),
		itertools::MinMaxResult::OneElement(a) => (*a, *a),
		itertools::MinMaxResult::MinMax(a, b) => (*a, *b),
	};

	let x_bar_count = 21;
	let x_step = (x_max - x_min) / x_bar_count as f64;
	let x_spec = (x_min..(x_max - 0.99 * x_step))
		.step(x_step)
		.use_floor()
		.into_segmented();

	let data_count = data.len();
	let data = data
		.into_iter()
		.filter_map(|x| x_spec.index_of(&SegmentValue::CenterOf(*x)))
		.counts()
		.into_iter()
		.map(|(index, count)| (
			x_spec.from_index(index).unwrap(),
			count as f64 * 100.0 / data_count as f64))
		.collect::<Vec<_>>();

	let y_max = data
		.iter()
		.map(|(_, count)| *count)
		.max_by(f64::total_cmp)
		.unwrap_or(0.0);
	let y_max = (y_max.ceil() as u64)
		.checked_next_multiple_of(2)
		.unwrap_or(0) as f64;
	let y_step = if y_max != 0.0 {
		y_max / 2.0
	} else {
		1.0
	};
	let y_spec = (0.0..(y_max + 1.0)).step(y_step);

	let area = SVGBackend::new(out_filename, (300, 300)).into_drawing_area();

	area.fill(&WHITE).unwrap();

	let mut chart = ChartBuilder::on(&area)
		.margin(5)
		.caption(caption, (font_family, 30))
		.set_label_area_size(LabelAreaPosition::Left, 75)
		.set_label_area_size(LabelAreaPosition::Bottom, 25)
		.build_cartesian_2d(x_spec, y_spec)
		.unwrap();

	chart
		.configure_mesh()
		.disable_mesh()
		.disable_x_axis()
		.y_labels(3)
		.y_label_style((font_family, 30))
		.y_label_formatter(&|val| format!("{:.0}%", val))
		.draw()
		.unwrap();

	let actual = Histogram::vertical(&chart)
		.style(BLUE.filled())
		.margin(1)
		.data(data);

	chart.draw_series(actual).unwrap();

	chart.configure_series_labels().draw().unwrap();

	area
		.present()
		.expect("Unable to present the plot");

	let x_min = fmtdown(x_min, 4);
	let x_max = fmtup(x_max, 4);
	gen_json(out_filename, &x_min, &x_max)
		.expect("Unable to generate the plot info file");

	eprintln!("{caption}: {x_min} - {x_max}");
}

static FONT_NOTO_SANS_REGULAR_RAW: &[u8] = include_bytes!("../../fonts/NotoSans-Regular.ttf");

pub fn gen_plots(db_pool: &db::DBPool, ent_experiment: &Experiment) {
	let mut conn = db_pool.get().unwrap();

	eprintln!("Generating plots...");

	let font_family = "Noto Sans";
	plotters::style::register_font(font_family, FontStyle::Normal, FONT_NOTO_SANS_REGULAR_RAW).map_err(|_| "Invalid font").unwrap();

	let prods = db::model::ExperimentProduct::get_with_experiment(&mut conn, &ent_experiment)
		.unwrap()
		.map(|e| e.unwrap())
		.collect_vec();

	let (dataset_fsp3, dataset_hba, dataset_hbd, dataset_clogp, dataset_mw, dataset_tpsa): (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>) = prods
		.iter()
		.map(|e| (
			e.desc_fsp3 as f64,
			e.desc_hba as i64,
			e.desc_hbd as i64,
			e.desc_clogp as f64,
			e.desc_mw as f64,
			e.desc_tpsa as f64,
		))
		.multiunzip();

	draw_plot("plot-fsp3.svg", "Fsp³", &dataset_fsp3, font_family);
	draw_plot_i64("plot-hba.svg", "HBA", &dataset_hba, font_family);
	draw_plot_i64("plot-hbd.svg", "HBD", &dataset_hbd, font_family);
	draw_plot("plot-clogp.svg", "cLogP", &dataset_clogp, font_family);
	draw_plot("plot-mw.svg", "MW", &dataset_mw, font_family);
	draw_plot("plot-tpsa.svg", "TPSA", &dataset_tpsa, font_family);
}
