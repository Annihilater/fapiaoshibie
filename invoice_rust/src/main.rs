mod config;
mod extractor;
mod gui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 900.0])
            .with_min_inner_size([1100.0, 800.0])
            .with_title("发票识别工具"),
        ..Default::default()
    };

    eframe::run_native(
        "发票识别工具",
        options,
        Box::new(|cc| {
            // 配置中文字体
            setup_custom_fonts(&cc.egui_ctx);
            Box::new(gui::InvoiceApp::new(cc))
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    use egui::{FontFamily, FontId};
    
    let mut fonts = egui::FontDefinitions::default();
    
    // 尝试从系统加载中文字体
    let font_loaded = load_system_chinese_font(&mut fonts);
    
    if font_loaded {
        // 设置默认字体大小
        ctx.style_mut(|style| {
            style.text_styles.insert(
                egui::TextStyle::Body,
                FontId::new(14.0, FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                FontId::new(14.0, FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Heading,
                FontId::new(20.0, FontFamily::Proportional),
            );
        });
        
        ctx.set_fonts(fonts);
    }
}

fn load_system_chinese_font(fonts: &mut egui::FontDefinitions) -> bool {
    use egui::{FontFamily, FontData};
    
    // macOS 中文字体路径
    #[cfg(target_os = "macos")]
    let font_paths = vec![
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/Library/Fonts/Arial Unicode.ttf",
    ];
    
    // Windows 中文字体路径
    #[cfg(target_os = "windows")]
    let font_paths = vec![
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simhei.ttf",
        "C:\\Windows\\Fonts\\simsun.ttc",
    ];
    
    // Linux 中文字体路径
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let font_paths = vec![
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/arphic/uming.ttc",
    ];
    
    for font_path in font_paths {
        if let Ok(font_data) = std::fs::read(font_path) {
            fonts.font_data.insert(
                "chinese_font".to_owned(),
                FontData::from_owned(font_data),
            );
            
            // 将中文字体添加到字体族首位
            if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
                family.insert(0, "chinese_font".to_owned());
            }
            if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
                family.insert(0, "chinese_font".to_owned());
            }
            
            return true;
        }
    }
    
    // 如果所有路径都失败，使用默认字体（不会崩溃，但中文可能显示为方块）
    false
}
