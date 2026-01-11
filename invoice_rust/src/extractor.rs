use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;
use rayon::prelude::*;
use std::sync::Mutex;

/// 发票信息结构
#[derive(Debug, Clone)]
pub struct InvoiceInfo {
    pub invoice_number: String,
    pub invoice_date: String,
    pub buyer: String,
    pub buyer_tax_number: String,
    pub seller: String,
    pub seller_tax_number: String,
    pub item_content: String,
    pub amount: String,
    pub remark: String,
}

impl Default for InvoiceInfo {
    fn default() -> Self {
        Self {
            invoice_number: String::new(),
            invoice_date: String::new(),
            buyer: String::new(),
            buyer_tax_number: String::new(),
            seller: String::new(),
            seller_tax_number: String::new(),
            item_content: String::new(),
            amount: String::new(),
            remark: String::new(),
        }
    }
}

/// 从PDF发票中提取信息
pub fn extract_invoice_info(
    pdf_path: &Path,
    buyer_keyword: Option<&str>,
) -> Result<InvoiceInfo, String> {
    let buyer_keyword = buyer_keyword.unwrap_or("");

    // 提取PDF文本
    let text = match pdf_extract::extract_text(pdf_path) {
        Ok(t) => t,
        Err(e) => {
            return Ok(InvoiceInfo {
                remark: format!("解析错误: {}", e),
                ..Default::default()
            });
        }
    };

    let mut info = InvoiceInfo::default();

    // 提取发票号码 (20位纯数字)
    let fp_regex = Regex::new(r"\b(\d{20})\b").unwrap();
    if let Some(caps) = fp_regex.captures(&text) {
        info.invoice_number = caps.get(1).unwrap().as_str().to_string();
    }

    // 提取开票日期
    let date_regex = Regex::new(r"(\d{4})年(\d{1,2})月(\d{1,2})日").unwrap();
    if let Some(caps) = date_regex.captures(&text) {
        let year = caps.get(1).unwrap().as_str();
        let month = format!("{:02}", caps.get(2).unwrap().as_str().parse::<u32>().unwrap_or(0));
        let day = format!("{:02}", caps.get(3).unwrap().as_str().parse::<u32>().unwrap_or(0));
        info.invoice_date = format!("{}-{}-{}", year, month, day);
    }

    // 提取税号（18位，可能包含字母）
    let tax_regex = Regex::new(r"\b[0-9A-Z]{18}\b").unwrap();
    let tax_numbers: Vec<String> = tax_regex
        .find_iter(&text)
        .map(|m| m.as_str().to_string())
        .collect();
    
    let valid_taxes: Vec<String> = tax_numbers
        .into_iter()
        .filter(|t| !(t.chars().all(|c| c.is_ascii_digit()) && t.len() == 20))
        .collect();

    if valid_taxes.len() >= 1 {
        info.buyer_tax_number = valid_taxes[0].clone();
    }
    if valid_taxes.len() >= 2 {
        info.seller_tax_number = valid_taxes[1].clone();
    }

    // 提取项目内容
    let item_regex = Regex::new(r"\*([^*]+)\*").unwrap();
    if let Some(caps) = item_regex.captures(&text) {
        let item = caps.get(0).unwrap().as_str();
        info.item_content = item.chars().take(30).collect();
    }

    // 排除模式
    let exclude_patterns = vec![
        r"\*[^*]+\*",
        r"项目|规格|单位|数量|单价|金额|税率|税额|合计|备注|开票人|下载次数|发票号码|开票日期",
        r"国家税务总局|发票监制|电子发票|普通发票|广东省税务局",
        r"价税合计|大写|小写",
    ];

    // 销售方关键词
    let seller_keywords = vec![
        "有限公司", "股份有限公司", "科技", "网络", "文化", "婴童",
        "贸易", "酒店", "饭店", "娱乐", "百货", "商店",
        "餐饮店", "饮食店", "加油站", "石油化工",
        "商行", "电子商务商行",
    ];

    // 提取所有可能的销售方名称
    let mut all_sellers = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    
    for line in lines {
        let line = line.trim();
        if line.len() < 5 || line.len() > 60 {
            continue;
        }
        
        // 检查排除模式
        let should_exclude = exclude_patterns.iter().any(|pattern| {
            Regex::new(pattern).unwrap().is_match(line)
        });
        if should_exclude {
            continue;
        }

        // 检查是否包含销售方关键词
        if seller_keywords.iter().any(|kw| line.contains(kw)) {
            if !line.ends_with('费') {
                if !all_sellers.contains(&line.to_string()) {
                    all_sellers.push(line.to_string());
                }
            }
        }
    }

    // 识别购买方（根据传入的关键词匹配）
    if !buyer_keyword.is_empty() {
        for seller in &all_sellers {
            if seller.contains(buyer_keyword) {
                info.buyer = seller.clone();
                break;
            }
        }
    }
    if info.buyer.is_empty() && !all_sellers.is_empty() {
        info.buyer = all_sellers[0].clone();
    }

    // 销售方是第二个不同的商家
    for seller in &all_sellers {
        if seller != &info.buyer {
            info.seller = seller.clone();
            break;
        }
    }

    // 如果没找到，尝试从税号附近提取
    if info.seller.is_empty() && !info.seller_tax_number.is_empty() {
        let tax = &info.seller_tax_number;
        if let Some(idx) = text.find(tax) {
            let start = idx.saturating_sub(100);
            let end = (idx + 100).min(text.len());
            let context = &text[start..end];
            
            let keywords = vec!["店", "商行", "有限公司", "商贸", "科技", "贸易", "酒店", "饭店", "餐饮"];
            for kw in keywords {
                let escaped_kw = regex::escape(kw);
                let pattern = format!(r"([^\s\n]+{}[^\s\n]*)", escaped_kw);
                if let Ok(kw_regex) = Regex::new(&pattern) {
                    if let Some(caps) = kw_regex.captures(context) {
                        let seller = caps.get(1).unwrap().as_str()
                            .trim_matches(|c: char| c == '*' || c == '、' || c == '。' || c == '.' || c == '\n' || c == '\t' || c == '\r');
                        if seller != info.buyer && seller.len() > 4 {
                            info.seller = seller.to_string();
                            break;
                        }
                    }
                }
            }
        }
    }

    // 提取金额 - 优先找"圆整"后的金额
    let yuanzheng_regex = Regex::new(r"圆整\s*[¥￥]?\s*([\d,]+\.?\d*)").unwrap();
    if let Some(caps) = yuanzheng_regex.captures(&text) {
        info.amount = caps.get(1).unwrap().as_str().replace(',', "");
    } else {
        // 找所有¥后的金额，取最大的（价税合计通常是最大的）
        let amount_regex = Regex::new(r"[¥￥]\s*([\d,]+\.?\d*)").unwrap();
        let mut amounts_float: Vec<(f64, String)> = Vec::new();
        
        for caps in amount_regex.captures_iter(&text) {
            if let Some(amt_str) = caps.get(1) {
                let amt_str_clean = amt_str.as_str().replace(',', "");
                if let Ok(amt) = amt_str_clean.parse::<f64>() {
                    if amt > 0.0 && amt < 10000000.0 {
                        amounts_float.push((amt, amt_str.as_str().to_string()));
                    }
                }
            }
        }
        
        if !amounts_float.is_empty() {
            let max_amount = amounts_float.iter().max_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            if let Some(max) = max_amount {
                info.amount = max.1.replace(',', "");
            }
        }
    }

    // 从文件名提取金额（备用方案）
    if info.amount.is_empty() {
        if let Some(filename) = pdf_path.file_name() {
            let filename_str = filename.to_string_lossy();
            let filename_regex = Regex::new(r"(\d+\.?\d*)\.pdf").unwrap();
            if let Some(caps) = filename_regex.captures(&filename_str) {
                info.amount = caps.get(1).unwrap().as_str().to_string();
            }
        }
    }

    Ok(info)
}

