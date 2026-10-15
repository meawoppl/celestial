use rizzma::artist::{Collection, Line2D, Patch, Rgba};
use rizzma::{Axes, Figure};
use std::path::Path;

type PlotResult = std::result::Result<(), Box<dyn std::error::Error>>;

const TITLE_SIZE_PX: f64 = 19.0;
const MARKER_DIAMETER_PT: f64 = 6.0 * 72.0 / 100.0;
const HEAD_LENGTH_FRACTION: f64 = 0.35;
const HEAD_HALF_WIDTH_FRACTION: f64 = 0.2;

pub fn scatter_svg(
    points: &[(f64, f64)],
    path: &Path,
    title: &str,
    x_label: &str,
    y_label: &str,
) -> PlotResult {
    let (x_range, y_range) = padded_ranges(points);
    let mut fig = new_figure();
    let ax = fig.add_subplot(1, 1, 1);
    setup_axes(ax, title, x_label, y_label, &x_range, &y_range);
    draw_crosshairs(ax);
    draw_points(ax, points);
    fig.save_svg(path)?;
    Ok(())
}

pub fn histogram_svg(values: &[f64], path: &Path, title: &str, x_label: &str) -> PlotResult {
    if values.is_empty() {
        return Ok(());
    }
    let mut fig = new_figure();
    let ax = fig.add_subplot(1, 1, 1);
    let (counts, edges) = ax.hist(values, 20);
    let (Some(&x_min), Some(&x_max)) = (edges.first(), edges.last()) else {
        return Ok(());
    };
    let max_count = counts.iter().copied().fold(1.0, f64::max);
    setup_axes(
        ax,
        title,
        x_label,
        "Count",
        &(x_min, x_max),
        &(0.0, max_count + 1.0),
    );
    ax.grid(false);
    fig.save_svg(path)?;
    Ok(())
}

pub fn vector_map_svg(
    positions: &[(f64, f64)],
    vectors: &[(f64, f64)],
    path: &Path,
    title: &str,
    x_label: &str,
    y_label: &str,
    scale: f64,
) -> PlotResult {
    let (x_range, y_range) = padded_ranges(positions);
    let mut fig = new_figure();
    let ax = fig.add_subplot(1, 1, 1);
    setup_axes(ax, title, x_label, y_label, &x_range, &y_range);
    draw_points(ax, positions);
    draw_vectors(ax, positions, vectors, scale);
    fig.save_svg(path)?;
    Ok(())
}

fn new_figure() -> Figure {
    Figure::new(8.0, 6.0).with_dpi(100.0)
}

fn padded_ranges(points: &[(f64, f64)]) -> ((f64, f64), (f64, f64)) {
    if points.is_empty() {
        return ((-1.0, 1.0), (-1.0, 1.0));
    }
    let (mut x_min, mut x_max) = extent(points.iter().map(|p| p.0));
    let (mut y_min, mut y_max) = extent(points.iter().map(|p| p.1));
    let x_pad = (x_max - x_min).abs() * 0.1 + 1e-6;
    let y_pad = (y_max - y_min).abs() * 0.1 + 1e-6;
    x_min -= x_pad;
    x_max += x_pad;
    y_min -= y_pad;
    y_max += y_pad;
    ((x_min, x_max), (y_min, y_max))
}

fn extent(iter: impl Iterator<Item = f64>) -> (f64, f64) {
    iter.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
        (lo.min(v), hi.max(v))
    })
}

fn setup_axes(
    ax: &mut Axes,
    title: &str,
    x_label: &str,
    y_label: &str,
    x_range: &(f64, f64),
    y_range: &(f64, f64),
) {
    ax.set_title(title)
        .set_title_size(TITLE_SIZE_PX)
        .set_xlabel(x_label)
        .set_ylabel(y_label)
        .set_xlim(x_range.0, x_range.1)
        .set_ylim(y_range.0, y_range.1)
        .grid(true);
}

fn draw_crosshairs(ax: &mut Axes) {
    let color = Rgba::BLACK.with_alpha(0.3);
    ax.axhline_with(0.0, color, 1.0);
    ax.axvline_with(0.0, color, 1.0);
}

fn draw_points(ax: &mut Axes, points: &[(f64, f64)]) {
    let xs: Vec<f64> = points.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = points.iter().map(|p| p.1).collect();
    let coll = ax.scatter(&xs, &ys);
    let styled = std::mem::replace(coll, Collection::scatter(Vec::new()));
    *coll = styled
        .with_facecolors(vec![Rgba::BLUE])
        .with_sizes(vec![MARKER_DIAMETER_PT]);
}

fn draw_vectors(ax: &mut Axes, positions: &[(f64, f64)], vectors: &[(f64, f64)], scale: f64) {
    for (&(px, py), &(vx, vy)) in positions.iter().zip(vectors.iter()) {
        let (dx, dy) = (vx * scale, vy * scale);
        let len = libm::hypot(dx, dy);
        if len == 0.0 || !len.is_finite() {
            continue;
        }
        let dir = (dx / len, dy / len);
        let tip = [px + dx, py + dy];
        let hl = HEAD_LENGTH_FRACTION * len;
        let hw = HEAD_HALF_WIDTH_FRACTION * len;
        let neck = [tip[0] - hl * dir.0, tip[1] - hl * dir.1];
        ax.add_line(Line2D::new(vec![px, neck[0]], vec![py, neck[1]]).with_color(Rgba::RED));
        let left = [neck[0] - hw * dir.1, neck[1] + hw * dir.0];
        let right = [neck[0] + hw * dir.1, neck[1] - hw * dir.0];
        ax.add_patch(
            Patch::polygon(&[tip, left, right])
                .facecolor(Some(Rgba::RED))
                .edgecolor(Some(Rgba::RED)),
        );
    }
}
