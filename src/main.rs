mod models;
mod scenario;
mod simulation;

use clap::{Arg, Command};
use models::{Position3D as ModelPosition3D, *};
use scenario::*;
use simulation::SimulationEngine;

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
        .get_matches();

    println!("防衛シミュレーション (Defense Simulation) - defsim v0.1.0");
    println!();

    // 詳細レベルの設定
    let verbose_level = matches.get_count("verbose");
    if verbose_level > 0 {
        println!("詳細出力レベル: {}", verbose_level);
    }

    // テストモードの実行
    if matches.get_flag("test") {
        println!("=== エージェントモデルテストモード ===");
        test_agent_models();
        return;
    }

    // シナリオファイルの処理
    if let Some(scenario_path) = matches.get_one::<String>("scenario") {
        match run_scenario(scenario_path, matches.get_flag("info"), verbose_level) {
            Ok(_) => {
                if verbose_level > 0 {
                    println!("シナリオ実行が正常に完了しました。");
                }
            }
            Err(e) => {
                eprintln!("エラー: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // デフォルト動作: 利用可能なシナリオ一覧を表示
        show_default_help();
    }
}

fn test_agent_models() {
    println!("\n=== エージェントモデルのテスト ===");
    
    // 指揮所の作成
    let command_post_pos = ModelPosition3D::new(800000.0, -800000.0, 0.0);
    let command_post = CommandPost::new(
        "CP001".to_string(),
        command_post_pos,
        20000.0, // 到達範囲
    );
    println!("指揮所が作成されました: {}", command_post.get_id());
    
    // センサーの作成
    let sensor_pos = ModelPosition3D::new(750000.0, -900000.0, 50.0);
    let sensor = Sensor::new(
        "S001".to_string(),
        sensor_pos,
    );
    println!("センサーが作成されました: {}", sensor.get_id());
    
    // ランチャーの作成
    let launcher_pos = ModelPosition3D::new(780000.0, -850000.0, 20.0);
    let launcher = Launcher::new(
        "L001".to_string(),
        launcher_pos,
    );
    println!("ランチャーが作成されました: {}", launcher.get_id());
    
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
    println!("ターゲットグループが作成されました: {} 機の敵", targets.len());
    
    // ミサイルの作成テスト
    let missile = Missile::new(
        "L001_M001".to_string(),
        launcher_pos,
        "G001_wave1_T001".to_string(),
    );
    println!("ミサイルが作成されました: {}", missile.get_id());
    
    println!("\n全てのエージェントモデルが正常に作成されました！");
}

/// シナリオファイルを読み込んで実行
fn run_scenario(scenario_path: &str, info_only: bool, verbose_level: u8) -> Result<(), Box<dyn std::error::Error>> {
    // シナリオファイルの読み込み
    let scenario = ScenarioConfig::from_file(scenario_path)?;
    
    if verbose_level > 0 {
        println!("シナリオファイル読み込み完了: {}", scenario_path);
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
    println!();
    
    if verbose_level > 0 {
        println!("シミュレーション設定:");
        println!("  時間刻み: {:.3}秒", scenario.sim.dt_s);
        println!("  最大時間: {:.1}秒", scenario.sim.t_max_s);
        println!("  シード値: {}", scenario.sim.seed);
        println!();
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
    println!("使用方法:");
    println!("  defsim [オプション]");
    println!();
    println!("オプション:");
    println!("  -s, --scenario <FILE>  シナリオファイルを指定して実行");
    println!("  -i, --info             シナリオ情報のみ表示");
    println!("  -t, --test             エージェントモデルのテスト実行");
    println!("  -v, --verbose          詳細出力 (複数指定で詳細レベル上昇)");
    println!("  -h, --help             このヘルプを表示");
    println!();
    println!("利用可能なシナリオファイル:");
    println!("  scenarios/scenario_simple_test.yaml     - 基本テスト用");
    println!("  scenarios/scenario_plane.yaml           - 標準シナリオ");
    println!("  scenarios/scenario_multi_wave.yaml      - 多波攻撃シナリオ");
    println!("  scenarios/scenario_performance_test.yaml - 性能テスト用");
    println!("  scenarios/scenario_validation_test.yaml - 検証テスト用");
    println!();
    println!("例:");
    println!("  defsim -s scenarios/scenario_simple_test.yaml");
    println!("  defsim -s scenarios/scenario_plane.yaml -v");
    println!("  defsim -s scenarios/scenario_multi_wave.yaml -i");
    println!("  defsim --test");
}
