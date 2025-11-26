use tao::window::Window;
use windows::Win32::{
    Foundation::RECT,
    Graphics::Gdi::{GetMonitorInfoW, MonitorFromRect, MONITORINFO, MONITOR_DEFAULTTONEAREST},
    UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
};

/// ウィンドウ位置のオフセット（ピクセル単位、96 DPI基準）
pub const WINDOW_OFFSET_X: i32 = 15;
pub const WINDOW_OFFSET_Y: i32 = 2;
pub const INDICATOR_OFFSET_X: f64 = 45.0;
pub const INDICATOR_OFFSET_Y: f64 = 2.0;

/// 作業領域を表す構造体
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkArea {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Default for WorkArea {
    fn default() -> Self {
        Self {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        }
    }
}

/// DPIスケールファクターを計算する（96 DPI = 1.0）
pub fn calculate_dpi_scale(dpi: u32) -> f64 {
    dpi as f64 / 96.0
}

/// 指定されたモニターのDPIスケールファクターを取得する
fn get_dpi_scale_factor(monitor: windows::Win32::Graphics::Gdi::HMONITOR) -> f64 {
    let mut dpi_x: u32 = 96;
    let mut dpi_y: u32 = 96;

    unsafe {
        // GetDpiForMonitor はWindows 8.1以降で利用可能
        let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y);
    }

    calculate_dpi_scale(dpi_x)
}

/// 候補ウィンドウの位置を計算する（純粋なロジック）
///
/// # Arguments
/// * `caret_top` - キャレットの上端
/// * `caret_left` - キャレットの左端
/// * `caret_bottom` - キャレットの下端
/// * `window_width` - ウィンドウの幅
/// * `window_height` - ウィンドウの高さ
/// * `work_area` - 作業領域の境界
/// * `dpi_scale` - DPIスケールファクター
///
/// # Returns
/// 計算された位置 (x, y)
pub fn calculate_candidate_position(
    caret_top: i32,
    caret_left: i32,
    caret_bottom: i32,
    window_width: i32,
    window_height: i32,
    work_area: &WorkArea,
    dpi_scale: f64,
) -> (i32, i32) {
    // オフセットをDPIスケールに合わせて調整
    let offset_x = (WINDOW_OFFSET_X as f64 * dpi_scale) as i32;
    let offset_y = (WINDOW_OFFSET_Y as f64 * dpi_scale) as i32;

    // 初期位置: キャレットの左下に配置
    let mut x = caret_left - offset_x;
    let mut y = caret_bottom + offset_y;

    // 下端がモニター外に出る場合は、キャレットの上に表示
    if y + window_height > work_area.bottom {
        y = caret_top - window_height - offset_y;
    }

    // 右端がモニター外に出る場合は、左にずらす
    if x + window_width > work_area.right {
        x = work_area.right - window_width;
    }

    // 左端がモニター外に出る場合は、作業領域の左端に配置
    if x < work_area.left {
        x = work_area.left;
    }

    // 上端がモニター外に出る場合は、作業領域の上端に配置
    if y < work_area.top {
        y = work_area.top;
    }

    (x, y)
}

/// インジケーターの位置を計算する（純粋なロジック）
pub fn calculate_indicator_position(
    caret_left: i32,
    caret_bottom: i32,
    dpi_scale: f64,
) -> (i32, i32) {
    let offset_x = (INDICATOR_OFFSET_X * dpi_scale) as i32;
    let offset_y = (INDICATOR_OFFSET_Y * dpi_scale) as i32;

    let x = caret_left - offset_x;
    let y = caret_bottom + offset_y;

    (x, y)
}

/// キャレット位置に基づいて候補ウィンドウの表示位置を計算する
///
/// # Arguments
/// * `top` - キャレットの上端（物理座標）
/// * `left` - キャレットの左端（物理座標）
/// * `bottom` - キャレットの下端（物理座標）
/// * `right` - キャレットの右端（物理座標）
/// * `window` - 候補ウィンドウ
///
/// # Returns
/// 候補ウィンドウの表示位置（物理座標）
pub fn get_candidate_window_position(
    top: i32,
    left: i32,
    bottom: i32,
    right: i32,
    window: &Window,
) -> (f64, f64) {
    // モニター情報を取得
    let monitor = unsafe {
        MonitorFromRect(
            &RECT {
                left,
                top,
                right,
                bottom,
            } as *const _,
            MONITOR_DEFAULTTONEAREST,
        )
    };

    let mut monitor_info = MONITORINFO::default();
    monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

    unsafe {
        let _ = GetMonitorInfoW(monitor, &mut monitor_info);
    }

    // DPIスケールファクターを取得
    let dpi_scale = get_dpi_scale_factor(monitor);

    // ウィンドウサイズを物理ピクセルで取得
    let window_size = window.inner_size();
    let window_width = window_size.width as i32;
    let window_height = window_size.height as i32;

    // 作業領域を構造体に変換
    let work_area = WorkArea {
        left: monitor_info.rcWork.left,
        top: monitor_info.rcWork.top,
        right: monitor_info.rcWork.right,
        bottom: monitor_info.rcWork.bottom,
    };

    let (x, y) = calculate_candidate_position(
        top,
        left,
        bottom,
        window_width,
        window_height,
        &work_area,
        dpi_scale,
    );

    (x as f64, y as f64)
}

