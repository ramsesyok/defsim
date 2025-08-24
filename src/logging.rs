//! # Logging モジュール
//! 
//! 防衛シミュレーションのログ管理機能を提供します。
//! 
//! このモジュールは、tracing-appenderを使用した非同期ログ出力システムを提供し、
//! コンソールとファイルへの同時出力、ログレベル制御、処理速度に影響を与えない
//! 非同期書き込みを実現します。
//! 
//! ## 主要機能
//! 
//! - **非同期ログ書き込み**: tracing-appenderによる高速なファイル出力
//! - **出力先選択**: コンソール、ファイル、またはその両方への出力切り替え
//! - **ログレベル制御**: TRACE, DEBUG, INFO, WARN, ERRORレベルの管理
//! - **構造化ログ**: 時刻、レベル、モジュール名を含む構造化されたログフォーマット
//! 
//! ## 設定可能な出力先
//! 
//! - `Console`: コンソールのみ
//! - `File`: ファイルのみ（logs/defsim.log）
//! - `Both`: コンソールとファイルの両方

use std::str::FromStr;
use tracing::{Level};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Registry,
};
use tracing_appender::{non_blocking, rolling};

/// ログ出力先の設定
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogOutput {
    /// コンソールのみ
    Console,
    /// ファイルのみ
    File,
    /// コンソールとファイルの両方
    Both,
}

impl FromStr for LogOutput {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "console" | "stdout" => Ok(LogOutput::Console),
            "file" => Ok(LogOutput::File),
            "both" | "all" => Ok(LogOutput::Both),
            _ => Err(format!("無効な出力先: {}. 利用可能: console, file, both", s)),
        }
    }
}

/// ログ設定構造体
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// ログレベル
    pub level: Level,
    /// 出力先
    pub output: LogOutput,
    /// ログファイルのディレクトリ（Fileまたは Bothの場合）
    pub log_dir: String,
    /// ログファイル名のプレフィックス
    pub file_prefix: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            output: LogOutput::Both,
            log_dir: "logs".to_string(),
            file_prefix: "defsim".to_string(),
        }
    }
}

/// ログシステムを初期化
/// 
/// 指定された設定に基づいてtracing-subscriberを設定し、
/// 非同期ログ出力システムを初期化します。
/// 
/// # 引数
/// 
/// * `config` - ログ設定
/// 
/// # 戻り値
/// 
/// 初期化に成功した場合はOk(())、失敗した場合はエラー
/// 
/// # 例
/// 
/// ```rust
/// use defsim::logging::{LogConfig, LogOutput, init_logging};
/// use tracing::Level;
/// 
/// let config = LogConfig {
///     level: Level::DEBUG,
///     output: LogOutput::Both,
///     log_dir: "logs".to_string(),
///     file_prefix: "defsim".to_string(),
/// };
/// 
/// init_logging(config).expect("ログ初期化に失敗");
/// ```
pub fn init_logging(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 環境変数またはconfigからログレベルを設定
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(config.level.to_string()))
        .unwrap_or_else(|_| EnvFilter::new("info"));

    match config.output {
        LogOutput::Console => {
            // コンソールのみ
            Registry::default()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_target(true)
                        .with_thread_ids(false)
                        .with_file(false)
                        .with_line_number(false)
                        .compact()
                )
                .init();
        }
        LogOutput::File => {
            // ファイルのみ（非同期）
            let file_appender = rolling::daily(&config.log_dir, &config.file_prefix);
            let (non_blocking_appender, _guard) = non_blocking(file_appender);
            
            Registry::default()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_writer(non_blocking_appender)
                        .with_target(true)
                        .with_thread_ids(false)
                        .with_file(false)
                        .with_line_number(false)
                        .json()
                )
                .init();
                
            // _guardをリークさせて非同期書き込みを維持
            std::mem::forget(_guard);
        }
        LogOutput::Both => {
            // コンソールとファイルの両方（非同期）
            let file_appender = rolling::daily(&config.log_dir, &config.file_prefix);
            let (non_blocking_appender, _guard) = non_blocking(file_appender);
            
            Registry::default()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_target(true)
                        .with_thread_ids(false)
                        .with_file(false)
                        .with_line_number(false)
                        .compact()
                )
                .with(
                    fmt::layer()
                        .with_writer(non_blocking_appender)
                        .with_target(true)
                        .with_thread_ids(false)
                        .with_file(false)
                        .with_line_number(false)
                        .json()
                )
                .init();
                
            // _guardをリークさせて非同期書き込みを維持
            std::mem::forget(_guard);
        }
    }

    Ok(())
}

/// ログレベルを文字列から解析
/// 
/// # 引数
/// 
/// * `level_str` - ログレベル文字列 ("trace", "debug", "info", "warn", "error")
/// 
/// # 戻り値
/// 
/// 解析されたログレベル、無効な場合はINFO
pub fn parse_log_level(level_str: &str) -> Level {
    match level_str.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => {
            eprintln!("警告: 無効なログレベル '{}'. INFOを使用します", level_str);
            Level::INFO
        }
    }
}

/// ログディレクトリを作成
/// 
/// ファイル出力が指定されている場合、ログディレクトリが存在しない時に作成します。
/// 
/// # 引数
/// 
/// * `log_dir` - ログディレクトリパス
/// 
/// # 戻り値
/// 
/// ディレクトリ作成に成功した場合はOk(())、失敗した場合はエラー
pub fn ensure_log_directory(log_dir: &str) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(log_dir)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_output_from_str() {
        assert_eq!(LogOutput::from_str("console"), Ok(LogOutput::Console));
        assert_eq!(LogOutput::from_str("file"), Ok(LogOutput::File));
        assert_eq!(LogOutput::from_str("both"), Ok(LogOutput::Both));
        assert!(LogOutput::from_str("invalid").is_err());
    }

    #[test]
    fn test_parse_log_level() {
        assert_eq!(parse_log_level("debug"), Level::DEBUG);
        assert_eq!(parse_log_level("INFO"), Level::INFO);
        assert_eq!(parse_log_level("invalid"), Level::INFO);
    }
}