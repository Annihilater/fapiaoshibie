use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::config::Config;
use crate::extractor;

// 专业配色系统
const BG_PRIMARY: egui::Color32 = egui::Color32::from_rgb(18, 18, 18);
const BG_CARD: egui::Color32 = egui::Color32::from_rgb(30, 30, 30);
const BG_INPUT: egui::Color32 = egui::Color32::from_rgb(40, 40, 40);
const BG_LOG: egui::Color32 = egui::Color32::from_rgb(12, 12, 12);
const ACCENT_SUCCESS: egui::Color32 = egui::Color32::from_rgb(0, 200, 83);
const ACCENT_TECH: egui::Color32 = egui::Color32::from_rgb(41, 121, 255);
const TEXT_HIGH: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);
const TEXT_MEDIUM: egui::Color32 = egui::Color32::from_rgb(158, 158, 158);
const TEXT_LOW: egui::Color32 = egui::Color32::from_rgb(97, 97, 97);
const BORDER: egui::Color32 = egui::Color32::from_rgb(45, 45, 45);

#[derive(Default, Clone)]
struct ProcessStats {
    total_files: usize,
    pdf_files: usize,
    seller_recognized: usize,
    amount_recognized: usize,
    elapsed_time: f64,
}

#[derive(Default, Clone)]
struct ProcessProgress {
    total: usize,
    completed: usize,
    is_active: bool,
}

impl ProcessStats {
    fn pdf_rate(&self) -> f64 {
        if self.total_files == 0 { 0.0 }
        else { (self.pdf_files as f64 / self.total_files as f64) * 100.0 }
    }
    
    fn accuracy_rate(&self) -> f64 {
        if self.pdf_files == 0 { 0.0 }
        else {
            let recognized = self.seller_recognized.min(self.amount_recognized);
            (recognized as f64 / self.pdf_files as f64) * 100.0
        }
    }
}

pub struct InvoiceApp {
    invoice_dir: String,
    buyer_keyword: String,
    output_path: String,
    log_messages: Vec<String>,
    is_processing: bool,
    status_message: String,
    result_receiver: Option<mpsc::Receiver<Result<extractor::ProcessResult, String>>>,
    progress_receiver: Option<mpsc::Receiver<extractor::ProgressMessage>>,
    browse_dir_clicked: bool,
    browse_output_clicked: bool,
    open_result_clicked: bool,
    start_time: Option<Instant>,
    stats: ProcessStats,
    show_result: bool,
    result_file_path: String,
    result_data: Vec<extractor::InvoiceFile>,
    show_table: bool,
    config: Config,
    progress: ProcessProgress,
}

impl Default for InvoiceApp {
    fn default() -> Self {
        let config = Config::load();
        Self {
            invoice_dir: String::new(),
            buyer_keyword: String::new(),
            output_path: String::new(),
            log_messages: Vec::new(),
            is_processing: false,
            status_message: "就绪".to_string(),
            result_receiver: None,
            progress_receiver: None,
            browse_dir_clicked: false,
            browse_output_clicked: false,
            open_result_clicked: false,
            start_time: None,
            stats: ProcessStats::default(),
            show_result: false,
            result_file_path: String::new(),
            result_data: Vec::new(),
            show_table: true,
            config,
            progress: ProcessProgress::default(),
        }
    }
}