/// 发票文件信息
#[derive(Debug, Clone)]
pub struct InvoiceFile {
    pub folder: String,
    pub filename: String,
    pub file_type: String,
    pub info: InvoiceInfo,
}

impl Default for InvoiceFile {
    fn default() -> Self {
        Self {
            folder: String::new(),
            filename: String::new(),
            file_type: String::new(),
            info: InvoiceInfo::default(),
        }
    }
}

/// 处理结果
#[derive(Clone)]
pub struct ProcessResult {
    pub output_file: String,
    pub invoices: Vec<InvoiceFile>,
}

/// 处理所有发票文件并生成Excel
/// 使用默认线程数（从配置文件加载）
#[allow(dead_code)]
pub fn process_invoices(
    base_path: &Path,
    buyer_keyword: Option<&str>,
    output_path: Option<&Path>,
) -> Result<ProcessResult, String> {
    process_invoices_with_threads(base_path, buyer_keyword, output_path, None)
}

/// 使用指定线程数处理所有发票文件并生成Excel
pub fn process_invoices_with_threads(
    base_path: &Path,
    buyer_keyword: Option<&str>,
    output_path: Option<&Path>,
    thread_count: Option<usize>,
) -> Result<ProcessResult, String> {
    // 设置线程池大小
    if let Some(threads) = thread_count {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .ok(); // 忽略错误,因为全局线程池可能已经初始化
    }

    // 首先收集所有需要处理的文件信息
    let file_entries: Vec<_> = WalkDir::new(base_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    // 使用 Mutex 保护共享的结果向量
    let all_invoices = Mutex::new(Vec::new());

    // 并行处理所有文件
    file_entries.par_iter().for_each(|entry| {
        let file_path = entry.path();
        let file_name = entry.file_name().to_string_lossy();
        
        // 跳过隐藏文件
        if file_name.starts_with('.') {
            return;
        }

        let file_ext = file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_uppercase();

        if !matches!(file_ext.as_str(), "PDF" | "PNG" | "JPG" | "JPEG") {
            return;
        }

        let rel_path = entry
            .path()
            .parent()
            .and_then(|p| p.strip_prefix(base_path).ok())
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_string();

        let mut invoice_file = InvoiceFile {
            folder: rel_path,
            filename: file_name.to_string(),
            file_type: file_ext.clone(),
            info: InvoiceInfo::default(),
        };

        // 处理PDF文件
        if file_ext == "PDF" {
            match extract_invoice_info(file_path, buyer_keyword) {
                Ok(info) => invoice_file.info = info,
                Err(e) => {
                    invoice_file.info.remark = format!("处理错误: {}", e);
                }
            }
        }

        // 将结果添加到共享向量中
        if let Ok(mut invoices) = all_invoices.lock() {
            invoices.push(invoice_file);
        }
    });

    // 从 Mutex 中取出结果
    let mut all_invoices = all_invoices.into_inner()
        .map_err(|e| format!("获取处理结果失败: {}", e))?;

    // 从图片文件名提取金额
    for inv in &mut all_invoices {
        if inv.info.amount.is_empty() && !inv.filename.to_uppercase().ends_with(".PDF") {
            let filename_regex = Regex::new(r"(\d+\.?\d*)\.(?:PNG|JPG|JPEG)").unwrap();
            if let Some(caps) = filename_regex.captures(&inv.filename.to_uppercase()) {
                inv.info.amount = caps.get(1).unwrap().as_str().to_string();
            }
        }
    }

    // 排序
    all_invoices.sort_by(|a, b| {
        a.folder
            .cmp(&b.folder)
            .then_with(|| a.filename.cmp(&b.filename))
    });

    // 生成Excel
    let output_file = output_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| base_path.join("发票清单.xlsx"));

    generate_excel(&all_invoices, &output_file)?;

    // 统计识别率
    let pdf_count = all_invoices
        .iter()
        .filter(|inv| inv.filename.to_uppercase().ends_with(".PDF"))
        .count();
    let with_seller = all_invoices
        .iter()
        .filter(|inv| {
            !inv.info.seller.is_empty() && !inv.info.seller.starts_with('*')
        })
        .count();
    let with_amount = all_invoices
        .iter()
        .filter(|inv| !inv.info.amount.is_empty())
        .count();

    let seller_rate = if pdf_count > 0 {
        (with_seller as f64 / pdf_count as f64) * 100.0
    } else {
        0.0
    };
    let amount_rate = if !all_invoices.is_empty() {
        (with_amount as f64 / all_invoices.len() as f64) * 100.0
    } else {
        0.0
    };

    println!("发票识别完成！");
    println!("  总文件数: {}", all_invoices.len());
    println!("  PDF发票数: {}", pdf_count);
    println!("  销售方识别率: {:.1}%", seller_rate);
    println!("  金额识别率: {:.1}%", amount_rate);
    println!("\nExcel已保存: {}", output_file.display());

    Ok(ProcessResult {
        output_file: output_file.to_string_lossy().to_string(),
        invoices: all_invoices,
    })
}

