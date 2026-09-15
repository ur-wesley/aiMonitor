use tauri::{PhysicalPosition, WebviewWindow};

pub const CURSOR_OFFSET: i32 = 16;
pub const EDGE_MARGIN: i32 = 8;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct WorkAreaBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn work_area_for_point(
    window: &WebviewWindow,
    x: f64,
    y: f64,
) -> Result<WorkAreaBounds, String> {
    let monitor = window
        .monitor_from_point(x, y)
        .map_err(|e| e.to_string())?
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| "no monitor found".to_string())?;

    let work = monitor.work_area();
    Ok(WorkAreaBounds {
        x: work.position.x,
        y: work.position.y,
        width: work.size.width,
        height: work.size.height,
    })
}

pub fn cursor_anchor(window: &WebviewWindow) -> Result<(i32, i32), String> {
    let cursor = window.cursor_position().map_err(|e| e.to_string())?;
    Ok((cursor.x.round() as i32, cursor.y.round() as i32))
}

pub fn clamp_position(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    work_area: WorkAreaBounds,
) -> (i32, i32) {
    let max_x = work_area.x
        + work_area.width.cast_signed()
        - width.cast_signed()
        - EDGE_MARGIN;
    let max_y = work_area.y
        + work_area.height.cast_signed()
        - height.cast_signed()
        - EDGE_MARGIN;

    let x = x.clamp(work_area.x + EDGE_MARGIN, max_x.max(work_area.x + EDGE_MARGIN));
    let y = y.clamp(work_area.y + EDGE_MARGIN, max_y.max(work_area.y + EDGE_MARGIN));
    (x, y)
}

pub fn fit_near_anchor(
    anchor_x: i32,
    anchor_y: i32,
    width: u32,
    height: u32,
    work_area: WorkAreaBounds,
) -> (i32, i32) {
    let width_i = width.cast_signed();
    let height_i = height.cast_signed();
    let right_limit = work_area.x + work_area.width.cast_signed() - EDGE_MARGIN;
    let bottom_limit = work_area.y + work_area.height.cast_signed() - EDGE_MARGIN;

    let mut x = anchor_x + CURSOR_OFFSET;
    let mut y = anchor_y + CURSOR_OFFSET;

    if x + width_i > right_limit {
        x = anchor_x - width_i - CURSOR_OFFSET;
    }

    if y + height_i > bottom_limit {
        y = anchor_y - height_i - CURSOR_OFFSET;
    }

    clamp_position(x, y, width, height, work_area)
}

pub fn place_window(
    window: &WebviewWindow,
    anchor: (i32, i32),
    work_area: WorkAreaBounds,
) -> Result<(), String> {
    let size = window.outer_size().map_err(|e| e.to_string())?;
    let max_height = work_area.height.saturating_sub((EDGE_MARGIN * 2).cast_unsigned());
    let height = size.height.min(max_height);
    let width = size.width;

    if height != size.height {
        window
            .set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }))
            .map_err(|e| e.to_string())?;
    }

    let (x, y) = fit_near_anchor(anchor.0, anchor.1, width, height, work_area);
    window
        .set_position(tauri::Position::Physical(PhysicalPosition { x, y }))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{fit_near_anchor, WorkAreaBounds, EDGE_MARGIN};

    const WA: WorkAreaBounds = WorkAreaBounds {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };

    #[test]
    fn fit_center_prefers_below_right() {
        let (x, y) = fit_near_anchor(960, 540, 340, 400, WA);
        assert_eq!(x, 976);
        assert_eq!(y, 556);
    }

    #[test]
    fn fit_flips_left_at_right_edge() {
        let (x, y) = fit_near_anchor(1900, 540, 340, 400, WA);
        assert_eq!(x, 1900 - 340 - super::CURSOR_OFFSET);
        assert_eq!(y, 556);
    }

    #[test]
    fn fit_flips_up_at_bottom_edge() {
        let (x, y) = fit_near_anchor(960, 1050, 340, 400, WA);
        assert_eq!(x, 976);
        assert_eq!(y, 1050 - 400 - super::CURSOR_OFFSET);
    }

    #[test]
    fn fit_clamps_to_work_area_origin() {
        let wa = WorkAreaBounds {
            x: 1920,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let (x, y) = fit_near_anchor(1930, 100, 340, 400, wa);
        assert!(x >= wa.x + EDGE_MARGIN);
        assert!(y >= wa.y + EDGE_MARGIN);
    }
}