impl InvoiceApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn log(&mut self, message: String) {
        self.log_messages.push(message);
        if self.log_messages.len() > 200 {
            self.log_messages.remove(0);
        }
    }

    fn start_processing(&mut self) {
        if self.invoice_dir.is_empty() {
            self.log("❌ 请选择发票目录".to_string());
            return;
        }

        if self.output_path.is_empty() {
            self.output_path = format!("{}/发票清单.xlsx", self.invoice_dir);
        }

        self.is_processing = true;
        self.show_result = false;
        self.status_message = "识别中...".to_string();
        self.log_messages.clear();
        self.stats = ProcessStats::default();
        self.start_time = Some(Instant::now());

        let invoice_dir = self.invoice_dir.clone();
        let buyer_keyword = self.buyer_keyword.clone();
        let output_path = self.output_path.clone();
        let thread_count = self.config.get_thread_pool_size();

        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
        self.log("⚡ 开始处理发票文件".to_string());
        self.log(format!("📂 目录: {}", Self::format_path(&invoice_dir)));
        if !buyer_keyword.is_empty() {
            self.log(format!("🏢 关键词: {}", buyer_keyword));
        } else {
            self.log("🏢 关键词: 未设置（自动识别）".to_string());
        }
        self.log(format!("🔧 线程数: {} 个并行处理", thread_count));
        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());

        // 创建结果通道和进度通道
        let (result_tx, result_rx) = mpsc::channel();
        let (progress_tx, progress_rx) = mpsc::channel();

        // 初始化进度
        self.progress = ProcessProgress {
            total: 0,
            completed: 0,
            is_active: true,
        };

        thread::spawn(move || {
            let base_path = PathBuf::from(&invoice_dir);
            let output_path_buf = PathBuf::from(&output_path);
            
            let buyer_kw = if buyer_keyword.is_empty() {
                None
            } else {
                Some(buyer_keyword.as_str())
            };

            let result = extractor::process_invoices_with_progress(
                &base_path,
                buyer_kw,
                Some(&output_path_buf),
                Some(thread_count),
                Some(progress_tx),
            );

            let _ = result_tx.send(result);
        });

        self.result_receiver = Some(result_rx);
        self.progress_receiver = Some(progress_rx);
    }

    fn check_progress(&mut self) {
        // 收集所有进度消息
        let mut messages = Vec::new();
        if let Some(ref rx) = self.progress_receiver {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }
        
        // 处理收集到的消息
        for msg in messages {
            match msg {
                extractor::ProgressMessage::Started { total } => {
                    self.progress.total = total;
                    self.progress.completed = 0;
                    self.progress.is_active = true;
                }
                extractor::ProgressMessage::FileCompleted { filename: _, completed, total } => {
                    self.progress.completed = completed;
                    self.progress.total = total;
                }
                extractor::ProgressMessage::Finished => {
                    self.progress.is_active = false;
                }
                extractor::ProgressMessage::Log(log_msg) => {
                    self.log(log_msg);
                }
            }
        }
    }

    fn check_result(&mut self) {
        if let Some(ref rx) = self.result_receiver {
            if let Ok(result) = rx.try_recv() {
                self.is_processing = false;
                self.progress.is_active = false;
                let elapsed = self.start_time.map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0);
                
                match result {
                    Ok(process_result) => {
                        self.result_file_path = process_result.output_file.clone();
                        self.result_data = process_result.invoices.clone();
                        self.stats.elapsed_time = elapsed;
                        
                        self.stats.total_files = self.result_data.len();
                        self.stats.pdf_files = self.result_data.iter()
                            .filter(|inv| inv.file_type == "PDF")
                            .count();
                        self.stats.seller_recognized = self.result_data.iter()
                            .filter(|inv| !inv.info.seller.is_empty() && !inv.info.seller.starts_with('*'))
                            .count();
                        self.stats.amount_recognized = self.result_data.iter()
                            .filter(|inv| !inv.info.amount.is_empty())
                            .count();
                        
                        self.show_result = true;
                        self.show_table = true;
                        
                        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
                        self.log("✅ 处理完成!".to_string());
                        self.log(format!("📊 总文件: {} | PDF: {} | 成功率: {:.1}% | 耗时: {:.2}s", 
                            self.stats.total_files, 
                            self.stats.pdf_files,
                            self.stats.accuracy_rate(),
                            elapsed));
                        self.log(format!("💾 输出: {}", Self::format_path(&process_result.output_file)));
                        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
                        
                        self.status_message = "识别完成".to_string();
                    }
                    Err(e) => {
                        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
                        self.log(format!("❌ 处理失败: {}", e));
                        self.log("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
                        self.status_message = "处理失败".to_string();
                    }
                }
                self.result_receiver = None;
                self.progress_receiver = None;
            }
        }
    }
    
    fn format_path(path: &str) -> String {
        if path.len() > 60 {
            let start = &path[..30];
            let end = &path[path.len()-27..];
            format!("{}...{}", start, end)
        } else {
            path.to_string()
        }
    }
    
    fn render_data_card(&self, ui: &mut egui::Ui, label: &str, value: String, unit: &str, color: egui::Color32) {
        egui::Frame::none()
            .fill(BG_CARD)
            .stroke(egui::Stroke::new(1.0, BORDER))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::symmetric(16.0, 14.0))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(label)
                            .size(11.0)
                            .color(TEXT_MEDIUM)
                    );
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&value)
                                .size(26.0)
                                .color(color)
                                .strong()
                                .family(egui::FontFamily::Monospace)
                        );
                        if !unit.is_empty() {
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new(unit)
                                    .size(13.0)
                                    .color(TEXT_LOW)
                            );
                        }
                    });
                });
            });
    }
}

