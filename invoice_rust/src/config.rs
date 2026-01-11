use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 应用配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 线程池大小，用于并行处理发票文件
    pub thread_pool_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            thread_pool_size: 32,
        }
    }
}

impl Config {
    /// 获取配置文件路径
    fn config_file_path() -> PathBuf {
        // 将配置文件放在可执行文件同目录下
        let exe_path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."));
        let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));
        exe_dir.join("config.toml")
    }

    /// 从配置文件加载配置
    pub fn load() -> Self {
        let config_path = Self::config_file_path();
        
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    return config;
                }
            }
        }
        
        // 如果配置文件不存在或加载失败，创建默认配置文件
        let default_config = Config::default();
        let _ = default_config.save();
        default_config
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_file_path();
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        
        fs::write(&config_path, content)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;
        
        Ok(())
    }

    /// 获取线程池大小，确保在合理范围内
    pub fn get_thread_pool_size(&self) -> usize {
        // 限制线程数在1-128之间
        self.thread_pool_size.clamp(1, 128)
    }
}
