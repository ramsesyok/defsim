mod models;
mod scenario;
mod simulation;
mod logging;

use clap::{Arg, Command};
use models::{Position3D as ModelPosition3D, *};
use scenario::*;
use simulation::SimulationEngine;
use logging::{LogConfig, LogOutput, init_logging, parse_log_level, ensure_log_directory};
use tracing::{info, warn, error, debug, trace};

fn main() {
    // コマンドライン引数の解析
    let matches = Command::new("defsim")
        .version("0.1.0")
        .about("防衛シミュレーション (Defense Simulation)")
        .long_about("エージェントベースの防衛シミュレーションシステム\n\
                     時間駆動型シミュレーションでミサイル防衛の戦術評価を行います。")
        .arg(
            Arg::new("scenario")
                .short('s')
                .long("scenario")
                .value_name("FILE")
                .help("シナリオファイル(.yaml)のパスを指定")
                .long_help("実行するシナリオファイル(.yaml)のパスを指定します。\n\
                           指定しない場合、デフォルトのテストモードで実行されます。")
        )
        .arg(
            Arg::new("info")
                .short('i')
                .long("info")
                .action(clap::ArgAction::SetTrue)
                .help("シナリオの情報のみ表示して終了")
                .conflicts_with("test")
        )
        .arg(
            Arg::new("test")
                .short('t')
                .long("test")
                .action(clap::ArgAction::SetTrue)
                .help("エージェントモデルのテストを実行")
                .conflicts_with("info")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::Count)
                .help("詳細出力レベル (-v: 基本, -vv: 詳細, -vvv: デバッグ)")
        )
        .arg(
            Arg::new("log-level")
                .long("log-level")
                .value_name("LEVEL")
                .help("ログレベルを指定 (trace, debug, info, warn, error)")
                .default_value("info")
        )
        .arg(
            Arg::new("log-output")
                .long("log-output")
                .value_name("OUTPUT")
                .help("ログ出力先を指定 (console, file, both)")
                .default_value("both")
        )
        .arg(
            Arg::new("log-dir")
                .long("log-dir")
                .value_name("DIR")
                .help("ログファイルの出力ディレクトリ")
                .default_value("logs")
        )
        .get_matches();

    // ログ設定の初期化
    let log_level_str = matches.get_one::<String>("log-level").unwrap();
    let log_output_str = matches.get_one::<String>("log-output").unwrap();
    let log_dir = matches.get_one::<String>("log-dir").unwrap();
    let verbose_level = matches.get_count("verbose");

    // ログレベルを verbose_level も考慮して決定
    let log_level = if verbose_level > 0 {
        match verbose_level {
            1 => tracing::Level::INFO,
            2 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        }
    } else {
        parse_log_level(log_level_str)
    };

    let log_output = match log_output_str.parse::<LogOutput>() {
        Ok(output) => output,
        Err(e) => {
            eprintln!("エラー: {}", e);
            std::process::exit(1);
        }
    };

    // ログディレクトリ作成（ファイル出力が必要な場合）
    if matches!(log_output, LogOutput::File | LogOutput::Both) {
        if let Err(e) = ensure_log_directory(log_dir) {
            eprintln!("ログディレクトリ作成エラー: {}", e);
            std::process::exit(1);
        }
    }

    let log_config = LogConfig {
        level: log_level,
        output: log_output,
        log_dir: log_dir.clone(),
        file_prefix: "defsim".to_string(),
    };

    // ログ初期化
    if let Err(e) = init_logging(log_config) {
        eprintln!("ログ初期化エラー: {}", e);
        std::process::exit(1);
    }

    info!("防衛シミュレーション (Defense Simulation) - defsim v0.1.0");
    
    if verbose_level > 0 {
        info!("詳細出力レベル: {}", verbose_level);
        debug!("ログレベル: {:?}", log_level);
        debug!("ログ出力先: {:?}", log_output);
    }

    // テストモードの実行
    if matches.get_flag("test") {
        info!("=== エージェントモデルテストモード ===");
        test_agent_models();
        return;
    }

    // シナリオファイルの処理
    if let Some(scenario_path) = matches.get_one::<String>("scenario") {
        match run_scenario(scenario_path, matches.get_flag("info"), verbose_level) {
            Ok(_) => {
                if verbose_level > 0 {
                    info!("シナリオ実行が正常に完了しました。");
                }
            }
            Err(e) => {
                error!("エラー: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // デフォルト動作: 利用可能なシナリオ一覧を表示
        show_default_help();
    }
}

fn test_agent_models() {
    info!("\n=== エージェントモデルのテスト ===");
    
    // 指揮所の作成
    let command_post_pos = ModelPosition3D::new(800000.0, -800000.0, 0.0);
    let command_post = CommandPost::new(
        "CP001".to_string(),
        command_post_pos,
        20000.0, // 到達範囲
    );
    info!("指揮所が作成されました: {}", command_post.get_id());
    
    // センサーの作成
    let sensor_pos = ModelPosition3D::new(750000.0, -900000.0, 50.0);
    let sensor = Sensor::new(
        "S001".to_string(),
        sensor_pos,
    );
    info!("センサーが作成されました: {}", sensor.get_id());
    
    // ランチャーの作成
    let launcher_pos = ModelPosition3D::new(780000.0, -850000.0, 20.0);
    let launcher = Launcher::new(
        "L001".to_string(),
        launcher_pos,
    );
    info!("ランチャーが作成されました: {}", launcher.get_id());
    
    // ターゲットグループの作成
    let group_center = ModelPosition3D::new(-800000.0, 600000.0, 3000.0);
    let target_group = TargetGroup {
        id: "G001_wave1".to_string(),
        center_position: group_center,
        count: 12,
        ring_spacing: 1500.0,
        start_angle: 0.0,
        ring_half_offset: true,
        endurance: 2,
        spawn_time: 120.0,
        speed: 200.0,
        destination: command_post_pos,
        arrival_radius: 20000.0,
    };
    
    let targets = target_group.generate_targets();
    info!("ターゲットグループが作成されました: {} 機の敵", targets.len());
    
    // ミサイルの作成テスト
    let missile = Missile::new(
        "L001_M001".to_string(),
        launcher_pos,
        "G001_wave1_T001".to_string(),
    );
    info!("ミサイルが作成されました: {}", missile.get_id());
    
    info!("\n全てのエージェントモデルが正常に作成されました！");
    
    // ターゲットイベントログのテスト
    info!("=== ターゲットイベントログテスト開始 ===");
    
    // 1つ目のターゲットでダメージテスト
    if !targets.is_empty() {
        let mut test_target = targets[0].clone();
        // テスト用にアクティブ状態にする
        test_target.status = AgentStatus::Active;
        info!("ダメージテストを実行: {}", test_target.get_id());
        test_target.take_damage(1);  // 1ダメージ
        test_target.take_damage(2);  // 最終ダメージで破壊
    }
    
    // 2つ目のターゲットで到達テスト（手動で目的地近くに移動）
    if targets.len() > 1 {
        let mut test_target = targets[1].clone();
        // テスト用にアクティブ状態にする
        test_target.status = AgentStatus::Active;
        info!("到達テストを実行: {}", test_target.get_id());
        // 目的地近くに移動
        test_target.set_position(ModelPosition3D::new(
            command_post_pos.x + 10000.0,  // 到達範囲内
            command_post_pos.y + 10000.0,
            0.0
        ));
        test_target.check_arrival();
    }
    
    // 3つ目のターゲットで領域外テスト
    if targets.len() > 2 {
        let mut test_target = targets[2].clone();
        // テスト用にアクティブ状態にする
        test_target.status = AgentStatus::Active;
        info!("領域外テストを実行: {}", test_target.get_id());
        // 領域外に移動
        test_target.set_position(ModelPosition3D::new(
            2_000_000.0,  // 領域外
            2_000_000.0,
            0.0
        ));
        test_target.check_out_of_bounds();
    }
    
    info!("=== ターゲットイベントログテスト完了 ===");
}

/// シナリオファイルを読み込んで実行
fn run_scenario(scenario_path: &str, info_only: bool, verbose_level: u8) -> Result<(), Box<dyn std::error::Error>> {
    // シナリオファイルの読み込み
    let scenario = ScenarioConfig::from_file(scenario_path)?;
    
    if verbose_level > 0 {
        info!("シナリオファイル読み込み完了: {}", scenario_path);
    }
    
    // 情報表示のみの場合
    if info_only {
        scenario.print_summary();
        return Ok(());
    }
    
    // シナリオ実行
    execute_scenario(scenario, verbose_level)?;
    
    Ok(())
}

/// シナリオの実行
fn execute_scenario(scenario: ScenarioConfig, verbose_level: u8) -> Result<(), Box<dyn std::error::Error>> {
    // 基本情報表示
    scenario.print_summary();
    
    if verbose_level > 0 {
        debug!("シミュレーション設定:");
        debug!("  時間刻み: {:.3}秒", scenario.sim.dt_s);
        debug!("  最大時間: {:.1}秒", scenario.sim.t_max_s);
        debug!("  シード値: {}", scenario.sim.seed);
    }
    
    // シミュレーションエンジンの作成と初期化
    let mut simulation = SimulationEngine::new(scenario, verbose_level);
    simulation.initialize()?;
    
    // シミュレーション実行
    simulation.run()?;
    
    Ok(())
}

/// デフォルトヘルプとシナリオ一覧を表示
fn show_default_help() {
    info!("使用方法:");
    info!("  defsim [オプション]");
    info!("オプション:");
    info!("  -s, --scenario <FILE>  シナリオファイルを指定して実行");
    info!("  -i, --info             シナリオ情報のみ表示");
    info!("  -t, --test             エージェントモデルのテスト実行");
    info!("  -v, --verbose          詳細出力 (複数指定で詳細レベル上昇)");
    info!("  -h, --help             このヘルプを表示");
    info!("  --log-level <LEVEL>    ログレベル指定 (trace, debug, info, warn, error)");
    info!("  --log-output <OUTPUT>  ログ出力先指定 (console, file, both)");
    info!("  --log-dir <DIR>        ログファイル出力ディレクトリ");
    info!("利用可能なシナリオファイル:");
    info!("  scenarios/scenario_simple_test.yaml     - 基本テスト用");
    info!("  scenarios/scenario_plane.yaml           - 標準シナリオ");
    info!("  scenarios/scenario_multi_wave.yaml      - 多波攻撃シナリオ");
    info!("  scenarios/scenario_performance_test.yaml - 性能テスト用");
    info!("  scenarios/scenario_validation_test.yaml - 検証テスト用");
    info!("例:");
    info!("  defsim -s scenarios/scenario_simple_test.yaml");
    info!("  defsim -s scenarios/scenario_plane.yaml -v");
    info!("  defsim -s scenarios/scenario_multi_wave.yaml -i");
    info!("  defsim --test");
    info!("  defsim -s scenarios/scenario_plane.yaml --log-level debug --log-output file");
}