/// 生成Excel文件
fn generate_excel(invoices: &[InvoiceFile], output_path: &Path) -> Result<(), String> {
    use rust_xlsxwriter::*;

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // 设置列宽
    worksheet.set_column_width(0, 6.0).map_err(|e| format!("设置列宽失败: {}", e))?;   // 序号
    worksheet.set_column_width(1, 26.0).map_err(|e| format!("设置列宽失败: {}", e))?;  // 文件夹
    worksheet.set_column_width(2, 36.0).map_err(|e| format!("设置列宽失败: {}", e))?;  // 文件名
    worksheet.set_column_width(3, 18.0).map_err(|e| format!("设置列宽失败: {}", e))?;  // 发票号码
    worksheet.set_column_width(4, 11.0).map_err(|e| format!("设置列宽失败: {}", e))?;  // 开票日期
    worksheet.set_column_width(5, 22.0).map_err(|e| format!("设置列宽失败: {}", e))?;  // 购买方
    worksheet.set_column_width(6, 16.0).map_err(|e| format!("设置列宽失败: {}", e))?;  // 购买方税号
    worksheet.set_column_width(7, 28.0).map_err(|e| format!("设置列宽失败: {}", e))?;  // 销售方
    worksheet.set_column_width(8, 16.0).map_err(|e| format!("设置列宽失败: {}", e))?;  // 销售方税号
    worksheet.set_column_width(9, 18.0).map_err(|e| format!("设置列宽失败: {}", e))?;  // 项目内容
    worksheet.set_column_width(10, 10.0).map_err(|e| format!("设置列宽失败: {}", e))?; // 金额
    worksheet.set_column_width(11, 12.0).map_err(|e| format!("设置列宽失败: {}", e))?; // 备注

    // 表头样式
    let header_format = Format::new()
        .set_bold()
        .set_font_size(10)
        .set_font_color(Color::RGB(0xFFFFFF))
        .set_background_color(Color::RGB(0x4472C4))
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_border(FormatBorder::Thin);

    // 表头
    let headers = vec![
        "序号", "文件夹", "文件名", "发票号码", "开票日期", "购买方",
        "购买方税号", "销售方", "销售方税号", "项目内容", "金额", "备注",
    ];

    for (col, header) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, col as u16, *header, &header_format)
            .map_err(|e| format!("写入表头失败: {}", e))?;
    }

    // 数据格式
    let border_format = Format::new().set_border(FormatBorder::Thin);

    // 写入数据
    for (idx, inv) in invoices.iter().enumerate() {
        let row = (idx + 1) as u32;
        worksheet.write_number_with_format(row, 0, (idx + 1) as f64, &border_format)
            .map_err(|e| format!("写入数据失败: {}", e))?;
        worksheet.write_string_with_format(row, 1, &inv.folder, &border_format)
            .map_err(|e| format!("写入数据失败: {}", e))?;
        worksheet.write_string_with_format(row, 2, &inv.filename, &border_format)
            .map_err(|e| format!("写入数据失败: {}", e))?;
        worksheet.write_string_with_format(row, 3, &inv.info.invoice_number, &border_format)
            .map_err(|e| format!("写入数据失败: {}", e))?;
        worksheet.write_string_with_format(row, 4, &inv.info.invoice_date, &border_format)
            .map_err(|e| format!("写入数据失败: {}", e))?;
        worksheet.write_string_with_format(row, 5, &inv.info.buyer, &border_format)
            .map_err(|e| format!("写入数据失败: {}", e))?;
        worksheet.write_string_with_format(row, 6, &inv.info.buyer_tax_number, &border_format)
            .map_err(|e| format!("写入数据失败: {}", e))?;
        worksheet.write_string_with_format(row, 7, &inv.info.seller, &border_format)
            .map_err(|e| format!("写入数据失败: {}", e))?;
        worksheet.write_string_with_format(row, 8, &inv.info.seller_tax_number, &border_format)
            .map_err(|e| format!("写入数据失败: {}", e))?;
        worksheet.write_string_with_format(row, 9, &inv.info.item_content, &border_format)
            .map_err(|e| format!("写入数据失败: {}", e))?;
        worksheet.write_string_with_format(row, 10, &inv.info.amount, &border_format)
            .map_err(|e| format!("写入数据失败: {}", e))?;
        worksheet.write_string_with_format(row, 11, &inv.info.remark, &border_format)
            .map_err(|e| format!("写入数据失败: {}", e))?;
    }

    // 冻结首行
    worksheet.set_freeze_panes(1, 0)
        .map_err(|e| format!("冻结首行失败: {}", e))?;

    workbook.save(output_path).map_err(|e| format!("保存Excel失败: {}", e))?;

    Ok(())
}