/// インジケーターウィンドウの表示位置を計算する
///
/// # Arguments
/// * `left` - キャレットの左端（物理座標）
/// * `bottom` - キャレットの下端（物理座標）
///
/// # Returns
/// インジケーターウィンドウの表示位置（物理座標）
pub fn get_indicator_window_position(left: i32, bottom: i32) -> (f64, f64) {
    // モニター情報を取得してDPIスケールを考慮
    let monitor = unsafe {
        MonitorFromRect(
            &RECT {
                left,
                top: bottom - 1,
                right: left + 1,
                bottom,
            } as *const _,
            MONITOR_DEFAULTTONEAREST,
        )
    };

    let dpi_scale = get_dpi_scale_factor(monitor);

    let (x, y) = calculate_indicator_position(left, bottom, dpi_scale);

    (x as f64, y as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================
    // DPIスケール計算のテスト
    // ============================================

    #[test]
    fn test_dpi_scale_100_percent() {
        // Given: DPI = 96 (100%)
        // When: calculate_dpi_scale を呼び出し
        // Then: スケールファクター 1.0 を返す
        let scale = calculate_dpi_scale(96);
        assert!((scale - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_dpi_scale_125_percent() {
        // Given: DPI = 120 (125%)
        // When: calculate_dpi_scale を呼び出し
        // Then: スケールファクター 1.25 を返す
        let scale = calculate_dpi_scale(120);
        assert!((scale - 1.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_dpi_scale_150_percent() {
        // Given: DPI = 144 (150%)
        // When: calculate_dpi_scale を呼び出し
        // Then: スケールファクター 1.5 を返す
        let scale = calculate_dpi_scale(144);
        assert!((scale - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_dpi_scale_200_percent() {
        // Given: DPI = 192 (200%)
        // When: calculate_dpi_scale を呼び出し
        // Then: スケールファクター 2.0 を返す
        let scale = calculate_dpi_scale(192);
        assert!((scale - 2.0).abs() < f64::EPSILON);
    }

    // ============================================
    // 候補ウィンドウ位置計算のテスト
    // ============================================

    #[test]
    fn test_candidate_position_normal() {
        // Given: キャレットが画面中央、DPI 100%
        // When: calculate_candidate_position を呼び出し
        // Then: キャレットの左下に配置される
        let work_area = WorkArea {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };

        let (x, y) = calculate_candidate_position(
            500, // caret_top
            500, // caret_left
            520, // caret_bottom
            200, // window_width
            150, // window_height
            &work_area, 1.0, // dpi_scale
        );

        // x = caret_left - offset_x = 500 - 15 = 485
        // y = caret_bottom + offset_y = 520 + 2 = 522
        assert_eq!(x, 485);
        assert_eq!(y, 522);
    }

    #[test]
    fn test_candidate_position_bottom_overflow() {
        // Given: キャレットがモニター下端付近
        // When: calculate_candidate_position を呼び出し
        // Then: キャレットの上に配置される
        let work_area = WorkArea {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };

        let (x, y) = calculate_candidate_position(
            1000, // caret_top (モニター下端付近)
            500,  // caret_left
            1020, // caret_bottom
            200,  // window_width
            150,  // window_height
            &work_area, 1.0, // dpi_scale
        );

        // y + window_height > work_area.bottom の場合
        // y = caret_top - window_height - offset_y = 1000 - 150 - 2 = 848
        assert_eq!(y, 848);
    }

    #[test]
    fn test_candidate_position_right_overflow() {
        // Given: キャレットがモニター右端付近
        // When: calculate_candidate_position を呼び出し
        // Then: 左にずれて表示される
        let work_area = WorkArea {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };

        let (x, _y) = calculate_candidate_position(
            500,  // caret_top
            1850, // caret_left (右端付近)
            520,  // caret_bottom
            200,  // window_width
            150,  // window_height
            &work_area, 1.0, // dpi_scale
        );

        // x + window_width > work_area.right の場合
        // x = work_area.right - window_width = 1920 - 200 = 1720
        assert_eq!(x, 1720);
    }

    #[test]
    fn test_candidate_position_left_overflow() {
        // Given: キャレットがモニター左端付近
        // When: calculate_candidate_position を呼び出し
        // Then: 作業領域の左端に配置される
        let work_area = WorkArea {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };

        let (x, _y) = calculate_candidate_position(
            500, // caret_top
            10,  // caret_left (左端付近、offset後に負になる)
            520, // caret_bottom
            200, // window_width
            150, // window_height
            &work_area, 1.0, // dpi_scale
        );

        // x < work_area.left の場合
        // x = work_area.left = 0
        assert_eq!(x, 0);
    }

    #[test]
    fn test_candidate_position_with_150_percent_dpi() {
        // Given: DPI 150% (スケール 1.5)
        // When: calculate_candidate_position を呼び出し
        // Then: オフセットがスケールされる
        let work_area = WorkArea {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        };

        let (x, y) = calculate_candidate_position(
            500, // caret_top
            500, // caret_left
            520, // caret_bottom
            200, // window_width
            150, // window_height
            &work_area, 1.5, // dpi_scale (150%)
        );

        // offset_x = 15 * 1.5 = 22 (切り捨て)
        // offset_y = 2 * 1.5 = 3
        // x = 500 - 22 = 478
        // y = 520 + 3 = 523
        assert_eq!(x, 478);
        assert_eq!(y, 523);
    }

    #[test]
    fn test_candidate_position_top_overflow() {
        // Given: 上方向にオーバーフローする状況
        // When: calculate_candidate_position を呼び出し
        // Then: 作業領域の上端に配置される
        let work_area = WorkArea {
            left: 0,
            top: 50, // タスクバーがある場合など
            right: 1920,
            bottom: 1080,
        };

        let (_, y) = calculate_candidate_position(
            60,   // caret_top (上端付近)
            500,  // caret_left
            1050, // caret_bottom (下端にあふれる)
            200,  // window_width
            150,  // window_height
            &work_area, 1.0, // dpi_scale
        );

        // 下にあふれるので上に表示しようとするが、上端より小さくなる場合
        // y = caret_top - window_height - offset_y = 60 - 150 - 2 = -92
        // y < work_area.top なので y = work_area.top = 50
        assert_eq!(y, 50);
    }

    // ============================================
    // インジケーター位置計算のテスト
    // ============================================

    #[test]
    fn test_indicator_position_normal() {
        // Given: DPI 100%
        // When: calculate_indicator_position を呼び出し
        // Then: 正しいオフセットで配置される
        let (x, y) = calculate_indicator_position(500, 520, 1.0);

        // x = 500 - 45 = 455
        // y = 520 + 2 = 522
        assert_eq!(x, 455);
        assert_eq!(y, 522);
    }

    #[test]
    fn test_indicator_position_with_200_percent_dpi() {
        // Given: DPI 200%
        // When: calculate_indicator_position を呼び出し
        // Then: オフセットが2倍になる
        let (x, y) = calculate_indicator_position(500, 520, 2.0);

        // x = 500 - (45 * 2) = 500 - 90 = 410
        // y = 520 + (2 * 2) = 520 + 4 = 524
        assert_eq!(x, 410);
        assert_eq!(y, 524);
    }

    // ============================================
    // 境界値テスト
    // ============================================

    #[test]
    fn test_boundary_zero_offset() {
        // Given: オフセット 0 を想定（DPIスケール 0 は実際には発生しないがロジック確認用）
        // When: 非常に小さいDPIスケールで計算
        // Then: 最小限のオフセットで配置
        let work_area = WorkArea::default();
        let (x, y) = calculate_candidate_position(100, 100, 120, 200, 150, &work_area, 0.5);

        // offset_x = 15 * 0.5 = 7
        // offset_y = 2 * 0.5 = 1
        // x = 100 - 7 = 93
        // y = 120 + 1 = 121
        assert_eq!(x, 93);
        assert_eq!(y, 121);
    }

    #[test]
    fn test_work_area_default() {
        // Given: WorkArea のデフォルト値
        // Then: 1920x1080 のフルHD解像度
        let work_area = WorkArea::default();
        assert_eq!(work_area.left, 0);
        assert_eq!(work_area.top, 0);
        assert_eq!(work_area.right, 1920);
        assert_eq!(work_area.bottom, 1080);
    }
}