impl eframe::App for InvoiceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.browse_dir_clicked {
            self.browse_dir_clicked = false;
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                self.invoice_dir = path.to_string_lossy().to_string();
            }
        }
        
        if self.browse_output_clicked {
            self.browse_output_clicked = false;
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("发票清单.xlsx")
                .add_filter("Excel文件", &["xlsx"])
                .save_file()
            {
                self.output_path = path.to_string_lossy().to_string();
            }
        }
        
        if self.open_result_clicked {
            self.open_result_clicked = false;
            if !self.result_file_path.is_empty() {
                #[cfg(target_os = "macos")]
                let _ = std::process::Command::new("open").arg(&self.result_file_path).spawn();
                
                #[cfg(target_os = "windows")]
                let _ = std::process::Command::new("cmd").args(&["/C", "start", "", &self.result_file_path]).spawn();
                
                #[cfg(target_os = "linux")]
                let _ = std::process::Command::new("xdg-open").arg(&self.result_file_path).spawn();
            }
        }

        if self.is_processing {
            self.check_progress();
            self.check_result();
            ctx.request_repaint();
        }

        ctx.style_mut(|style| {
            style.visuals.dark_mode = true;
            style.visuals.panel_fill = BG_PRIMARY;
            
            style.visuals.widgets.inactive.bg_fill = BG_INPUT;
            style.visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);
            style.visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
            style.visuals.widgets.inactive.fg_stroke.color = TEXT_HIGH;
            
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(50, 50, 50);
            style.visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, ACCENT_TECH);
            
            style.visuals.widgets.active.bg_fill = BG_INPUT;
            style.visuals.widgets.active.bg_stroke = egui::Stroke::new(2.0, ACCENT_TECH);
            
            style.visuals.selection.bg_fill = ACCENT_TECH.linear_multiply(0.3);
            
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(16.0, 8.0);
            style.spacing.scroll.bar_width = 8.0;  // 滚动条宽度
            style.spacing.scroll.bar_inner_margin = 2.0;
            style.spacing.scroll.bar_outer_margin = 2.0;
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(BG_PRIMARY)
                .inner_margin(egui::Margin::symmetric(24.0, 20.0)))
            .show(ctx, |ui| {
                // 使用ScrollArea包裹整个内容
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        // 顶部标题
                        ui.vertical_centered(|ui| {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("发票识别工具")
                                    .size(28.0)
                                    .color(TEXT_HIGH)
                                    .strong()
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new("自动提取PDF发票信息 · 生成Excel清单")
                                    .size(13.0)
                                    .color(TEXT_MEDIUM.gamma_multiply(0.8))
                            );
                            ui.add_space(20.0);
                        });

                        // 上半部分：左右分栏（配置 + 统计日志）
                        ui.horizontal_top(|ui| {
                            // 左侧配置区 35%
                            ui.vertical(|ui| {
                                ui.set_width(ui.available_width() * 0.35);
                                
                                egui::Frame::none()
                                    .fill(BG_CARD)
                                    .stroke(egui::Stroke::new(1.0, BORDER))
                                    .rounding(egui::Rounding::same(12.0))
                                    .inner_margin(egui::Margin::same(20.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                egui::RichText::new("配置")
                                                    .size(16.0)
                                                    .color(TEXT_HIGH)
                                                    .strong()
                                            );
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.label(
                                                    egui::RichText::new(format!("🔧 {} 线程", self.config.get_thread_pool_size()))
                                                        .size(11.0)
                                                        .color(ACCENT_TECH)
                                                );
                                            });
                                        });
                                        ui.add_space(16.0);

                                        // 发票目录
                                        ui.label(egui::RichText::new("发票目录").size(12.0).color(TEXT_MEDIUM));
                                        ui.add_space(6.0);
                                        ui.horizontal(|ui| {
                                            ui.text_edit_singleline(&mut self.invoice_dir);
                                            if ui.button(egui::RichText::new("📁").size(15.0)).clicked() {
                                                self.browse_dir_clicked = true;
                                            }
                                        });
                                        
                                        ui.add_space(12.0);

                                        // 购买方关键词
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("购买方关键词").size(12.0).color(TEXT_MEDIUM));
                                            ui.label(egui::RichText::new("(可选)").size(10.0).color(TEXT_LOW));
                                        });
                                        ui.add_space(6.0);
                                        ui.text_edit_singleline(&mut self.buyer_keyword);
                                        
                                        ui.add_space(12.0);

                                        // 输出文件
                                        ui.label(egui::RichText::new("输出文件").size(12.0).color(TEXT_MEDIUM));
                                        ui.add_space(6.0);
                                        ui.horizontal(|ui| {
                                            ui.text_edit_singleline(&mut self.output_path);
                                            if ui.button(egui::RichText::new("💾").size(15.0)).clicked() {
                                                self.browse_output_clicked = true;
                                            }
                                        });
                                        
                                        ui.add_space(20.0);

                                        // 开始按钮
                                        let button_text = if self.is_processing { "⏳ 识别中..." } else { "🚀 开始识别" };
                                        let button_color = if self.is_processing { TEXT_LOW } else { ACCENT_SUCCESS };
                                        
                                        let button = egui::Button::new(
                                            egui::RichText::new(button_text).size(15.0).color(TEXT_HIGH).strong()
                                        )
                                        .fill(button_color)
                                        .rounding(egui::Rounding::same(8.0))
                                        .min_size(egui::vec2(ui.available_width(), 44.0));
                                        
                                        if ui.add_enabled(!self.is_processing, button).clicked() {
                                            self.start_processing();
                                        }

                                        // 进度显示
                                        if self.progress.is_active && self.progress.total > 0 {
                                            ui.add_space(16.0);
                                            
                                            // 进度文本
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    egui::RichText::new(format!("进度: {}/{}", 
                                                        self.progress.completed, 
                                                        self.progress.total))
                                                        .size(12.0)
                                                        .color(TEXT_MEDIUM)
                                                );
                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    let percentage = if self.progress.total > 0 {
                                                        (self.progress.completed as f64 / self.progress.total as f64 * 100.0) as usize
                                                    } else {
                                                        0
                                                    };
                                                    ui.label(
                                                        egui::RichText::new(format!("{}%", percentage))
                                                            .size(12.0)
                                                            .color(ACCENT_TECH)
                                                            .strong()
                                                    );
                                                });
                                            });
                                            
                                            ui.add_space(6.0);
                                            
                                            // 进度条
                                            let progress_ratio = if self.progress.total > 0 {
                                                self.progress.completed as f32 / self.progress.total as f32
                                            } else {
                                                0.0
                                            };
                                            
                                            let progress_bar_height = 8.0;
                                            let (rect, _) = ui.allocate_exact_size(
                                                egui::vec2(ui.available_width(), progress_bar_height),
                                                egui::Sense::hover()
                                            );
                                            
                                            // 背景
                                            ui.painter().rect_filled(
                                                rect,
                                                egui::Rounding::same(4.0),
                                                BG_INPUT
                                            );
                                            
                                            // 进度
                                            if progress_ratio > 0.0 {
                                                let progress_rect = egui::Rect::from_min_size(
                                                    rect.min,
                                                    egui::vec2(rect.width() * progress_ratio, rect.height())
                                                );
                                                ui.painter().rect_filled(
                                                    progress_rect,
                                                    egui::Rounding::same(4.0),
                                                    ACCENT_SUCCESS
                                                );
                                            }
                                        }
                                    });
                            });
                            
                            ui.add_space(16.0);

                            // 右侧区域 65%
                            ui.vertical(|ui| {
                                // 统计卡片
                                if self.show_result {
                                    ui.horizontal(|ui| {
                                        let card_width = (ui.available_width() - 24.0) / 4.0;
                                        ui.allocate_ui_with_layout(
                                            egui::vec2(card_width, 72.0),
                                            egui::Layout::top_down(egui::Align::Center),
                                            |ui| {
                                                self.render_data_card(ui, "总文件数", 
                                                    self.stats.total_files.to_string(), "", ACCENT_TECH);
                                            }
                                        );
                                        ui.add_space(8.0);
                                        
                                        ui.allocate_ui_with_layout(
                                            egui::vec2(card_width, 72.0),
                                            egui::Layout::top_down(egui::Align::Center),
                                            |ui| {
                                                self.render_data_card(ui, "PDF成功率", 
                                                    format!("{:.0}", self.stats.pdf_rate()), "%", ACCENT_SUCCESS);
                                            }
                                        );
                                        ui.add_space(8.0);
                                        
                                        ui.allocate_ui_with_layout(
                                            egui::vec2(card_width, 72.0),
                                            egui::Layout::top_down(egui::Align::Center),
                                            |ui| {
                                                self.render_data_card(ui, "识别成功率", 
                                                    format!("{:.0}", self.stats.accuracy_rate()), "%", ACCENT_SUCCESS);
                                            }
                                        );
                                        ui.add_space(8.0);
                                        
                                        ui.allocate_ui_with_layout(
                                            egui::vec2(card_width, 72.0),
                                            egui::Layout::top_down(egui::Align::Center),
                                            |ui| {
                                                self.render_data_card(ui, "耗时", 
                                                    format!("{:.2}", self.stats.elapsed_time), "秒", TEXT_MEDIUM);
                                            }
                                        );
                                    });
                                    
                                    ui.add_space(12.0);
                                    
                                    // 按钮组
                                    ui.horizontal(|ui| {
                                        let toggle_text = if self.show_table { "📋 隐藏表格" } else { "📋 显示表格" };
                                        if ui.button(egui::RichText::new(toggle_text).size(13.0)).clicked() {
                                            self.show_table = !self.show_table;
                                        }
                                        
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button(egui::RichText::new("📊 打开 Excel").size(13.0).color(TEXT_HIGH))
                                                .clicked() {
                                                self.open_result_clicked = true;
                                            }
                                        });
                                    });
                                    
                                    ui.add_space(12.0);
                                }
                                
                                // 运行日志
                                egui::Frame::none()
                                    .fill(BG_CARD)
                                    .stroke(egui::Stroke::new(1.0, BORDER))
                                    .rounding(egui::Rounding::same(12.0))
                                    .inner_margin(egui::Margin::same(16.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(egui::RichText::new("运行日志").size(15.0).color(TEXT_HIGH).strong());
                                            
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if !self.log_messages.is_empty() {
                                                    if ui.small_button("清除").clicked() {
                                                        self.log_messages.clear();
                                                    }
                                                }
                                            });
                                        });
                                        
                                        ui.add_space(8.0);
                                        
                                        egui::Frame::none()
                                            .fill(BG_LOG)
                                            .rounding(egui::Rounding::same(8.0))
                                            .inner_margin(egui::Margin::same(12.0))
                                            .show(ui, |ui| {
                                                let log_height = if self.show_result { 260.0 } else { 360.0 };
                                                egui::ScrollArea::vertical()
                                                    .max_height(log_height)
                                                    .stick_to_bottom(true)
                                                    .show(ui, |ui| {
                                                        if self.log_messages.is_empty() {
                                                            ui.vertical_centered(|ui| {
                                                                ui.add_space(100.0);
                                                                ui.label(egui::RichText::new("📝").size(48.0).color(TEXT_LOW));
                                                                ui.add_space(8.0);
                                                                ui.label(egui::RichText::new("填写配置后点击开始识别").size(13.0).color(TEXT_LOW));
                                                            });
                                                        } else {
                                                            for msg in &self.log_messages {
                                                                let color = if msg.contains("✅") {
                                                                    ACCENT_SUCCESS
                                                                } else if msg.contains("❌") {
                                                                    egui::Color32::from_rgb(255, 82, 82)
                                                                } else if msg.contains("⚡") {
                                                                    ACCENT_TECH
                                                                } else if msg.starts_with("━") {
                                                                    BORDER
                                                                } else {
                                                                    TEXT_MEDIUM
                                                                };
                                                                
                                                                ui.label(
                                                                    egui::RichText::new(msg.as_str())
                                                                        .size(11.5)
                                                                        .color(color)
                                                                        .family(egui::FontFamily::Monospace)
                                                                );
                                                            }
                                                        }
                                                    });
                                            });
                                    });
                            });
                        });

                        // 下半部分：全宽表格区域
                        if self.show_result && self.show_table && !self.result_data.is_empty() {
                            ui.add_space(20.0);
                            
                            egui::Frame::none()
                                .fill(BG_CARD)
                                .stroke(egui::Stroke::new(1.0, BORDER))
                                .rounding(egui::Rounding::same(12.0))
                                .inner_margin(egui::Margin::same(20.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("发票数据 (共 {} 条)", self.result_data.len()))
                                                .size(16.0)
                                                .color(TEXT_HIGH)
                                                .strong()
                                        );
                                        
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.label(
                                                egui::RichText::new("可横向滚动查看更多列")
                                                    .size(11.0)
                                                    .color(TEXT_LOW)
                                                    .italics()
                                            );
                                        });
                                    });
                                    
                                    ui.add_space(12.0);
                                    
                                    // 表格区域
                                    egui::ScrollArea::both()
                                        .auto_shrink([false; 2])
                                        .max_height(400.0)
                                        .show(ui, |ui| {
                                            use egui_extras::{TableBuilder, Column};
                                            
                                            TableBuilder::new(ui)
                                                .striped(true)
                                                .resizable(true)
                                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                                .column(Column::exact(50.0))   // 序号
                                                .column(Column::initial(200.0).at_least(150.0))  // 文件名
                                                .column(Column::initial(160.0).at_least(140.0))  // 发票号码
                                                .column(Column::initial(100.0).at_least(90.0))   // 日期
                                                .column(Column::initial(150.0).at_least(120.0))  // 购买方
                                                .column(Column::initial(150.0).at_least(120.0))  // 销售方
                                                .column(Column::initial(100.0).at_least(80.0))   // 金额
                                                .column(Column::initial(120.0).at_least(100.0))  // 备注
                                                .header(28.0, |mut header| {
                                                    header.col(|ui| {
                                                        ui.strong("序号");
                                                    });
                                                    header.col(|ui| {
                                                        ui.strong("文件名");
                                                    });
                                                    header.col(|ui| {
                                                        ui.strong("发票号码");
                                                    });
                                                    header.col(|ui| {
                                                        ui.strong("开票日期");
                                                    });
                                                    header.col(|ui| {
                                                        ui.strong("购买方");
                                                    });
                                                    header.col(|ui| {
                                                        ui.strong("销售方");
                                                    });
                                                    header.col(|ui| {
                                                        ui.strong("金额");
                                                    });
                                                    header.col(|ui| {
                                                        ui.strong("备注");
                                                    });
                                                })
                                                .body(|mut body| {
                                                    for (idx, inv) in self.result_data.iter().enumerate() {
                                                        body.row(24.0, |mut row| {
                                                            row.col(|ui| {
                                                                ui.label(egui::RichText::new((idx + 1).to_string()).size(11.0).color(TEXT_LOW));
                                                            });
                                                            row.col(|ui| {
                                                                ui.label(egui::RichText::new(&inv.filename).size(11.0).color(TEXT_HIGH));
                                                            });
                                                            row.col(|ui| {
                                                                ui.label(egui::RichText::new(&inv.info.invoice_number).size(11.0).color(TEXT_MEDIUM).family(egui::FontFamily::Monospace));
                                                            });
                                                            row.col(|ui| {
                                                                ui.label(egui::RichText::new(&inv.info.invoice_date).size(11.0).color(TEXT_MEDIUM));
                                                            });
                                                            row.col(|ui| {
                                                                ui.label(egui::RichText::new(&inv.info.buyer).size(11.0).color(TEXT_MEDIUM));
                                                            });
                                                            row.col(|ui| {
                                                                ui.label(egui::RichText::new(&inv.info.seller).size(11.0).color(TEXT_MEDIUM));
                                                            });
                                                            row.col(|ui| {
                                                                let amount_color = if inv.info.amount.is_empty() {
                                                                    TEXT_LOW
                                                                } else {
                                                                    ACCENT_SUCCESS
                                                                };
                                                                ui.label(egui::RichText::new(&inv.info.amount).size(11.0).color(amount_color).family(egui::FontFamily::Monospace));
                                                            });
                                                            row.col(|ui| {
                                                                ui.label(egui::RichText::new(&inv.info.remark).size(11.0).color(TEXT_LOW));
                                                            });
                                                        });
                                                    }
                                                });
                                        });
                                });
                        }
                        
                        ui.add_space(20.0);
                    });
            });
    }
}
